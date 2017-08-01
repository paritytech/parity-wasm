//! WebAssembly interpreter module.

/// Interpreter error.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterError<E: CustomUserError> {
	/// Internal error.
	Internal(Error),
	/// Custom user error.
	User(E),
}

/// Custom user error.
pub trait CustomUserError: 'static + ::std::fmt::Display + ::std::fmt::Debug + Clone + PartialEq {
}

/// Internal interpreter error.
#[derive(Debug, Clone, PartialEq)]
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
	/// Env module error.
	Env(String),
	/// Native module error.
	Native(String),
	/// Trap.
	Trap(String),
}

impl<E> From<Error> for InterpreterError<E> where E: CustomUserError {
	fn from(other: Error) -> Self {
		InterpreterError::Internal(other)
	}
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
			Error::Env(s) => s,
			Error::Native(s) => s,
			Error::Trap(s) => format!("trap: {}", s),
		}
	}
}

/// Dummy user error.
#[derive(Debug, Clone, PartialEq)]
pub struct DummyCustomUserError;

impl CustomUserError for DummyCustomUserError {}

impl ::std::fmt::Display for DummyCustomUserError {
	fn fmt(&self, _f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> { Ok(()) }
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
pub use self::module::{ModuleInstance as CustomModuleInstance,
	ModuleInstanceInterface as CustomModuleInstanceInterface,
	ItemIndex, ExportEntryType, CallerContext, ExecutionParams, FunctionSignature};
pub use self::table::TableInstance;
pub use self::program::ProgramInstance as CustomProgramInstance;
pub use self::value::RuntimeValue;
pub use self::variable::{VariableInstance, VariableType, ExternalVariableValue};
pub use self::env_native::{env_native_module, UserDefinedElements, UserFunctionExecutor, UserFunctionDescriptor};
pub use self::env::EnvParams;

/// Default type of ProgramInstance if you do not need any custom user errors.
/// To work with custom user errors or interpreter internals, use CustomProgramInstance.
pub type ProgramInstance = self::program::ProgramInstance<DummyCustomUserError>;

/// Default type of ModuleInstance if you do not need any custom user errors.
/// To work with custom user errors or interpreter internals, use CustomModuleInstance.
pub type ModuleInstance = self::module::ModuleInstance<DummyCustomUserError>;

/// Default type of ModuleInstanceInterface if you do not need any custom user errors.
/// To work with custom user errors or interpreter internals, use CustomModuleInstanceInterface.
pub type ModuleInstanceInterface = self::module::ModuleInstanceInterface<DummyCustomUserError>;
