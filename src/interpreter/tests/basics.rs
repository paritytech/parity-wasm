///! Basic tests for instructions/constructions, missing in wabt tests

use std::rc::Rc;
use std::cell::RefCell;
use builder::module;
use elements::{ExportEntry, Internal, ImportEntry, External, GlobalEntry, GlobalType,
	InitExpr, ValueType, Opcodes, Opcode, TableType, MemoryType};
use interpreter::{
	Error, GlobalInstance, HostModule, HostModuleBuilder, MemoryInstance,
	ModuleInstance, ProgramInstance, RuntimeValue, UserError
};
use super::utils::program_with_default_env;

#[test]
fn import_function() {
	let module1 = module()
		.with_export(ExportEntry::new("external_func".into(), Internal::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(3),
				Opcode::End,
			])).build()
			.build()
		.build();

	let module2 = module()
		.with_import(ImportEntry::new("external_module".into(), "external_func".into(), External::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::Call(0),
				Opcode::I32Const(7),
				Opcode::I32Add,
				Opcode::End,
			])).build()
			.build()
		.build();

	let external_module = ModuleInstance::instantiate(&module1)
		.assert_no_start()
		.unwrap();
	let main_module = ModuleInstance::instantiate(&module2)
		.with_import("external_module", &*external_module)
		.assert_no_start()
		.unwrap();

	assert_eq!(external_module.invoke_index(0, vec![], &mut ()).unwrap().unwrap(), RuntimeValue::I32(3));
	assert_eq!(main_module.invoke_index(1, vec![], &mut ()).unwrap().unwrap(), RuntimeValue::I32(10));
}

#[test]
fn wrong_import() {
	let side_module = module()
		.with_export(ExportEntry::new("cool_func".into(), Internal::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(3),
				Opcode::End,
			])).build()
			.build()
		.build();

	let module = module()
		.with_import(ImportEntry::new("side_module".into(), "not_cool_func".into(), External::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::Call(0),
				Opcode::I32Const(7),
				Opcode::I32Add,
				Opcode::End,
			])).build()
			.build()
		.build();

	let mut program = ProgramInstance::new();
	let _side_module_instance = program.add_module("side_module", side_module, &mut ()).unwrap();
	assert!(program.add_module("main", module, &mut ()).is_err());
}

#[test]
fn global_get_set() {
	let module = module()
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(42), Opcode::End])))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::GetGlobal(0),
				Opcode::I32Const(8),
				Opcode::I32Add,
				Opcode::SetGlobal(0),
				Opcode::GetGlobal(0),
				Opcode::End,
			])).build()
			.build()
		.build();

	let mut program = ProgramInstance::new();
	program.add_module("main", module, &mut ()).unwrap();
	assert_eq!(program.invoke_index("main", 0, vec![], &mut ()).unwrap().unwrap(), RuntimeValue::I32(50));
	assert_eq!(program.invoke_index("main", 0, vec![], &mut ()).unwrap().unwrap(), RuntimeValue::I32(58));
}

// custom user error
#[derive(Debug, Clone, PartialEq)]
struct UserErrorWithCode {
	error_code: i32,
}

impl ::std::fmt::Display for UserErrorWithCode {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		write!(f, "{}", self.error_code)
	}
}

impl UserError for UserErrorWithCode {}

struct TestState {
	pub memory: Rc<MemoryInstance>,
	pub values: RefCell<Vec<i32>>,
}

