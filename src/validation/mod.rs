#![allow(unused, missing_docs)]

use std::fmt;
use std::iter::repeat;
use elements::{BlockType, External, FunctionType, GlobalEntry, GlobalType, MemoryType, Module,
               Opcode, ResizableLimits, TableType, Type, ValueType};
use common::stack;
use self::context::ModuleContext;
use self::func::{FunctionValidationContext, Validator};

mod context;
mod module;
mod func;

#[cfg(test)]
mod tests;

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
	let context = prepare_context(module)?;

	let function_section_len = module
		.function_section()
		.map(|s| s.entries().len())
		.unwrap_or(0);
	let code_section_len = module.code_section().map(|s| s.bodies().len()).unwrap_or(0);
	if function_section_len != code_section_len {
		return Err(Error(format!(
			"length of function section is {}, while len of code section is {}",
			function_section_len,
			code_section_len
		)));
	}

	// validate every function body in user modules
	if function_section_len != 0 {
		// tests use invalid code
		let function_section = module
			.function_section()
			.expect("function_section_len != 0; qed");
		let code_section = module
			.code_section()
			.expect("function_section_len != 0; function_section_len == code_section_len; qed");
		// check every function body
		for (index, function) in function_section.entries().iter().enumerate() {
			let function_labels = {
				let function_body = code_section
					.bodies()
					.get(index as usize)
					.ok_or(Error(format!("Missing body for function {}", index)))?;
				Validator::validate_function(&context, function, function_body).map_err(|e| {
					let Error(ref msg) = e;
					Error(format!("Function #{} validation error: {}", index, msg))
				})?;

				// TODO: pepyakin
				// context.function_labels()
			};

			// TODO: pepyakin
			// self.functions_labels.insert(index as u32, function_labels);
		}
	}
	Ok(())
}

fn prepare_context(module: &Module) -> Result<ModuleContext, Error> {
	// TODO: Validate start
 // TODO: Validate imports
 // TODO: Validate exports

	// Copy types from module as is.
	let types = module
		.type_section()
		.map(|ts| ts.types().into_iter().cloned().collect())
		.unwrap_or_default();

	// Fill elements with imported values.

	// TODO: Use Func::type_ref?
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
			global_entry.validate(&globals)?;
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

impl GlobalEntry {
	fn validate(&self, globals_sofar: &[GlobalType]) -> Result<(), Error> {
		let init = self.init_expr().code();
		if init.len() != 2 {
			return Err(Error(
				format!("Init expression should always be with length 2"),
			));
		}
		let init_expr_ty: ValueType = match init[0] {
			Opcode::I32Const(_) => ValueType::I32,
			Opcode::I64Const(_) => ValueType::I64,
			Opcode::F32Const(_) => ValueType::F32,
			Opcode::F64Const(_) => ValueType::F64,
			Opcode::GetGlobal(idx) => match globals_sofar.get(idx as usize) {
				Some(target_global) => {
					if target_global.is_mutable() {
						return Err(Error(format!("Global {} is mutable", idx)));
					}
					target_global.content_type()
				}
				None => {
					return Err(Error(
						format!("Global {} doesn't exists or not yet defined", idx),
					))
				}
			},
			_ => return Err(Error(format!("Non constant opcode in init expr"))),
		};
		if init_expr_ty != self.global_type().content_type() {
			return Err(Error(format!(
				"Trying to initialize variable of type {:?} with value of type {:?}",
				self.global_type().content_type(),
				init_expr_ty
			)));
		}
		if init[1] != Opcode::End {
			return Err(Error(format!("Expression doesn't ends with `end` opcode")));
		}
		Ok(())
	}
}
