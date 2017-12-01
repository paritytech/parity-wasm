use elements::{MemoryType, TableType, GlobalType, Type};

pub struct ValidatedModule {
}

impl ValidatedModule {
	pub fn memories(&self) -> &[MemoryType] {
		unimplemented!();
	}

	pub fn tables(&self) -> &[TableType] {
		unimplemented!();
	}

	pub fn globals(&self) -> &[GlobalType] {
		unimplemented!();
	}

	pub fn types(&self) -> &[Type] {
		unimplemented!();
	}

	pub fn function_types(&self) -> &[Type] {
		unimplemented!();
	}
}
