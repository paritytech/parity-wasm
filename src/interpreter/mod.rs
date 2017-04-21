#![allow(dead_code, unused_variables, missing_docs)]

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
	Program(String),
	Initialization(String),
	Function(String),
	Table(String),
	Memory(String),
	Variable(String),
	Global(String),
	Local(String),
	ValueStack(String),
	FrameStack(String),
	Value(String),
	Interpreter(String),
	Trap,
	NotImplemented,
}

impl Into<String> for Error {
	fn into(self) -> String {
		match self {
			Error::Program(s) => s,
			Error::Initialization(s) => s,
			Error::Function(s) => s,
			Error::Table(s) => s,
			Error::Memory(s) => s,
			Error::Variable(s) => s,
			Error::Global(s) => s,
			Error::Local(s) => s,
			Error::ValueStack(s) => s,
			Error::FrameStack(s) => s,
			Error::Interpreter(s) => s,
			Error::Value(s) => s,
			Error::Trap => "trap".into(),
			Error::NotImplemented => "not implemented".into(),
		}
	}
}

mod imports;
mod memory;
mod module;
mod program;
mod runner;
mod table;
mod value;
mod variable;
