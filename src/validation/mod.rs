#![allow(unused, missing_docs)]

mod module;
mod func;

use std::fmt;
use elements::{Module, ResizableLimits, MemoryType, TableType, GlobalType, FunctionType, External, Opcode, ValueType};
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
	prepare_context(module).map(|_| ())
}

fn prepare_context(module: &Module) -> Result<ModuleContext, Error> {
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
	if let Some(table_section) = module.table_section() {
		for table_entry in table_section.entries() {
			table_entry.validate()?;
			tables.push(table_entry.clone());
		}
	}
	if let Some(mem_section) = module.memory_section() {
		for mem_entry in mem_section.entries() {
			mem_entry.validate()?;
			memories.push(mem_entry.clone());
		}
	}
	if let Some(global_section) = module.global_section() {
		for global_entry in global_section.entries() {
			let init = global_entry.init_expr().code();
			if init.len() != 2 {
				return Err(Error(format!("Init expression should always be with length 2")));
			}
			let init_expr_ty: ValueType = match init[0] {
				Opcode::I32Const(_) => ValueType::I32,
				Opcode::I64Const(_) => ValueType::I64,
				Opcode::F32Const(_) => ValueType::F32,
				Opcode::F64Const(_) => ValueType::F64,
				Opcode::GetGlobal(idx) => {
					match globals.get(idx as usize) {
						Some(target_global) => {
							if target_global.is_mutable() {
								return Err(Error(
									format!("Global {} is mutable", idx)
								));
							}
							target_global.content_type()
						},
						None => return Err(Error(
							format!("Global {} doesn't exists", idx)
						)),
					}
				},
				_ => return Err(Error(format!("Non constant opcode in init expr"))),
			};
			if init_expr_ty != global_entry.global_type().content_type() {
				return Err(Error(
					format!(
						"Trying to initialize variable of type {:?} with value of type {:?}",
						global_entry.global_type().content_type(),
						init_expr_ty
					)
				));
			}
			if init[1] != Opcode::End {
				return Err(Error(format!("Expression doesn't ends with `end` opcode")));
			}
			globals.push(global_entry.global_type().clone());
		}
	}

	Ok(ModuleContext {
		types,
		tables,
		memories,
		globals,
		func_type_indexes,
	})
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

		// mem is always valid without max
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

		// table is always valid without max
		let m = module()
			.table()
				.with_min(10)
				.build()
			.build();
		assert!(validate_module(&m).is_ok());
	}

	#[test]
	fn global_init_const() {
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(
							vec![Opcode::I32Const(42), Opcode::End]
						)
					)
				)
			.build();
		assert!(validate_module(&m).is_ok());

		// without delimiting End opcode
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(vec![Opcode::I32Const(42)])
					)
				)
			.build();
		assert!(validate_module(&m).is_err());

		// init expr type differs from declared global type
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I64, true),
						InitExpr::new(vec![Opcode::I32Const(42), Opcode::End])
					)
				)
			.build();
		assert!(validate_module(&m).is_err());
	}

	#[test]
	fn global_init_global() {
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, false),
						InitExpr::new(vec![Opcode::I32Const(0), Opcode::End])
					)
				)
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(vec![Opcode::GetGlobal(0), Opcode::End])
					)
				)
			.build();
		assert!(validate_module(&m).is_ok());

		// get_global can reference only previously defined globals
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(vec![Opcode::GetGlobal(0), Opcode::End])
					)
				)
			.build();
		assert!(validate_module(&m).is_err());

		// get_global can reference only const globals
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(vec![Opcode::I32Const(0), Opcode::End])
					)
				)
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(vec![Opcode::GetGlobal(0), Opcode::End])
					)
				)
			.build();
		assert!(validate_module(&m).is_err());
	}

	#[test]
	fn global_init_misc() {
		// empty init expr
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(vec![Opcode::End])
					)
				)
			.build();
		assert!(validate_module(&m).is_err());

		// not an constant opcode used
		let m = module()
				.with_global(
					GlobalEntry::new(
						GlobalType::new(ValueType::I32, true),
						InitExpr::new(vec![Opcode::Unreachable, Opcode::End])
					)
				)
			.build();
		assert!(validate_module(&m).is_err());
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
