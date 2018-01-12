
use std::rc::Rc;
use std::cell::RefCell;
use builder::module;
use elements::{
	deserialize_buffer, ExportEntry, Internal, ImportEntry, External, GlobalEntry, GlobalType,
	InitExpr, ValueType, Opcodes, Opcode, TableType, MemoryType, FunctionType,
};
use validation::{validate_module, ValidatedModule};
use interpreter::{
	Error, GlobalInstance, MemoryInstance, ModuleInstance, RuntimeValue,
	HostError, MemoryRef, ImportsBuilder, Externals, TryInto, TableRef,
	GlobalRef, FuncRef, FuncInstance, ModuleImportResolver,
};
use wabt::wat2wasm;

const SUB_FUNC_INDEX: usize = 0;
const ERR_FUNC_INDEX: usize = 1;
const INC_MEM_FUNC_INDEX: usize = 2;
const GET_MEM_FUNC_INDEX: usize = 3;

#[derive(Debug, Clone, PartialEq)]
struct HostErrorWithCode {
	error_code: u32,
}

impl ::std::fmt::Display for HostErrorWithCode {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		write!(f, "{}", self.error_code)
	}
}

impl HostError for HostErrorWithCode {}

struct TestHost {
	memory: Option<MemoryRef>,
}

impl TestHost {
	fn new() -> TestHost {
		TestHost {
			memory: Some(MemoryInstance::alloc(1, Some(1)).unwrap()),
		}
	}
}

impl Externals for TestHost {
	fn invoke_index(
		&mut self,
		index: usize,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		match index {
			// sub(a: i32, b: i32) -> i32
			SUB_FUNC_INDEX => {
				let mut args = args.iter();
				let a: i32 = args.next().unwrap().try_into().unwrap();
				let b: i32 = args.next().unwrap().try_into().unwrap();

				let result: RuntimeValue = (a - b).into();

				Ok(Some(result))
			}
			// err(error_code: i32) -> !
			ERR_FUNC_INDEX => {
				let mut args = args.iter();
				let error_code: u32 = args.next().unwrap().try_into().unwrap();
				let error = HostErrorWithCode { error_code };
				Err(Error::Host(Box::new(error)))
			}
			// inc_mem(ptr: *mut u8)
			INC_MEM_FUNC_INDEX => {
				let mut args = args.iter();
				let ptr: u32 = args.next().unwrap().try_into().unwrap();

				let memory = self.memory.as_ref().unwrap();
				let mut buf = [0u8; 1];
				memory.get_into(ptr, &mut buf).unwrap();
				buf[0] += 1;
				memory.set(ptr, &buf).unwrap();

				Ok(None)
			}
			// get_mem(ptr: *mut u8) -> u8
			GET_MEM_FUNC_INDEX => {
				let mut args = args.iter();
				let ptr: u32 = args.next().unwrap().try_into().unwrap();

				let memory = self.memory.as_ref().unwrap();
				let mut buf = [0u8; 1];
				memory.get_into(ptr, &mut buf).unwrap();

				Ok(Some(RuntimeValue::I32(buf[0] as i32)))
			}
			_ => panic!("SpecModule doesn't provide function at index {}", index),
		}
	}

	fn check_signature(&self, index: usize, func_type: &FunctionType) -> bool {
		let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
			SUB_FUNC_INDEX => (&[ValueType::I32, ValueType::I32], Some(ValueType::I32)),
			ERR_FUNC_INDEX => (&[ValueType::I32], None),
			INC_MEM_FUNC_INDEX => (&[ValueType::I32], None),
			GET_MEM_FUNC_INDEX => (&[ValueType::I32], Some(ValueType::I32)),
			_ => return false,
		};

		func_type.params() == params && func_type.return_type() == ret_ty
	}
}

impl ModuleImportResolver for TestHost {
	fn resolve_func(&self, field_name: &str, func_type: &FunctionType) -> Result<FuncRef, Error> {
		let index = match field_name {
			"sub" => SUB_FUNC_INDEX,
			"err" => ERR_FUNC_INDEX,
			"inc_mem" => INC_MEM_FUNC_INDEX,
			"get_mem" => GET_MEM_FUNC_INDEX,
			_ => {
				return Err(Error::Instantiation(
					format!("Export {} not found", field_name),
				))
			}
		};

		if !self.check_signature(index, func_type) {
			return Err(Error::Instantiation(format!(
				"Export `{}` doesnt match expected type {:?}",
				field_name,
				func_type
			)));
		}

		Ok(FuncInstance::alloc_host(func_type.clone(), index))
	}

