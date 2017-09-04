use std::u32;
use std::sync::Arc;
use parking_lot::RwLock;
use elements::{TableType, ResizableLimits};
use interpreter::{Error, UserError};
use interpreter::module::check_limits;
use interpreter::variable::{VariableInstance, VariableType};
use interpreter::value::RuntimeValue;

/// Table instance.
pub struct TableInstance<E: UserError> {
	/// Table limits.
	limits: ResizableLimits,
	/// Table variables type.
	variable_type: VariableType,
	/// Table memory buffer.
	buffer: RwLock<Vec<TableElement<E>>>,
}

/// Table element. Cloneable wrapper around VariableInstance.
struct TableElement<E: UserError> {
	pub var: VariableInstance<E>,
}

impl<E> TableInstance<E> where E: UserError {
	/// New instance of the table
	pub fn new(table_type: &TableType) -> Result<Arc<Self>, Error<E>> {
		check_limits(table_type.limits())?;

		let variable_type = table_type.elem_type().into();
		Ok(Arc::new(TableInstance {
			limits: table_type.limits().clone(),
			variable_type: variable_type,
			buffer: RwLock::new(
				vec![TableElement::new(VariableInstance::new(true, variable_type, RuntimeValue::Null)?); table_type.limits().initial() as usize]
			),
		}))
	}

	/// Return table limits.
	pub fn limits(&self) -> &ResizableLimits {
		&self.limits
	}

	/// Get variable type for this table.
	pub fn variable_type(&self) -> VariableType {
		self.variable_type
	}

	/// Get the specific value in the table
	pub fn get(&self, offset: u32) -> Result<RuntimeValue, Error<E>> {
		let buffer = self.buffer.read();
		let buffer_len = buffer.len();
		buffer.get(offset as usize)
			.map(|v| v.var.get())
			.ok_or(Error::Table(format!("trying to read table item with index {} when there are only {} items", offset, buffer_len)))
	}

	/// Set the table value from raw slice
	pub fn set_raw(&self, mut offset: u32, module_name: String, value: &[u32]) -> Result<(), Error<E>> {
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
	pub fn set(&self, offset: u32, value: RuntimeValue) -> Result<(), Error<E>> {
		let mut buffer = self.buffer.write();
		let buffer_len = buffer.len();
		buffer.get_mut(offset as usize)
			.ok_or(Error::Table(format!("trying to update table item with index {} when there are only {} items", offset, buffer_len)))
			.and_then(|v| v.var.set(value))
	}
}

impl<E> TableElement<E> where E: UserError {
	pub fn new(var: VariableInstance<E>) -> Self {
		TableElement {
			var: var,
		}
	}
}

impl<E> Clone for TableElement<E> where E: UserError {
	fn clone(&self) -> Self {
		TableElement::new(VariableInstance::new(self.var.is_mutable(), self.var.variable_type(), self.var.get())
			.expect("it only fails when variable_type() != passed variable value; both are read from already constructed var; qed"))
	}
}
