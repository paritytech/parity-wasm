use std::rc::Rc;
use std::collections::HashMap;
use elements::{FunctionType, GlobalType, MemoryType, TableType};
use interpreter::func::FuncInstance;
use interpreter::global::GlobalInstance;
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
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
	) -> Result<Rc<FuncInstance>, Error>;

	fn resolve_global(
		&self,
		field_name: &str,
		global_type: &GlobalType,
	) -> Result<Rc<GlobalInstance>, Error>;

	fn resolve_memory(
		&self,
		field_name: &str,
		memory_type: &MemoryType,
	) -> Result<Rc<MemoryInstance>, Error>;

	fn resolve_table(
		&self,
		field_name: &str,
		table_type: &TableType,
	) -> Result<Rc<TableInstance>, Error>;
}