	fn resolve_global(
		&self,
		field_name: &str,
		_global_type: &GlobalType,
	) -> Result<GlobalRef, Error> {
		Err(Error::Instantiation(
			format!("Export {} not found", field_name),
		))
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		_memory_type: &MemoryType,
	) -> Result<MemoryRef, Error> {
		Err(Error::Instantiation(
			format!("Export {} not found", field_name),
		))
	}

	fn resolve_table(&self, field_name: &str, _table_type: &TableType) -> Result<TableRef, Error> {
		Err(Error::Instantiation(
			format!("Export {} not found", field_name),
		))
	}
}

fn parse_wat(source: &str) -> ValidatedModule {
	let wasm_binary = wat2wasm(source).expect("Failed to parse wat source");
	let module = deserialize_buffer(wasm_binary).expect("Failed to deserialize module");
	let validated_module = validate_module(module).expect("Failed to validate module");
	validated_module
}

#[test]
fn call_host_func() {
	let module = parse_wat(
		r#"
(module
	(import "env" "sub" (func $sub (param i32 i32) (result i32)))

	(func (export "test") (result i32)
		(call $sub
			(i32.const 5)
			(i32.const 7)
		)
	)
)
"#,
	);

	let mut env = TestHost::new();

	let instance = ModuleInstance::new(
		&module,
		&ImportsBuilder::default().with_resolver("env", &env),
	).expect("Failed to instantiate module")
		.assert_no_start();

	assert_eq!(
		instance
			.invoke_export("test", &[], &mut env)
			.expect("Failed to invoke 'test' function"),
		Some(RuntimeValue::I32(-2))
	);
}

#[test]
fn host_err() {
	let module = parse_wat(
		r#"
(module
	(import "env" "err" (func $err (param i32)))

	(func (export "test")
		(call $err
			(i32.const 228)
		)
	)
)
"#,
	);

	let mut env = TestHost::new();

	let instance = ModuleInstance::new(
		&module,
		&ImportsBuilder::default().with_resolver("env", &env),
	).expect("Failed to instantiate module")
		.assert_no_start();

	let error = instance
		.invoke_export("test", &[], &mut env)
		.expect_err("`test` expected to return error");

	let host_error: Box<HostError> = match error {
		Error::Host(err) => err,
		err => panic!("Unexpected error {:?}", err),
	};

	let error_with_code = host_error
		.downcast_ref::<HostErrorWithCode>()
		.expect("Failed to downcast to expected error type");
	assert_eq!(error_with_code.error_code, 228);
}

#[test]
fn modify_mem_with_host_funcs() {
	let module = parse_wat(
	r#"
(module
	(import "env" "inc_mem" (func $inc_mem (param i32)))
	;; (import "env" "get_mem" (func $get_mem (param i32) (result i32)))

	(func (export "modify_mem")
		;; inc memory at address 12 for 4 times.
		(call $inc_mem (i32.const 12))
		(call $inc_mem (i32.const 12))
		(call $inc_mem (i32.const 12))
		(call $inc_mem (i32.const 12))
	)
)
"#,
	);

	let mut env = TestHost::new();

	let instance = ModuleInstance::new(
		&module,
		&ImportsBuilder::default().with_resolver("env", &env),
	).expect("Failed to instantiate module")
		.assert_no_start();

	instance
		.invoke_export("modify_mem", &[], &mut env)
		.expect("Failed to invoke 'test' function");

	// Check contents of memory at address 12.
	let mut buf = [0u8; 1];
	env.memory.unwrap().get_into(12, &mut buf).unwrap();

	assert_eq!(&buf, &[4]);
}

#[test]
fn pull_internal_mem_from_module() {
	let module = parse_wat(
	r#"
(module
	(import "env" "inc_mem" (func $inc_mem (param i32)))
	(import "env" "get_mem" (func $get_mem (param i32) (result i32)))

	;; declare internal memory and export it under name "mem"
	(memory (export "mem") 1 1)

	(func (export "test") (result i32)
		;; Increment value at address 1337
		(call $inc_mem (i32.const 1337))

		;; Return value at address 1337
		(call $get_mem (i32.const 1337))
	)
)
"#,
	);

	let mut env = TestHost {
		memory: None,
	};

	let instance = ModuleInstance::new(
		&module,
		&ImportsBuilder::default().with_resolver("env", &env),
	).expect("Failed to instantiate module")
		.assert_no_start();

	// Get memory instance exported by name 'mem' from the module instance.
	let internal_mem = instance
		.export_by_name("mem")
		.expect("Module expected to have 'mem' export")
		.as_memory()
		.expect("'mem' export should be a memory");

	env.memory = Some(internal_mem);

	assert_eq!(
		instance.invoke_export("test", &[], &mut env).unwrap(),
		Some(RuntimeValue::I32(1))
	);
}
