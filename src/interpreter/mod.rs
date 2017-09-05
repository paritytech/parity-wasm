//! WebAssembly interpreter module.

/// Custom user error.
pub trait UserError: 'static + ::std::fmt::Display + ::std::fmt::Debug + Clone + PartialEq {
}

/// Internal interpreter error.
#[derive(Debug, Clone, PartialEq)]
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
	User(E),
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
		Error::User(e)
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
