use super::validate_module;
use builder::module;
use elements::{BlockType, ExportEntry, External, FunctionType, GlobalEntry, GlobalType,
				ImportEntry, InitExpr, Internal, MemoryType, Opcode, Opcodes, TableType,
				ValueType};

#[test]
fn empty_is_valid() {
	let module = module().build();
	assert!(validate_module(&module).is_ok());
}

#[test]
fn mem_limits() {
	// min > max
	let m = module()
		.memory()
			.with_min(10)
			.with_max(Some(9))
			.build()
		.build();
	assert!(validate_module(&m).is_err());

	// min = max
	let m = module()
		.memory()
			.with_min(10)
			.with_max(Some(10))
			.build()
		.build();
	assert!(validate_module(&m).is_ok());

	// mem is always valid without max
	let m = module()
		.memory()
			.with_min(10)
			.build()
		.build();
	assert!(validate_module(&m).is_ok());
}

#[test]
fn table_limits() {
	// min > max
	let m = module()
		.table()
			.with_min(10)
			.with_max(Some(9))
			.build()
		.build();
	assert!(validate_module(&m).is_err());

	// min = max
	let m = module()
		.table()
			.with_min(10)
			.with_max(Some(10))
			.build()
		.build();
	assert!(validate_module(&m).is_ok());

	// table is always valid without max
	let m = module()
		.table()
			.with_min(10)
			.build()
		.build();
	assert!(validate_module(&m).is_ok());
}

#[test]
fn global_init_const() {
	let m = module()
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I32, true),
				InitExpr::new(
					vec![Opcode::I32Const(42), Opcode::End]
				)
			)
		)
		.build();
	assert!(validate_module(&m).is_ok());

	// init expr type differs from declared global type
	let m = module()
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I64, true),
				InitExpr::new(vec![Opcode::I32Const(42), Opcode::End])
			)
		)
		.build();
	assert!(validate_module(&m).is_err());
}

#[test]
fn global_init_global() {
	let m = module()
		.with_import(
			ImportEntry::new(
				"env".into(),
				"ext_global".into(),
				External::Global(GlobalType::new(ValueType::I32, false))
			)
		)
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I32, true),
				InitExpr::new(vec![Opcode::GetGlobal(0), Opcode::End])
			)
		)
		.build();
	assert!(validate_module(&m).is_ok());

	// get_global can reference only previously defined globals
	let m = module()
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I32, true),
				InitExpr::new(vec![Opcode::GetGlobal(0), Opcode::End])
			)
		)
		.build();
	assert!(validate_module(&m).is_err());

	// get_global can reference only const globals
	let m = module()
		.with_import(
			ImportEntry::new(
				"env".into(),
				"ext_global".into(),
				External::Global(GlobalType::new(ValueType::I32, true))
			)
		)
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I32, true),
				InitExpr::new(vec![Opcode::GetGlobal(0), Opcode::End])
			)
		)
		.build();
	assert!(validate_module(&m).is_err());

	// get_global in init_expr can only refer to imported globals.
	let m = module()
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I32, false),
				InitExpr::new(vec![Opcode::I32Const(0), Opcode::End])
			)
		)
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I32, true),
				InitExpr::new(vec![Opcode::GetGlobal(0), Opcode::End])
			)
		)
		.build();
	assert!(validate_module(&m).is_err());
}

#[test]
fn global_init_misc() {
	// without delimiting End opcode
	let m = module()
		.with_global(
			GlobalEntry::new(
				GlobalType::new(ValueType::I32, true),
				InitExpr::new(vec![Opcode::I32Const(42)])
			)
		)
		.build();
	assert!(validate_module(&m).is_err());

	// empty init expr
	let m = module()
			.with_global(
				GlobalEntry::new(
					GlobalType::new(ValueType::I32, true),
					InitExpr::new(vec![Opcode::End])
				)
			)
		.build();
	assert!(validate_module(&m).is_err());

	// not an constant opcode used
	let m = module()
			.with_global(
				GlobalEntry::new(
					GlobalType::new(ValueType::I32, true),
					InitExpr::new(vec![Opcode::Unreachable, Opcode::End])
				)
			)
		.build();
	assert!(validate_module(&m).is_err());
}

// #[test]
// fn if_else_with_return_type_validation() {
// 	let module_instance = ModuleInstance::new(Weak::default(), "test".into(), module().build()).unwrap();
// 	let mut context = FunctionValidationContext::new(&module_instance, None, &[], 1024, 1024, FunctionSignature::Module(&FunctionType::default()));

// 	Validator::validate_function(&mut context, BlockType::NoResult, &[
// 		Opcode::I32Const(1),
// 		Opcode::If(BlockType::NoResult),
// 			Opcode::I32Const(1),
// 			Opcode::If(BlockType::Value(ValueType::I32)),
// 				Opcode::I32Const(1),
// 			Opcode::Else,
// 				Opcode::I32Const(2),
// 			Opcode::End,
// 		Opcode::Drop,
// 		Opcode::End,
// 		Opcode::End,
// 	]).unwrap();
// }
