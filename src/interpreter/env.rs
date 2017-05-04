use std::sync::Arc;
use builder::module;
use elements::{Module, FunctionType, ExportEntry, Internal, MemoryType};
use interpreter::Error;
use interpreter::module::{ModuleInstanceInterface, ItemIndex, CallerContext};
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::variable::VariableInstance;

const MEMORY_LIMIT_MIN: u32 = 1;

pub struct EnvModuleInstance {
	module: Module,
	memory: Arc<MemoryInstance>,
}

impl EnvModuleInstance {
	pub fn new(module: Module) -> Result<Self, Error> {
		Ok(EnvModuleInstance {
			module: module,
			memory: MemoryInstance::new(&MemoryType::new(MEMORY_LIMIT_MIN, None))?,
		})
	}
}

impl ModuleInstanceInterface for EnvModuleInstance {
	fn execute_main(&self, _args: Vec<RuntimeValue>) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn execute(&self, _index: u32, _args: Vec<RuntimeValue>) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn module(&self) -> &Module {
		&self.module
	}

	fn table(&self, _index: ItemIndex) -> Result<Arc<TableInstance>, Error> {
		unimplemented!()
	}

	fn memory(&self, index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		match &index {
			&ItemIndex::Internal(0) => Ok(self.memory.clone()),
			_ => Err(Error::Env(format!("trying to get memory with index {:?}", index))),
		}
	}

	fn global(&self, _index: ItemIndex) -> Result<Arc<VariableInstance>, Error> {
		unimplemented!()
	}

	fn call_function(&self, _outer: CallerContext, _index: ItemIndex) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn call_function_indirect(&self, _outer: CallerContext, _table_index: ItemIndex, _type_index: u32, _func_index: u32) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn call_internal_function(&self, _outer: CallerContext, _index: u32, _function_type: Option<&FunctionType>) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}
}

pub fn env_module() -> Result<EnvModuleInstance, Error> {
	let module = module()
		.memory()
			.with_min(MEMORY_LIMIT_MIN)
			.build()
		.with_export(ExportEntry::new("memory".into(), Internal::Memory(0)))
		.build();
	EnvModuleInstance::new(module)
}
