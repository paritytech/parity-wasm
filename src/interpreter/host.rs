use elements::FunctionType;
use interpreter::global::GlobalInstance;
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::Error;

pub type HostFuncIndex = u32;

pub trait Externals {
	fn invoke_index(
		&mut self,
		index: HostFuncIndex,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error>;

	fn check_signature(&self, index: HostFuncIndex, signature: &FunctionType) -> bool;

	fn memory_by_index(&self, index: usize) -> &MemoryInstance;
	fn table_by_index(&self, index: usize) -> &TableInstance;
	fn global_by_index(&self, index: usize) -> &GlobalInstance;
}

pub struct EmptyExternals;

impl Externals for EmptyExternals {
	fn invoke_index(
		&mut self,
		_index: HostFuncIndex,
		_args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		panic!("called invoke_index on EmptyExternals")
	}

	fn check_signature(&self, _index: HostFuncIndex, _signature: &FunctionType) -> bool {
		panic!("called check_signature on EmptyExternals")
	}

	fn memory_by_index(&self, _index: usize) -> &MemoryInstance {
		panic!("called memory_by_index on EmptyExternals")
	}

	fn table_by_index(&self, _index: usize) -> &TableInstance {
		panic!("called table_by_index on EmptyExternals")
	}

	fn global_by_index(&self, _index: usize) -> &GlobalInstance {
		panic!("called global_by_index on EmptyExternals")
	}
}
