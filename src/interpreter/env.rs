use std::sync::Arc;
use builder::module;
use elements::{Module, FunctionType};
use interpreter::Error;
use interpreter::module::{ModuleInstanceInterface, ItemIndex, CallerContext};
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::variable::VariableInstance;

pub struct EnvModuleInstance {
	module: Module,
}

impl EnvModuleInstance {
	pub fn new(module: Module) -> Result<Self, Error> {
		Ok(EnvModuleInstance {
			module: module,
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

	fn memory(&self, _index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		unimplemented!()
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
		.memory().build() // TODO: limits
		.build();
	EnvModuleInstance::new(module)
}
