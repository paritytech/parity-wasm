use std::u32;
use std::sync::Arc;
use elements::TableType;
use interpreter::Error;
use interpreter::value::RuntimeValue;

/// Table instance.
pub struct TableInstance {
	/// Table memory buffer.
	buffer: Vec<RuntimeValue>,
	/// Maximum buffer size.
	maximum_size: u32,
}

impl TableInstance {
	pub fn new(table_type: &TableType) -> Result<Arc<Self>, Error> {
		Ok(Arc::new(TableInstance {
			buffer: vec![RuntimeValue::Null; table_type.limits().initial() as usize],
			maximum_size: table_type.limits().maximum().unwrap_or(u32::MAX),
		}))
	}

	pub fn set(&self, offset: u32, value: &[u32]) -> Result<Self, Error> {
		unimplemented!()
	}
}
