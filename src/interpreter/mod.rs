//! WebAssembly interpreter module.

use std::any::TypeId;

/// Custom user error.
pub trait UserError: 'static + ::std::fmt::Display + ::std::fmt::Debug {
	#[doc(hidden)]
    fn __private_get_type_id__(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl UserError {
	pub fn downcast_ref<T: UserError>(&self) -> Option<&T> {
        if self.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const UserError as *const T)) }
        } else {
            None
        }
    }

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
pub enum Error<E> where E: UserError {
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
	/// Env module error.
	Env(String),
	/// Native module error.
	Native(String),
	/// Trap.
	Trap(String),
	/// Custom user error.
	User(Box<UserError>),
	Other(E),
}

impl<E> Into<String> for Error<E> where E: UserError {
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
			Error::Env(s) => s,
			Error::Native(s) => s,
			Error::Trap(s) => format!("trap: {}", s),
			Error::User(e) => format!("user: {}", e),
			Error::Other(_) => panic!("TODO: Remove this arm "),
		}
	}
}

impl<E> ::std::fmt::Display for Error<E> where E: UserError {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
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
			Error::Env(ref s) => write!(f, "Env: {}", s),
			Error::Native(ref s) => write!(f, "Native: {}", s),
			Error::Trap(ref s) => write!(f, "Trap: {}", s),
			Error::User(ref e) => write!(f, "User: {}", e),
			Error::Other(_) => panic!("TODO: Remove this arm "),
		}
	}
}

/// Dummy user error.
#[derive(Debug, Clone, PartialEq)]
pub struct DummyUserError;

impl UserError for DummyUserError {}

impl ::std::fmt::Display for DummyUserError {
	fn fmt(&self, _f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> { Ok(()) }
}

impl<U> From<U> for Error<U> where U: UserError + Sized {
	fn from(e: U) -> Self {
		Error::User(Box::new(e))
	}
}

mod env;
mod env_native;
mod imports;
mod memory;
mod module;
mod program;
mod runner;
mod stack;
mod table;
mod validator;
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
pub use self::env_native::{env_native_module, UserDefinedElements, UserFunctionExecutor, UserFunctionDescriptor};
pub use self::env::EnvParams;

/// Default type of Error if you do not need any custom user errors.
pub type DummyError = Error<DummyUserError>;

/// Default type of ProgramInstance if you do not need any custom user errors.
/// To work with custom user errors or interpreter internals, use CustomProgramInstance.
pub type DefaultProgramInstance = self::program::ProgramInstance<DummyUserError>;

/// Default type of ModuleInstance if you do not need any custom user errors.
/// To work with custom user errors or interpreter internals, use CustomModuleInstance.
pub type DefaultModuleInstance = self::module::ModuleInstance<DummyUserError>;

/// Default type of ModuleInstanceInterface if you do not need any custom user errors.
/// To work with custom user errors or interpreter internals, use CustomModuleInstanceInterface.
pub type DefaultModuleInstanceInterface = self::module::ModuleInstanceInterface<DummyUserError>;
