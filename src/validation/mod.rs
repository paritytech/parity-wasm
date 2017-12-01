#![allow(unused, missing_docs)]

mod module;
mod func;

use std::fmt;
use elements::{Module, ResizableLimits, MemoryType, TableType, GlobalType, External};
use common::stack;
use self::module::ModuleContext;

pub struct Error(String);

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<stack::Error> for Error {
	fn from(e: stack::Error) -> Error {
		Error(format!("Stack: {}", e))
	}
}

pub fn validate_module(module: &Module) -> Result<(), Error> {
	if let Some(table_section) = module.table_section() {
		table_section
			.entries()
			.iter()
			.map(TableType::validate)
			.collect::<Result<_, _>>()?
	}

	if let Some(mem_section) = module.memory_section() {
		mem_section
			.entries()
			.iter()
			.map(MemoryType::validate)
			.collect::<Result<_, _>>()?
	}

	Ok(())
}

fn prepare_context(module: &Module) -> ModuleContext {
	// Copy types from module as is.
	let types = module
		.type_section()
		.map(|ts| ts.types().into_iter().cloned().collect())
		.unwrap_or_default();

	// Fill elements with imported values.
	let mut func_type_indexes = Vec::new();
	let mut tables = Vec::new();
	let mut memories = Vec::new();
	let mut globals = Vec::new();

	for import_entry in module
		.import_section()
		.map(|i| i.entries())
		.unwrap_or_default()
	{
		match import_entry.external() {
			&External::Function(idx) => func_type_indexes.push(idx),
			&External::Table(ref table) => tables.push(table.clone()),
			&External::Memory(ref memory) => memories.push(memory.clone()),
			&External::Global(ref global) => globals.push(global.clone()),
		}
	}

	// Concatenate elements with defined in the module.

	ModuleContext {
		types,
		tables,
		memories,
		globals,
		func_type_indexes,
	}
}

impl ResizableLimits {
	fn validate(&self) -> Result<(), Error> {
		if let Some(maximum) = self.maximum() {
			if self.initial() > maximum {
				return Err(Error(format!(
					"maximum limit {} is lesser than minimum {}",
					maximum,
					self.initial()
				)));
			}
		}
		Ok(())
	}
}

impl MemoryType {
	fn validate(&self) -> Result<(), Error> {
		self.limits().validate()
	}
}

impl TableType {
	fn validate(&self) -> Result<(), Error> {
		self.limits().validate()
	}
}

#[cfg(test)]
mod tests {
	use super::validate_module;
	use builder::module;
	use elements::{BlockType, ExportEntry, External, FunctionType, GlobalEntry, GlobalType,
	               ImportEntry, InitExpr, Internal, MemoryType, Opcode, Opcodes, TableType,
	               ValueType};

	#[test]
	fn empty_is_valid() {
		let module = module().build();
		assert!(validate_module(&module).is_ok());
	}

	#[test]
	fn mem_limits() {
		// min > max
		let m = module()
			.memory()
				.with_min(10)
				.with_max(Some(9))
				.build()
			.build();
		assert!(validate_module(&m).is_err());

		// min = max
		let m = module()
			.memory()
				.with_min(10)
				.with_max(Some(10))
				.build()
			.build();
		assert!(validate_module(&m).is_ok());

		// mem is always valid without max.
		let m = module()
			.memory()
				.with_min(10)
				.build()
			.build();
		assert!(validate_module(&m).is_ok());
	}

	#[test]
	fn table_limits() {
		// min > max
		let m = module()
			.table()
				.with_min(10)
				.with_max(Some(9))
				.build()
			.build();
		assert!(validate_module(&m).is_err());

		// min = max
		let m = module()
			.table()
				.with_min(10)
				.with_max(Some(10))
				.build()
			.build();
		assert!(validate_module(&m).is_ok());

		// table is always valid without max.
		let m = module()
			.table()
				.with_min(10)
				.build()
			.build();
		assert!(validate_module(&m).is_ok());
	}

	// #[test]
	// fn if_else_with_return_type_validation() {
	// 	let module_instance = ModuleInstance::new(Weak::default(), "test".into(), module().build()).unwrap();
	// 	let mut context = FunctionValidationContext::new(&module_instance, None, &[], 1024, 1024, FunctionSignature::Module(&FunctionType::default()));

	// 	Validator::validate_function(&mut context, BlockType::NoResult, &[
	// 		Opcode::I32Const(1),
	// 		Opcode::If(BlockType::NoResult),
	// 			Opcode::I32Const(1),
	// 			Opcode::If(BlockType::Value(ValueType::I32)),
	// 				Opcode::I32Const(1),
	// 			Opcode::Else,
	// 				Opcode::I32Const(2),
	// 			Opcode::End,
	// 		Opcode::Drop,
	// 		Opcode::End,
	// 		Opcode::End,
	// 	]).unwrap();
	// }
}
