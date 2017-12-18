use std::rc::Rc;
use std::collections::HashMap;
use elements::{FunctionType, GlobalType, MemoryType, TableType};
use interpreter::func::FuncInstance;
use interpreter::global::GlobalInstance;
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::Error;

pub struct Imports<'a, St: 'a> {
	modules: HashMap<String, &'a ImportResolver<St>>,
}

impl<'a, St: 'a> Default for Imports<'a, St> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a, St: 'a> Imports<'a, St> {
	pub fn new() -> Imports<'a, St> {
		Imports { modules: HashMap::new() }
	}

	pub fn with_resolver<N: Into<String>>(
		mut self,
		name: N,
		resolver: &'a ImportResolver<St>,
	) -> Self {
		self.modules.insert(name.into(), resolver);
		self
	}

	pub fn push_resolver<N: Into<String>>(&mut self, name: N, resolver: &'a ImportResolver<St>) {
		self.modules.insert(name.into(), resolver);
	}

	pub fn resolver(&self, name: &str) -> Option<&ImportResolver<St>> {
		self.modules.get(name).cloned()
	}
}

pub trait ImportResolver<St> {
	fn resolve_func(
		&self,
		field_name: &str,
		func_type: &FunctionType,
	) -> Result<Rc<FuncInstance<St>>, Error>;

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
	) -> Result<Rc<TableInstance<St>>, Error>;
}
