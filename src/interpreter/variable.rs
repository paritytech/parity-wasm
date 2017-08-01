use std::fmt;
use parking_lot::RwLock;
use elements::{GlobalType, ValueType, TableElementType};
use interpreter::{Error, UserError};
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

/// Externally stored variable value.
pub trait ExternalVariableValue<E: UserError> {
	/// Get variable value.
	fn get(&self) -> RuntimeValue;
	/// Set variable value.
	fn set(&mut self, value: RuntimeValue) -> Result<(), Error<E>>;
}

/// Variable instance.
#[derive(Debug)]
pub struct VariableInstance<E: UserError> {
	/// Is mutable?
	is_mutable: bool,
	/// Variable type.
	variable_type: VariableType,
	/// Global value.
	value: RwLock<VariableValue<E>>,
}

/// Enum variable value.
enum VariableValue<E: UserError> {
	/// Internal value.
	Internal(RuntimeValue),
	/// External value.
	External(Box<ExternalVariableValue<E>>),
}

impl<E> VariableInstance<E> where E: UserError {
	/// New variable instance
	pub fn new(is_mutable: bool, variable_type: VariableType, value: RuntimeValue) -> Result<Self, Error<E>> {
		// TODO: there is nothing about null value in specification + there is nothing about initializing missing table elements? => runtime check for nulls
		if !value.is_null() && value.variable_type() != Some(variable_type) {
			return Err(Error::Variable(format!("trying to initialize variable of type {:?} with value of type {:?}", variable_type, value.variable_type())));
		}

		Ok(VariableInstance {
			is_mutable: is_mutable,
			variable_type: variable_type,
			value: RwLock::new(VariableValue::Internal(value)),
		})
	}

	/// New global variable
	pub fn new_global(global_type: &GlobalType, value: RuntimeValue) -> Result<Self, Error<E>> {
		Self::new(global_type.is_mutable(), global_type.content_type().into(), value)
	}

	/// New global with externally stored value.
	pub fn new_external_global(is_mutable: bool, variable_type: VariableType, value: Box<ExternalVariableValue<E>>) -> Result<Self, Error<E>> {
		// TODO: there is nothing about null value in specification + there is nothing about initializing missing table elements? => runtime check for nulls
		let current_value = value.get();
		if !current_value.is_null() && current_value.variable_type() != Some(variable_type) {
			return Err(Error::Variable(format!("trying to initialize variable of type {:?} with value of type {:?}", variable_type, current_value.variable_type())));
		}

		Ok(VariableInstance {
			is_mutable: is_mutable,
			variable_type: variable_type,
			value: RwLock::new(VariableValue::External(value)),
		})
	}

	/// Is mutable
	pub fn is_mutable(&self) -> bool {
		self.is_mutable
	}

	/// Get variable type.
	pub fn variable_type(&self) -> VariableType {
		self.variable_type
	}

	/// Get the value of the variable instance
	pub fn get(&self) -> RuntimeValue {
		self.value.read().get()
	}

	/// Set the value of the variable instance
	pub fn set(&self, value: RuntimeValue) -> Result<(), Error<E>> {
		if !self.is_mutable {
			return Err(Error::Variable("trying to update immutable variable".into()));
		}
		if value.variable_type() != Some(self.variable_type) {
			return Err(Error::Variable(format!("trying to update variable of type {:?} with value of type {:?}", self.variable_type, value.variable_type())));
		}

		self.value.write().set(value)
	}
}

impl<E> VariableValue<E> where E: UserError {
	fn get(&self) -> RuntimeValue {
		match *self {
			VariableValue::Internal(ref value) => value.clone(),
			VariableValue::External(ref value) => value.get(),
		}
	}

	fn set(&mut self, new_value: RuntimeValue) -> Result<(), Error<E>> {
		match *self {
			VariableValue::Internal(ref mut value) => {
				*value = new_value;
				Ok(())
			},
			VariableValue::External(ref mut value) => value.set(new_value),
		}
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

impl From<TableElementType> for VariableType {
	fn from(tt: TableElementType) -> VariableType {
		match tt {
			TableElementType::AnyFunc => VariableType::AnyFunc,
		}
	}
}

impl<E> fmt::Debug for VariableValue<E> where E: UserError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			VariableValue::Internal(ref value) => write!(f, "Variable.Internal({:?})", value),
			VariableValue::External(ref value) => write!(f, "Variable.External({:?})", value.get()),
		}
	}
}
