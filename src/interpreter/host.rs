use std::any::TypeId;
use elements::FunctionType;
use interpreter::value::RuntimeValue;
use interpreter::Error;

/// Custom user error.
pub trait HostError: 'static + ::std::fmt::Display + ::std::fmt::Debug {
	#[doc(hidden)]
	fn __private_get_type_id__(&self) -> TypeId {
		TypeId::of::<Self>()
	}
}

impl HostError {
	/// Attempt to downcast this `HostError` to a concrete type by reference.
	pub fn downcast_ref<T: HostError>(&self) -> Option<&T> {
		if self.__private_get_type_id__() == TypeId::of::<T>() {
			unsafe { Some(&*(self as *const HostError as *const T)) }
		} else {
			None
		}
	}

	/// Attempt to downcast this `HostError` to a concrete type by mutable
	/// reference.
	pub fn downcast_mut<T: HostError>(&mut self) -> Option<&mut T> {
		if self.__private_get_type_id__() == TypeId::of::<T>() {
			unsafe { Some(&mut *(self as *mut HostError as *mut T)) }
		} else {
			None
		}
	}
}

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
