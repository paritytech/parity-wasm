//! WebAssembly interpreter module.

/// Interpreter error.
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
pub use self::module::{ModuleInstance, ModuleInstanceInterface, ItemIndex, CallerContext, ExecutionParams};
pub use self::table::TableInstance;
pub use self::program::ProgramInstance;
pub use self::value::RuntimeValue;
pub use self::variable::VariableInstance;
pub use self::env_native::{env_native_module, UserFunctions, UserFunction, UserFunctionExecutor};
pub use self::env::EnvParams;