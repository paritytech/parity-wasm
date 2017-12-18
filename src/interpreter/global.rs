use std::cell::Cell;
use elements::ValueType;
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
		assert!(self.mutable, "Attempt to change an immutable variable");
		assert!(self.value_type() == val.value_type(), "Attempt to change variable type");
		self.val.set(val);
		Ok(())
	}

	pub fn get(&self) -> RuntimeValue {
		self.val.get()
	}

	pub fn is_mutable(&self) -> bool {
		self.mutable
	}

	pub fn value_type(&self) -> ValueType {
		self.val.get().value_type()
	}
}
