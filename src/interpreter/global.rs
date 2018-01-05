use std::rc::Rc;
use std::cell::Cell;
use elements::{ValueType, GlobalType};
use interpreter::value::RuntimeValue;
use interpreter::Error;

#[derive(Clone, Debug)]
pub struct GlobalRef(Rc<GlobalInstance>);

impl ::std::ops::Deref for GlobalRef {
	type Target = GlobalInstance;
	fn deref(&self) -> &GlobalInstance {
		&self.0
	}
}

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

	pub fn alloc(global_type: &GlobalType, val: RuntimeValue) -> GlobalRef {
		let global = GlobalInstance::new(val, global_type.is_mutable());
		GlobalRef(Rc::new(global))
	}

	pub fn set(&self, val: RuntimeValue) -> Result<(), Error> {
		assert!(self.mutable, "Attempt to change an immutable variable");
		assert!(
			self.value_type() == val.value_type(),
			"Attempt to change variable type"
		);
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
