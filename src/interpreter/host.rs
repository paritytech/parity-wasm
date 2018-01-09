use elements::FunctionType;
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
}
