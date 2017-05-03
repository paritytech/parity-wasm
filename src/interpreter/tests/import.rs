use builder::module;
use elements::{ExportEntry, Internal, ImportEntry, External, Opcodes, Opcode};
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

	let program = ProgramInstance::new();
	let external_module = program.add_module("external_module", module1).unwrap();
	let main_module = program.add_module("main", module2).unwrap();

	assert_eq!(external_module.execute(0, vec![]).unwrap().unwrap(), RuntimeValue::I32(3));
	assert_eq!(main_module.execute(1, vec![]).unwrap().unwrap(), RuntimeValue::I32(10));
}
