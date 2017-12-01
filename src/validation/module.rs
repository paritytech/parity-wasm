use elements::{MemoryType, TableType, GlobalType, Type};

pub struct ModuleContext {
	pub memories: Vec<MemoryType>,
	pub tables: Vec<TableType>,
	pub globals: Vec<GlobalType>,
	pub types: Vec<Type>,
	pub func_type_indexes: Vec<u32>,
}

impl ModuleContext {
	pub fn memories(&self) -> &[MemoryType] {
		&self.memories
	}

	pub fn tables(&self) -> &[TableType] {
		&self.tables
	}

	pub fn globals(&self) -> &[GlobalType] {
		&self.globals
	}

	pub fn types(&self) -> &[Type] {
		&self.types
	}

	pub fn func_type_indexes(&self) -> &[u32] {
		&self.func_type_indexes
	}
}
