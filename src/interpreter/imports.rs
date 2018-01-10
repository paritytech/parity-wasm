use std::collections::HashMap;
use elements::{FunctionType, GlobalType, MemoryType, TableType};
use interpreter::global::GlobalRef;
use interpreter::memory::MemoryRef;
use interpreter::func::FuncRef;
use interpreter::table::TableRef;
use interpreter::module::ModuleRef;
use interpreter::Error;

pub struct Imports<'a> {
	modules: HashMap<String, &'a ImportResolver>,
}

impl<'a> Default for Imports<'a> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a> Imports<'a> {
	pub fn new() -> Imports<'a> {
		Imports { modules: HashMap::new() }
	}

	pub fn with_resolver<N: Into<String>>(
		mut self,
		name: N,
		resolver: &'a ImportResolver,
	) -> Self {
		self.modules.insert(name.into(), resolver);
		self
	}

	pub fn push_resolver<N: Into<String>>(&mut self, name: N, resolver: &'a ImportResolver) {
		self.modules.insert(name.into(), resolver);
	}

	pub fn resolver(&self, name: &str) -> Option<&ImportResolver> {
		self.modules.get(name).cloned()
	}
}

pub trait ImportResolver {
	fn resolve_func(
		&self,
		field_name: &str,
		func_type: &FunctionType,
	) -> Result<FuncRef, Error>;

	fn resolve_global(
		&self,
		field_name: &str,
		global_type: &GlobalType,
	) -> Result<GlobalRef, Error>;

	fn resolve_memory(
		&self,
		field_name: &str,
		memory_type: &MemoryType,
	) -> Result<MemoryRef, Error>;

	fn resolve_table(
		&self,
		field_name: &str,
		table_type: &TableType,
	) -> Result<TableRef, Error>;
}

impl ImportResolver for ModuleRef {
	fn resolve_func(
		&self,
		field_name: &str,
		_func_type: &FunctionType,
	) -> Result<FuncRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Validation(format!("Export {} not found", field_name))
			})?
			.as_func()
			.ok_or_else(|| {
				Error::Validation(format!("Export {} is not a function", field_name))
			})?)
	}

	fn resolve_global(
		&self,
		field_name: &str,
		_global_type: &GlobalType,
	) -> Result<GlobalRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} not found", field_name))
			})?
			.as_global()
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} is not a global", field_name))
			})?)
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		_memory_type: &MemoryType,
	) -> Result<MemoryRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} not found", field_name))
			})?
			.as_memory()
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} is not a memory", field_name))
			})?)
	}

	fn resolve_table(
		&self,
		field_name: &str,
		_table_type: &TableType,
	) -> Result<TableRef, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} not found", field_name))
			})?
			.as_table()
			.ok_or_else(|| {
				Error::Instantiation(format!("Export {} is not a table", field_name))
			})?)
	}
}
