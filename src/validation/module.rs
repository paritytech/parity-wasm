use elements::{MemoryType, TableType, GlobalType, Type};

pub struct ValidatedModule {
	pub memories: Vec<MemoryType>,
	pub tables: Vec<TableType>,
	pub globals: Vec<GlobalType>,
	pub types: Vec<Type>,
	pub func_type_indexes: Vec<u32>,
}
