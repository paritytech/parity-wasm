mod basics;
mod wabt;
mod wasm;

mod utils {
	use elements::{Internal, ExportEntry, InitExpr, Opcode, ValueType, GlobalType, GlobalEntry};
	use interpreter::ProgramInstance;
	use builder::module;

	pub fn program_with_default_env() -> ProgramInstance {
		let program = ProgramInstance::new();
		let env_module = module()
			.memory()
				.with_min(256) // 256 pages. 256 * 64K = 16MB
				.build()
				.with_export(ExportEntry::new("memory".into(), Internal::Memory(0)))
			.table()
				.with_min(64)
				.build()
				.with_export(ExportEntry::new("table".into(), Internal::Table(0)))
			.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const(0), Opcode::End])))
				.with_export(ExportEntry::new("tableBase".into(), Internal::Global(0)))
			.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const(0), Opcode::End])))
				.with_export(ExportEntry::new("memoryBase".into(), Internal::Global(1)))
			.build();
		program.add_module("env", env_module, None).unwrap();
		program
	}
}
