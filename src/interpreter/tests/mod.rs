// mod basics;
// mod wabt;
// mod wasm;

// mod utils {
// 	use elements::{MemoryType, TableType};
// 	use interpreter::{ProgramInstance, HostModuleBuilder, MemoryInstance, TableInstance, GlobalInstance, RuntimeValue};
// 	use std::rc::Rc;

// 	pub fn program_with_default_env<St: 'static>() -> ProgramInstance<St> {
// 		let mut program = ProgramInstance::<St>::new();

// 		let mut builder = HostModuleBuilder::<St>::new();
// 		builder.insert_memory("memory", Rc::new(MemoryInstance::new(&MemoryType::new(256, None)).unwrap()));
// 		builder.insert_table("table", Rc::new(TableInstance::new(&TableType::new(64, None)).unwrap()));
// 		builder.insert_global("tableBase", Rc::new(GlobalInstance::new(RuntimeValue::I32(0), false)));
// 		builder.insert_global("memoryBase", Rc::new(GlobalInstance::new(RuntimeValue::I32(0), false)));
// 		let env_host_module = builder.build();

// 		program.add_host_module("env", env_host_module);
// 		program
// 	}
// }
