///! Basic tests for instructions/constructions, missing in wabt tests

use builder::module;
use elements::{ExportEntry, Internal, ImportEntry, External, GlobalEntry, GlobalType,
	InitExpr, ValueType, Opcodes, Opcode};
use interpreter::Error;
use interpreter::module::{ModuleInstanceInterface, CallerContext};
use interpreter::program::ProgramInstance;
use interpreter::value::RuntimeValue;

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
	let external_module = program.add_module("external_module", module1).unwrap();
	let main_module = program.add_module("main", module2).unwrap();

	assert_eq!(external_module.execute_index(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(3));
	assert_eq!(main_module.execute_index(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(10));
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
	let _side_module_instance = program.add_module("side_module", side_module).unwrap();
	let module_instance = program.add_module("main", module).unwrap();

	assert!(module_instance.execute_index(1, vec![]).is_err());	
}

#[test]
fn global_get_set() {
	let module = module()
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(42)])))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const(777)])))
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
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::GetGlobal(1),
				Opcode::I32Const(8),
				Opcode::I32Add,
				Opcode::SetGlobal(1),
				Opcode::GetGlobal(1),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(8),
				Opcode::SetGlobal(0),
				Opcode::GetGlobal(0),
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new().unwrap();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute_index(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(50));
	assert_eq!(module.execute_index(1, vec![]).unwrap_err(), Error::Variable("trying to update immutable variable".into()));
	assert_eq!(module.execute_index(2, vec![]).unwrap_err(), Error::Variable("trying to update variable of type I32 with value of type Some(I64)".into()));
}

#[test]
fn with_user_functions() {
	use interpreter::{UserFunction, UserFunctions};

	let module = module()
		.with_import(ImportEntry::new("env".into(), "custom_alloc".into(), External::Function(0)))
		.with_import(ImportEntry::new("env".into(), "custom_increment".into(), External::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(32),
				Opcode::Call(0),
				Opcode::End,
			])).build()
			.build()
		.build();

	let mut top = 0i32;
	let mut user_functions = UserFunctions::new();
	user_functions.insert(
		"custom_alloc".to_owned(), 
		UserFunction {
			params: vec![ValueType::I32],
			result: Some(ValueType::I32),
			closure: Box::new(move |context: CallerContext| {
				let prev = top;
				top = top + context.value_stack.pop_as::<i32>()?;
				Ok(Some(prev.into()))
			}),
		}
	);

	let mut rolling = 9999i32;
	user_functions.insert(
		"custom_increment".to_owned(), 
		UserFunction {
			params: vec![ValueType::I32],
			result: Some(ValueType::I32),
			closure: Box::new(move |_: CallerContext| {
				rolling = rolling + 1;
				Ok(Some(rolling.into()))
			}),
		}
	);	

	let program = ProgramInstance::with_functions(user_functions).unwrap();
	let module_instance = program.add_module("main", module).unwrap();	

	// internal function using first import
	assert_eq!(module_instance.execute_index(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));	
	assert_eq!(module_instance.execute_index(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(32));	
	assert_eq!(module_instance.execute_index(2, vec![]).unwrap().unwrap(), RuntimeValue::I32(64));	
	
	// second import
	assert_eq!(module_instance.execute_index(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(10000));	
	assert_eq!(module_instance.execute_index(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(10001));	
}

#[test]
fn with_user_functions_extended() {
	use interpreter::{UserFunction, UserFunctions, UserFunctionInterface};

	struct UserMAlloc {
		top: i32,
	}

	impl UserFunctionInterface for UserMAlloc {
		fn call(&mut self, context: CallerContext) -> Result<Option<RuntimeValue>, Error> {
			let prev = self.top;
			self.top += context.value_stack.pop_as::<i32>()?;
			Ok(Some(prev.into()))
		}
	}

	let module = module()
		.with_import(ImportEntry::new("env".into(), "_malloc".into(), External::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(32),
				Opcode::Call(0),
				Opcode::End,
			])).build()
			.build()
		.build();

	let mut user_functions = UserFunctions::new();
	user_functions.insert(
		"_malloc".to_owned(), 
		UserFunction {
			params: vec![ValueType::I32],
			result: Some(ValueType::I32),
			closure: Box::new(UserMAlloc { top: 0 }),
		}
	);

	let program = ProgramInstance::with_functions(user_functions).unwrap();
	let module_instance = program.add_module("main", module).unwrap();	

	// internal function using first import
	assert_eq!(module_instance.execute_index(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(0));	
	assert_eq!(module_instance.execute_index(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(32));	
	assert_eq!(module_instance.execute_index(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(64));	
}