fn build_env_module() -> HostModule<TestState> {
	let mut builder = HostModuleBuilder::<TestState>::new();
	builder.with_func2("add", |state: &TestState, arg: i32, unused: i32| {
		let memory_value = state.memory.get(0, 1).unwrap()[0];
		let fn_argument_unused = unused as u8;
		let fn_argument = arg as u8;
		assert_eq!(fn_argument_unused, 0);

		let sum = memory_value + fn_argument;
		state.memory.set(0, &vec![sum]).unwrap();
		state.values.borrow_mut().push(sum as i32);
		Ok(Some(sum as i32))
	});
	builder.with_func2("sub", |state: &TestState, arg: i32, unused: i32| {
		let memory_value = state.memory.get(0, 1).unwrap()[0];
		let fn_argument_unused = unused as u8;
		let fn_argument = arg as u8;
		assert_eq!(fn_argument_unused, 0);

		let diff = memory_value - fn_argument;
		state.memory.set(0, &vec![diff]).unwrap();
		state.values.borrow_mut().push(diff as i32);
		Ok(Some(diff as i32))
	});
	builder.with_func2("err", |_: &TestState, _unused1: i32, _unused2: i32| -> Result<Option<i32>, Error> {
		Err(Error::User(Box::new(UserErrorWithCode { error_code: 777 })))
	});
	builder.insert_memory("memory", Rc::new(MemoryInstance::new(&MemoryType::new(256, None)).unwrap()));
	builder.build()
}

#[test]
fn native_env_function() {
	let mut program = program_with_default_env();
	let env_host_module = build_env_module();
	let env_memory = env_host_module.export_by_name("memory").unwrap().as_memory().unwrap();
	program.add_host_module("env", env_host_module);

	let mut state = TestState {
		memory: env_memory,
		values: RefCell::new(Vec::new()),
	};
	{
		let module = module()
			.with_import(ImportEntry::new("env".into(), "add".into(), External::Function(0)))
			.with_import(ImportEntry::new("env".into(), "sub".into(), External::Function(0)))
			.function()
				.signature().param().i32().param().i32().return_type().i32().build()
				.body().with_opcodes(Opcodes::new(vec![
					Opcode::GetLocal(0),
					Opcode::GetLocal(1),
					Opcode::Call(0),
					Opcode::End,
				])).build()
				.build()
			.function()
				.signature().param().i32().param().i32().return_type().i32().build()
				.body().with_opcodes(Opcodes::new(vec![
					Opcode::GetLocal(0),
					Opcode::GetLocal(1),
					Opcode::Call(1),
					Opcode::End,
				])).build()
				.build()
			.build();

		// load module
		program.add_module("main", module, &mut state).unwrap();
		{
			assert_eq!(
				program.invoke_index("main", 2, vec![RuntimeValue::I32(7), RuntimeValue::I32(0)], &mut state)
					.unwrap()
					.unwrap(),
				RuntimeValue::I32(7)
			);
			assert_eq!(
				program.invoke_index("main", 2, vec![RuntimeValue::I32(50), RuntimeValue::I32(0)], &mut state)
					.unwrap()
					.unwrap(),
				RuntimeValue::I32(57)
			);
			assert_eq!(
				program.invoke_index("main", 3, vec![RuntimeValue::I32(15), RuntimeValue::I32(0)], &mut state)
					.unwrap()
					.unwrap(),
				RuntimeValue::I32(42)
			);
		}
	}

	assert_eq!(state.memory.get(0, 1).unwrap()[0], 42);
	assert_eq!(&*state.values.borrow(), &[7, 57, 42]);
}

#[test]
fn native_env_global() {
	struct State;

	let module_constructor = |host_module: HostModule<State>| {
		let mut program = ProgramInstance::new();
		program.add_host_module("env", host_module);

		let module = module()
			.with_import(ImportEntry::new("env".into(), "ext_global".into(), External::Global(GlobalType::new(ValueType::I32, false))))
			.function()
				.signature().return_type().i32().build()
				.body().with_opcodes(Opcodes::new(vec![
					Opcode::GetGlobal(0),
					Opcode::End,
				])).build()
				.build()
			.build();
		program.add_module("main", module, &State)?;
		program.invoke_index("main", 0, vec![], &State)
	};

	// try to add module, exporting non-existant env' variable => error
	{
		let host_module_builder = HostModuleBuilder::<State>::new();
		assert!(module_constructor(host_module_builder.build()).is_err());
	}

	// now add simple variable natively => ok
	{
		let mut host_module_builder = HostModuleBuilder::<State>::new();
		host_module_builder.insert_global("ext_global", Rc::new(GlobalInstance::new(RuntimeValue::I32(777), false)));
		assert_eq!(module_constructor(host_module_builder.build()).unwrap().unwrap(), RuntimeValue::I32(777));
	}
}

