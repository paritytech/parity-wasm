///! Basic tests for instructions/constructions, missing in wabt tests

use std::sync::{Arc, Weak};
use std::collections::HashMap;
use builder::module;
use elements::{ExportEntry, Internal, ImportEntry, External, GlobalEntry, GlobalType,
	InitExpr, ValueType, BlockType, Opcodes, Opcode, FunctionType};
use interpreter::Error;
use interpreter::env_native::{env_native_module, UserDefinedElements, UserFunctionExecutor, UserFunctionDescriptor};
use interpreter::memory::MemoryInstance;
use interpreter::module::{ModuleInstance, ModuleInstanceInterface, CallerContext, ItemIndex, ExecutionParams, ExportEntryType, FunctionSignature};
use interpreter::program::ProgramInstance;
use interpreter::validator::{FunctionValidationContext, Validator};
use interpreter::value::{RuntimeValue, TryInto};
use interpreter::variable::{VariableInstance, ExternalVariableValue, VariableType};

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

	let program = ProgramInstance::new().unwrap();
	let external_module = program.add_module("external_module", module1, None).unwrap();
	let main_module = program.add_module("main", module2, None).unwrap();

	assert_eq!(external_module.execute_index(0, vec![].into()).unwrap().unwrap(), RuntimeValue::I32(3));
	assert_eq!(main_module.execute_index(1, vec![].into()).unwrap().unwrap(), RuntimeValue::I32(10));
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

	let program = ProgramInstance::new().unwrap();
	let _side_module_instance = program.add_module("side_module", side_module, None).unwrap();
	assert!(program.add_module("main", module, None).is_err());	
}

#[test]
fn global_get_set() {
	let module = module()
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(42)])))
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

	let program = ProgramInstance::new().unwrap();
	let module = program.add_module("main", module, None).unwrap();
	assert_eq!(module.execute_index(0, vec![].into()).unwrap().unwrap(), RuntimeValue::I32(50));
	assert_eq!(module.execute_index(0, vec![].into()).unwrap().unwrap(), RuntimeValue::I32(58));
}

const SIGNATURE_I32_I32: &'static [ValueType] = &[ValueType::I32, ValueType::I32];

const SIGNATURES: &'static [UserFunctionDescriptor] = &[
	UserFunctionDescriptor::Static(
		"add",
		SIGNATURE_I32_I32,
		Some(ValueType::I32),
	),
	UserFunctionDescriptor::Static(
		"sub",
		SIGNATURE_I32_I32,
		Some(ValueType::I32),
	),
];

const NO_SIGNATURES: &'static [UserFunctionDescriptor] = &[];

// 'external' variable
struct MeasuredVariable {
	pub val: i32,
}

impl ExternalVariableValue for MeasuredVariable {
	fn get(&self) -> RuntimeValue {
		RuntimeValue::I32(self.val)
	}

	fn set(&mut self, val: RuntimeValue) -> Result<(), Error> {
		self.val = val.try_into()?;
		Ok(())
	}
}

// user function executor
struct FunctionExecutor {
	pub memory: Arc<MemoryInstance>,
	pub values: Vec<i32>,
}

impl UserFunctionExecutor for FunctionExecutor {
	fn execute(&mut self, name: &str, context: CallerContext) -> Result<Option<RuntimeValue>, Error> {
		match name {
			"add" => {
				let memory_value = self.memory.get(0, 1).unwrap()[0];
				let fn_argument_unused = context.value_stack.pop_as::<u32>().unwrap() as u8;
				let fn_argument = context.value_stack.pop_as::<u32>().unwrap() as u8;
				assert_eq!(fn_argument_unused, 0);

				let sum = memory_value + fn_argument;
				self.memory.set(0, &vec![sum]).unwrap();
				self.values.push(sum as i32);
				Ok(Some(RuntimeValue::I32(sum as i32)))
			},
			"sub" => {
				let memory_value = self.memory.get(0, 1).unwrap()[0];
				let fn_argument_unused = context.value_stack.pop_as::<u32>().unwrap() as u8;
				let fn_argument = context.value_stack.pop_as::<u32>().unwrap() as u8;
				assert_eq!(fn_argument_unused, 0);

				let diff = memory_value - fn_argument;
				self.memory.set(0, &vec![diff]).unwrap();
				self.values.push(diff as i32);
				Ok(Some(RuntimeValue::I32(diff as i32)))
			},
			_ => Err(Error::Trap("not implemented".into())),
		}
	}
}

