use elements::FunctionType;
use interpreter::global::GlobalInstance;
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::Error;

pub type HostFuncIndex = u32;
pub type HostMemoryIndex = u32;
pub type HostTableIndex = u32;
pub type HostGlobalIndex = u32;

pub trait Externals {
	fn invoke_index(
		&mut self,
		index: HostFuncIndex,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error>;

	fn check_signature(&self, index: HostFuncIndex, signature: &FunctionType) -> bool;

	fn memory_by_index(&self, index: HostMemoryIndex) -> Option<&MemoryInstance>;
	fn table_by_index(&self, index: HostTableIndex) -> Option<&TableInstance>;
	fn global_by_index(&self, index: HostGlobalIndex) -> Option<&GlobalInstance>;
}

pub struct EmptyExternals;

impl Externals for EmptyExternals {
	fn invoke_index(
		&mut self,
		_index: HostFuncIndex,
		_args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		Err(Error::Trap("invoke index on empty externals".into()))
	}

	fn check_signature(&self, _index: HostFuncIndex, _signature: &FunctionType) -> bool {
		false
	}

	fn memory_by_index(&self, _index: HostMemoryIndex) -> Option<&MemoryInstance> {
		None
	}

	fn table_by_index(&self, _index: HostTableIndex) -> Option<&TableInstance> {
		None
	}

	fn global_by_index(&self, _index: HostGlobalIndex) -> Option<&GlobalInstance> {
		None
	}
}