#[test]
fn native_custom_error() {
	let mut program = program_with_default_env();
	let env_host_module = build_env_module();
	let env_memory = env_host_module.export_by_name("memory").unwrap().as_memory().unwrap();
	program.add_host_module("env", env_host_module);

	let mut state = TestState {
		memory: env_memory,
		values: RefCell::new(Vec::new()),
	};

	let module = module()
		.with_import(ImportEntry::new("env".into(), "err".into(), External::Function(0)))
		.function()
			.signature().param().i32().param().i32().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::GetLocal(0),
				Opcode::GetLocal(1),
				Opcode::Call(0),
				Opcode::End,
			])).build()
			.build()
		.build();

	program.add_module("main", module, &mut state).unwrap();
	let user_error = match program.invoke_index(
		"main",
		0,
		vec![RuntimeValue::I32(7), RuntimeValue::I32(0)],
		&mut state
	) {
		Err(Error::User(user_error)) => user_error,
		result => panic!("Unexpected result {:?}", result),
	};
	assert_eq!(user_error.downcast_ref::<UserErrorWithCode>().unwrap(), &UserErrorWithCode { error_code: 777 });
}

#[test]
fn native_ref_state() {
	// Some external state that we want to use from our host module.
	type ExtState = i32;
	let mut ext_state = 0;

	// This structure holds state for our host module.
	struct HostState<'a> {
		ext_state: RefCell<&'a mut ExtState>,
	}

	let main_module = module()
		.function().signature().param().i32().build().body().build().build()
		.with_import(ImportEntry::new("env".into(), "inc".into(), External::Function(0)))
		.function()
			.signature().param().i32().param().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				// inc(arg_0);
				// inc(arg_1);
				Opcode::GetLocal(0),
				Opcode::Call(0),
				Opcode::GetLocal(1),
				Opcode::Call(0),
				Opcode::End,
			])).build()
			.build()
		.build();

	{
		let host_module = {
			let mut host_module_builder = HostModuleBuilder::<HostState>::new();
			host_module_builder.with_func1(
				"inc",
				|state: &HostState, val: i32| -> Result<Option<()>, Error> {
					let mut ext_state = state.ext_state.borrow_mut();
					// TODO: fix this
					fn inc(acc: &mut i32, val: i32) {
						*acc += val;
					}
					inc(&mut *ext_state, val);
					Ok(None)
				},
			);
			host_module_builder.build()
		};

		let mut host_state = HostState {
			ext_state: RefCell::new(&mut ext_state),
		};

		let instance = ModuleInstance::instantiate(&main_module)
			.with_import("env", &host_module)
			.assert_no_start()
			.expect("Instantiate module successfully");
		instance
			.invoke_index(
				1,
				vec![RuntimeValue::I32(7), RuntimeValue::I32(2)],
				&mut host_state,
			)
			.expect("Invoke internal func successfully");
	};

	assert_eq!(ext_state, 7 + 2);
}

#[test]
fn memory_import_limits_initial() {
	let core_module = module()
		.memory().with_min(10).build()
		.with_export(ExportEntry::new("memory".into(), Internal::Memory(0)))
		.build();

	let mut program = ProgramInstance::new();
	program.add_module("core", core_module, &mut ()).unwrap();

	let test_cases = vec![
		(9, false),
		(10, false),
		(11, true),
	];

	for test_case in test_cases {
		let (import_initial, is_error) = test_case;
		let client_module = module()
			.with_import(ImportEntry::new("core".into(), "memory".into(), External::Memory(MemoryType::new(import_initial, None))))
			.build();
		match program.add_module("client", client_module, &mut ()).map(|_| ()) {
			Ok(_) if !is_error => (),
			Err(Error::Initialization(_)) if is_error => (),
			x @ _ => panic!("unexpected result for test_case {:?}: {:?}", test_case, x),
		}
	}
}

