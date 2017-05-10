use std::u32;
use std::sync::Arc;
use parking_lot::RwLock;
use elements::TableType;
use interpreter::Error;
use interpreter::variable::{VariableInstance, VariableType};
use interpreter::value::RuntimeValue;

/// Table instance.
pub struct TableInstance {
	/// Table variables type.
	variable_type: VariableType,
	/// Table memory buffer.
	buffer: RwLock<Vec<VariableInstance>>,
}

impl TableInstance {
	pub fn new(variable_type: VariableType, table_type: &TableType) -> Result<Arc<Self>, Error> {
		Ok(Arc::new(TableInstance {
			variable_type: variable_type,
			buffer: RwLock::new(
				vec![VariableInstance::new(true, variable_type, RuntimeValue::Null)?; table_type.limits().initial() as usize]
			),
		}))
	}

	pub fn get(&self, offset: u32) -> Result<RuntimeValue, Error> {
		let buffer = self.buffer.read();
		let buffer_len = buffer.len();
		buffer.get(offset as usize)
			.map(|v| v.get())
			.ok_or(Error::Table(format!("trying to read table item with index {} when there are only {} items", offset, buffer_len)))
	}

	pub fn set_raw(&self, mut offset: u32, value: &[u32]) -> Result<(), Error> {
		for val in value {
			match self.variable_type {
				VariableType::AnyFunc => self.set(offset, RuntimeValue::AnyFunc(*val))?,
				_ => return Err(Error::Table(format!("table of type {:?} is not supported", self.variable_type))),
			}
			offset += 1;
		}
		Ok(())
	}

	pub fn set(&self, offset: u32, value: RuntimeValue) -> Result<(), Error> {
		let mut buffer = self.buffer.write();
		let buffer_len = buffer.len();
		buffer.get_mut(offset as usize)
			.ok_or(Error::Table(format!("trying to update table item with index {} when there are only {} items", offset, buffer_len)))
			.and_then(|v| v.set(value))
	}
}
