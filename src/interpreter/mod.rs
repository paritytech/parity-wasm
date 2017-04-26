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
	Stack(String),
	Value(String),
	Interpreter(String),
	Trap(String),
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
			Error::Stack(s) => s,
			Error::Interpreter(s) => s,
			Error::Value(s) => s,
			Error::Trap(s) => format!("trap: {}", s),
			Error::NotImplemented => "not implemented".into(),
		}
	}
}

mod imports;
mod memory;
mod module;
mod program;
mod runner;
mod stack;
mod table;
mod utils;
mod value;
mod variable;
