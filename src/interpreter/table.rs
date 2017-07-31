use std::u32;
use std::sync::Arc;
use parking_lot::RwLock;
use elements::TableType;
use interpreter::Error;
use interpreter::module::check_limits;
use interpreter::variable::{VariableInstance, VariableType};
use interpreter::value::RuntimeValue;

/// Table instance.
pub struct TableInstance {
	/// Table variables type.
	variable_type: VariableType,
	/// Table memory buffer.
	buffer: RwLock<Vec<TableElement>>,
}

/// Table element. Cloneable wrapper around VariableInstance.
struct TableElement {
	pub var: VariableInstance,
}

impl TableInstance {
	/// New instance of the table
	pub fn new(table_type: &TableType) -> Result<Arc<Self>, Error> {
		check_limits(table_type.limits())?;

		let variable_type = table_type.elem_type().into();
		Ok(Arc::new(TableInstance {
			variable_type: variable_type,
			buffer: RwLock::new(
				vec![TableElement::new(VariableInstance::new(true, variable_type, RuntimeValue::Null)?); table_type.limits().initial() as usize]
			),
		}))
	}

	/// Get variable type for this table.
	pub fn variable_type(&self) -> VariableType {
		self.variable_type
	}

	/// Get the specific value in the table
	pub fn get(&self, offset: u32) -> Result<RuntimeValue, Error> {
		let buffer = self.buffer.read();
		let buffer_len = buffer.len();
		buffer.get(offset as usize)
			.map(|v| v.var.get())
			.ok_or(Error::Table(format!("trying to read table item with index {} when there are only {} items", offset, buffer_len)))
	}

	/// Set the table value from raw slice
	pub fn set_raw(&self, mut offset: u32, module_name: String, value: &[u32]) -> Result<(), Error> {
		for val in value {
			match self.variable_type {
				VariableType::AnyFunc => self.set(offset, RuntimeValue::AnyFunc(module_name.clone(), *val))?,
				_ => return Err(Error::Table(format!("table of type {:?} is not supported", self.variable_type))),
			}
			offset += 1;
		}
		Ok(())
	}

	/// Set the table from runtime variable value
	pub fn set(&self, offset: u32, value: RuntimeValue) -> Result<(), Error> {
		let mut buffer = self.buffer.write();
		let buffer_len = buffer.len();
		buffer.get_mut(offset as usize)
			.ok_or(Error::Table(format!("trying to update table item with index {} when there are only {} items", offset, buffer_len)))
			.and_then(|v| v.var.set(value))
	}
}

impl TableElement {
	pub fn new(var: VariableInstance) -> Self {
		TableElement {
			var: var,
		}
	}
}

impl Clone for TableElement {
	fn clone(&self) -> Self {
		TableElement::new(VariableInstance::new(self.var.is_mutable(), self.var.variable_type(), self.var.get())
			.expect("it only fails when variable_type() != passed variable value; both are read from already constructed var; qed"))
	}
}
