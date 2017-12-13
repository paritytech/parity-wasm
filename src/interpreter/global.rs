use std::cell::Cell;
use interpreter::value::RuntimeValue;
use interpreter::Error;

#[derive(Debug)]
pub struct GlobalInstance {
	val: Cell<RuntimeValue>,
	mutable: bool,
}

impl GlobalInstance {
	pub fn new(val: RuntimeValue, mutable: bool) -> GlobalInstance {
		GlobalInstance {
			val: Cell::new(val),
			mutable,
		}
	}

	pub fn set(&self, val: RuntimeValue) -> Result<(), Error> {
		if !self.mutable {
			return Err(Error::Validation("Can't set immutable global".into()));
		}
		self.val.set(val);
		Ok(())
	}

	pub fn get(&self) -> RuntimeValue {
		self.val.get()
	}
}
