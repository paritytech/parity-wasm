use interpreter::{Error as InterpreterError};
use interpreter::value::{RuntimeValue, TryInto};
use common::stack::StackWithLimit;

impl StackWithLimit<RuntimeValue> {
	pub fn pop_as<T>(&mut self) -> Result<T, InterpreterError>
	where
		RuntimeValue: TryInto<T, InterpreterError>,
	{
		let value = self.pop()?;
		TryInto::try_into(value)
	}

	pub fn pop_pair_as<T>(&mut self) -> Result<(T, T), InterpreterError>
	where
		RuntimeValue: TryInto<T, InterpreterError>,
	{
		let right = self.pop_as()?;
		let left = self.pop_as()?;
		Ok((left, right))
	}

	pub fn pop_triple(&mut self) -> Result<(RuntimeValue, RuntimeValue, RuntimeValue), InterpreterError> {
		let right = self.pop()?;
		let mid = self.pop()?;
		let left = self.pop()?;
		Ok((left, mid, right))
	}
}
