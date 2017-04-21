use std::sync::Arc;
use parking_lot::RwLock;
use elements::{GlobalType, ValueType};
use interpreter::Error;
use interpreter::value::RuntimeValue;

/// Variable type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariableType {
	/// Any func value.
	AnyFunc,
	/// i32 value.
	I32,
	/// i64 value.
	I64,
	/// f32 value.
	F32,
	/// f64 value.
	F64,
}

/// Variable instance.
pub struct VariableInstance {
	/// Is mutable?
	is_mutable: bool,
	/// Variable type.
	variable_type: VariableType,
	/// Global value.
	value: RwLock<RuntimeValue>,
}

impl VariableInstance {
	pub fn new(global_type: &GlobalType, value: RuntimeValue) -> Result<Arc<Self>, Error> {
		let variable_type: VariableType = global_type.content_type().into();
		if !value.is_null() && value.variable_type() != Some(variable_type) {
			return Err(Error::Variable(format!("trying to initialize variable of type {:?} with value of type {:?}", variable_type, value.variable_type())));
		}

		Ok(Arc::new(VariableInstance {
			is_mutable: global_type.is_mutable(),
			variable_type: variable_type,
			value: RwLock::new(value),
		}))
	}

	pub fn get(&self) -> RuntimeValue {
		self.value.read().clone()
	}

	pub fn set(&self, value: RuntimeValue) -> Result<(), Error> {
		if !self.is_mutable {
			return Err(Error::Variable("trying to update immutable variable".into()));
		}
		if value.variable_type() != Some(self.variable_type) {
			return Err(Error::Variable(format!("trying to update variable of type {:?} with value of type {:?}", self.variable_type, value.variable_type())));
		}

		*self.value.write() = value;
		Ok(())
	}
}

impl From<ValueType> for VariableType {
	fn from(vt: ValueType) -> VariableType {
		match vt {
			ValueType::I32 => VariableType::I32,
			ValueType::I64 => VariableType::I64,
			ValueType::F32 => VariableType::F32,
			ValueType::F64 => VariableType::F64,
		}
	}
}