#[test]
fn memory_import_limits_maximum() {
	#[derive(Debug, Clone, Copy, PartialEq)]
	enum MaximumError { ValueMismatch, Ok };

	let test_cases = vec![
		(Some(100), None, MaximumError::Ok),
		(Some(100), Some(98), MaximumError::ValueMismatch),
		(Some(100), Some(100), MaximumError::Ok),
		(Some(100), Some(101), MaximumError::Ok),
		(None, None, MaximumError::Ok),
	];

	let mut program = ProgramInstance::new();
	for test_case in test_cases {
		let (core_maximum, client_maximum, expected_err) = test_case;
		let core_module = module()
			.memory().with_min(10).with_max(core_maximum).build()
			.with_export(ExportEntry::new("memory".into(), Internal::Memory(0)))
			.build();
		let client_module = module()
			.with_import(ImportEntry::new("core".into(), "memory".into(), External::Memory(MemoryType::new(10, client_maximum))))
			.build();

		program.add_module("core", core_module, &mut ()).unwrap();
		match program.add_module("client", client_module, &mut ()).map(|_| ()) {
			Err(Error::Initialization(actual_err)) => match expected_err {
				MaximumError::ValueMismatch
					if actual_err == format!("trying to import with limits l1.max={:?} and l2.max={:?}", core_maximum, client_maximum) => (),
				_ => panic!("unexpected validation error for test_case {:?}: {}", test_case, actual_err),
			},
			Ok(_) if expected_err == MaximumError::Ok => (),
			x @ _ => panic!("unexpected result for test_case {:?}: {:?}", test_case, x),
		}
	}
}

#[test]
fn table_import_limits_initial() {
	let core_module = module()
		.table().with_min(10).build()
		.with_export(ExportEntry::new("table".into(), Internal::Table(0)))
		.build();

	let mut program = ProgramInstance::new();
	program.add_module("core", core_module, &mut ()).unwrap();

	let test_cases = vec![
		(9, false),
		(10, false),
		(11, true),
	];

	for test_case in test_cases {
		let (import_initial, is_error) = test_case;
		let client_module = module()
			.with_import(ImportEntry::new("core".into(), "table".into(), External::Table(TableType::new(import_initial, None))))
			.build();
		match program.add_module("client", client_module, &mut ()).map(|_| ()) {
			Ok(_) if !is_error => (),
			Err(Error::Initialization(ref actual_error))
				if is_error && actual_error == &format!("trying to import with limits l1.initial=10 and l2.initial={}", import_initial) => (),
			x @ _ => panic!("unexpected result for test_case {:?}: {:?}", test_case, x),
		}
	}
}

#[test]
fn table_import_limits_maximum() {
	#[derive(Debug, Clone, Copy, PartialEq)]
	enum MaximumError { ValueMismatch, Ok };

	let test_cases = vec![
		(Some(100), None, MaximumError::Ok),
		(Some(100), Some(98), MaximumError::ValueMismatch),
		(Some(100), Some(100), MaximumError::Ok),
		(Some(100), Some(101), MaximumError::Ok),
		(None, None, MaximumError::Ok),
	];

	let mut program = ProgramInstance::new();
	for test_case in test_cases {
		let (core_maximum, client_maximum, expected_err) = test_case;
		let core_module = module()
			.table().with_min(10).with_max(core_maximum).build()
			.with_export(ExportEntry::new("table".into(), Internal::Table(0)))
			.build();
		let client_module = module()
			.with_import(ImportEntry::new("core".into(), "table".into(), External::Table(TableType::new(10, client_maximum))))
			.build();

		program.add_module("core", core_module, &mut ()).unwrap();
		match program.add_module("client", client_module, &mut ()).map(|_| ()) {
			Err(Error::Initialization(actual_err)) => match expected_err {
				MaximumError::ValueMismatch
					if actual_err == format!("trying to import with limits l1.max={:?} and l2.max={:?}", core_maximum, client_maximum) => (),
				_ => panic!("unexpected validation error for test_case {:?}: {}", test_case, actual_err),
			},
			Ok(_) if expected_err == MaximumError::Ok => (),
			x @ _ => panic!("unexpected result for test_case {:?}: {:?}", test_case, x),
		}
	}
}