#[test]
fn native_env_function() {
	// create new program
	let program = ProgramInstance::new().unwrap();
	// => env module is created
	let env_instance = program.module("env").unwrap();
	// => linear memory is created
	let env_memory = env_instance.memory(ItemIndex::Internal(0)).unwrap();

	// create native env module executor
	let mut executor = FunctionExecutor {
		memory: env_memory.clone(),
		values: Vec::new(),
	};
	{
		let functions: UserDefinedElements = UserDefinedElements {
			executor: Some(&mut executor),
			globals: HashMap::new(),
			functions: ::std::borrow::Cow::from(SIGNATURES),
		};
		let native_env_instance = Arc::new(env_native_module(env_instance, functions).unwrap());
		let params = ExecutionParams::with_external("env".into(), native_env_instance);

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
		let module_instance = program.add_module("main", module, Some(&params.externals)).unwrap();

		{
			assert_eq!(module_instance.execute_index(2, params.clone().add_argument(RuntimeValue::I32(7)).add_argument(RuntimeValue::I32(0))).unwrap().unwrap(), RuntimeValue::I32(7));
			assert_eq!(module_instance.execute_index(2, params.clone().add_argument(RuntimeValue::I32(50)).add_argument(RuntimeValue::I32(0))).unwrap().unwrap(), RuntimeValue::I32(57));
			assert_eq!(module_instance.execute_index(3, params.clone().add_argument(RuntimeValue::I32(15)).add_argument(RuntimeValue::I32(0))).unwrap().unwrap(), RuntimeValue::I32(42));
		}
	}

	assert_eq!(executor.memory.get(0, 1).unwrap()[0], 42);
	assert_eq!(executor.values, vec![7, 57, 42]);
}

#[test]
fn native_env_global() {
	// module constructor
	let module_constructor = |elements| {
		let program = ProgramInstance::new().unwrap();
		let env_instance = program.module("env").unwrap();
		let native_env_instance = Arc::new(env_native_module(env_instance.clone(), elements).unwrap());
		let params = ExecutionParams::with_external("env".into(), native_env_instance);

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
		program.add_module("main", module, Some(&params.externals))?
			.execute_index(0, params.clone())
	};

	// try to add module, exporting non-existant env' variable => error
	{
		assert!(module_constructor(UserDefinedElements {
			executor: None,
			globals: HashMap::new(),
			functions: ::std::borrow::Cow::from(NO_SIGNATURES),
		}).is_err());
	}

	// now add simple variable natively => ok
	{
		assert_eq!(module_constructor(UserDefinedElements {
			executor: None,
			globals: vec![("ext_global".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(777)).unwrap()))].into_iter().collect(),
			functions: ::std::borrow::Cow::from(NO_SIGNATURES),
		}).unwrap().unwrap(), RuntimeValue::I32(777));
	}

	// now add 'getter+setter' variable natively => ok
	{
		assert_eq!(module_constructor(UserDefinedElements {
			executor: None,
			globals: vec![("ext_global".into(), Arc::new(VariableInstance::new_external_global(false, VariableType::I32, Box::new(MeasuredVariable { val: 345 })).unwrap()))].into_iter().collect(),
			functions: ::std::borrow::Cow::from(NO_SIGNATURES),
		}).unwrap().unwrap(), RuntimeValue::I32(345));
	}
}

#[test]
fn import_env_mutable_global() {
	let program = ProgramInstance::new().unwrap();

	let module = module()
		.with_import(ImportEntry::new("env".into(), "STACKTOP".into(), External::Global(GlobalType::new(ValueType::I32, false))))
		.build();

	program.add_module("main", module, None).unwrap();
}

#[test]
fn env_native_export_entry_type_check() {
	let program = ProgramInstance::new().unwrap();
	let mut function_executor = FunctionExecutor {
		memory: program.module("env").unwrap().memory(ItemIndex::Internal(0)).unwrap(),
		values: Vec::new(),
	};
	let native_env_instance = Arc::new(env_native_module(program.module("env").unwrap(), UserDefinedElements {
		executor: Some(&mut function_executor),
		globals: HashMap::new(),
		functions: ::std::borrow::Cow::from(SIGNATURES),
	}).unwrap());

	assert!(native_env_instance.export_entry("add", &ExportEntryType::Function(FunctionSignature::Module(&FunctionType::new(vec![ValueType::I32, ValueType::I32], Some(ValueType::I32))))).is_ok());
	assert!(native_env_instance.export_entry("add", &ExportEntryType::Function(FunctionSignature::Module(&FunctionType::new(vec![], Some(ValueType::I32))))).is_err());
	assert!(native_env_instance.export_entry("add", &ExportEntryType::Function(FunctionSignature::Module(&FunctionType::new(vec![ValueType::I32, ValueType::I32], None)))).is_err());
	assert!(native_env_instance.export_entry("add", &ExportEntryType::Function(FunctionSignature::Module(&FunctionType::new(vec![ValueType::I32, ValueType::I32], Some(ValueType::I64))))).is_err());
}

#[test]
fn if_else_with_return_type_validation() {
	let module_instance = ModuleInstance::new(Weak::default(), "test".into(), module().build()).unwrap();
	let mut context = FunctionValidationContext::new(&module_instance, None, &[], 1024, 1024, FunctionSignature::Module(&FunctionType::default()));

	Validator::validate_function(&mut context, BlockType::NoResult, &[
		Opcode::I32Const(1),
		Opcode::If(BlockType::NoResult),
			Opcode::I32Const(1),
			Opcode::If(BlockType::Value(ValueType::I32)),
				Opcode::I32Const(1),
			Opcode::Else,
				Opcode::I32Const(2),
			Opcode::End,
		Opcode::Drop,
		Opcode::End,
		Opcode::End,
	]).unwrap();
}
