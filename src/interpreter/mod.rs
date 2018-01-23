//! WebAssembly interpreter module.

#![deprecated(since = "0.23", note = "Use wasmi crate to interpret wasm")]

use std::any::TypeId;
use std::error;
use std::fmt;
use validation;
use common;

/// Custom user error.
pub trait UserError: 'static + ::std::fmt::Display + ::std::fmt::Debug {
	#[doc(hidden)]
	fn __private_get_type_id__(&self) -> TypeId {
		TypeId::of::<Self>()
	}
}

impl UserError {
	/// Attempt to downcast this `UserError` to a concrete type by reference.
	pub fn downcast_ref<T: UserError>(&self) -> Option<&T> {
		if self.__private_get_type_id__() == TypeId::of::<T>() {
			unsafe { Some(&*(self as *const UserError as *const T)) }
		} else {
			None
		}
	}

	/// Attempt to downcast this `UserError` to a concrete type by mutable
	/// reference.
	pub fn downcast_mut<T: UserError>(&mut self) -> Option<&mut T> {
		if self.__private_get_type_id__() == TypeId::of::<T>() {
			unsafe { Some(&mut *(self as *mut UserError as *mut T)) }
		} else {
			None
		}
	}
}

/// Internal interpreter error.
#[derive(Debug)]
pub enum Error {
	/// Program-level error.
	Program(String),
	/// Validation error.
	Validation(String),
	/// Initialization error.
	Initialization(String),
	/// Function-level error.
	Function(String),
	/// Table-level error.
	Table(String),
	/// Memory-level error.
	Memory(String),
	/// Variable-level error.
	Variable(String),
	/// Global-level error.
	Global(String),
	/// Local-level error.
	Local(String),
	/// Stack-level error.
	Stack(String),
	/// Value-level error.
	Value(String),
	/// Interpreter (code) error.
	Interpreter(String),
	/// Native module error.
	Native(String),
	/// Trap.
	Trap(String),
	/// Custom user error.
	User(Box<UserError>),
}

impl Into<String> for Error {
	fn into(self) -> String {
		match self {
			Error::Program(s) => s,
			Error::Validation(s) => s,
			Error::Initialization(s) => s,
			Error::Function(s) => s,
			Error::Table(s) => s,
			Error::Memory(s) => s,
			Error::Variable(s) => s,
			Error::Global(s) => s,
			Error::Local(s) => s,
			Error::Stack(s) => s,
			Error::Interpreter(s) => s,
			Error::Value(s) => s,
			Error::Native(s) => s,
			Error::Trap(s) => format!("trap: {}", s),
			Error::User(e) => format!("user: {}", e),
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Program(ref s) => write!(f, "Program: {}", s),
			Error::Validation(ref s) => write!(f, "Validation: {}", s),
			Error::Initialization(ref s) => write!(f, "Initialization: {}", s),
			Error::Function(ref s) => write!(f, "Function: {}", s),
			Error::Table(ref s) => write!(f, "Table: {}", s),
			Error::Memory(ref s) => write!(f, "Memory: {}", s),
			Error::Variable(ref s) => write!(f, "Variable: {}", s),
			Error::Global(ref s) => write!(f, "Global: {}", s),
			Error::Local(ref s) => write!(f, "Local: {}", s),
			Error::Stack(ref s) => write!(f, "Stack: {}", s),
			Error::Interpreter(ref s) => write!(f, "Interpreter: {}", s),
			Error::Value(ref s) => write!(f, "Value: {}", s),
			Error::Native(ref s) => write!(f, "Native: {}", s),
			Error::Trap(ref s) => write!(f, "Trap: {}", s),
			Error::User(ref e) => write!(f, "User: {}", e),
		}
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		match *self {
			Error::Program(ref s) => s,
			Error::Validation(ref s) => s,
			Error::Initialization(ref s) => s,
			Error::Function(ref s) => s,
			Error::Table(ref s) => s,
			Error::Memory(ref s) => s,
			Error::Variable(ref s) => s,
			Error::Global(ref s) => s,
			Error::Local(ref s) => s,
			Error::Stack(ref s) => s,
			Error::Interpreter(ref s) => s,
			Error::Value(ref s) => s,
			Error::Native(ref s) => s,
			Error::Trap(ref s) => s,
			Error::User(_) => "User error",
		}
	}
}

impl<U> From<U> for Error where U: UserError + Sized {
	fn from(e: U) -> Self {
		Error::User(Box::new(e))
	}
}

impl From<validation::Error> for Error {
	fn from(e: validation::Error) -> Self {
		Error::Validation(e.to_string())
	}
}

impl From<common::stack::Error> for Error {
	fn from(e: common::stack::Error) -> Self {
		Error::Stack(e.to_string())
	}
}

mod validator;
mod native;
mod imports;
mod memory;
mod module;
mod program;
mod runner;
mod stack;
mod table;
mod value;
mod variable;

#[cfg(test)]
mod tests;

pub use self::memory::MemoryInstance;
pub use self::module::{ModuleInstance, ModuleInstanceInterface,
	ItemIndex, ExportEntryType, CallerContext, ExecutionParams, FunctionSignature};
pub use self::table::TableInstance;
pub use self::program::ProgramInstance;
pub use self::value::RuntimeValue;
pub use self::variable::{VariableInstance, VariableType, ExternalVariableValue};
pub use self::native::{native_module, UserDefinedElements, UserFunctionExecutor, UserFunctionDescriptor};
