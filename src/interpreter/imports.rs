use std::rc::Rc;
use elements::{FunctionType, GlobalType, MemoryType, TableType};
use interpreter::store::{FuncInstance, GlobalInstance};
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::Error;

pub trait ImportResolver {
	fn resolve_func(
		&self,
		module_name: &str,
		field_name: &str,
		func_type: &FunctionType,
	) -> Result<Rc<FuncInstance>, Error>;

	fn resolve_global(
		&self,
		module_name: &str,
		field_name: &str,
		global_type: &GlobalType,
	) -> Result<Rc<GlobalInstance>, Error>;

	fn resolve_memory(
		&self,
		module_name: &str,
		field_name: &str,
		memory_type: &MemoryType,
	) -> Result<Rc<MemoryInstance>, Error>;

	fn resolve_table(
		&self,
		module_name: &str,
		field_name: &str,
		table_type: &TableType,
	) -> Result<Rc<TableInstance>, Error>;
}
