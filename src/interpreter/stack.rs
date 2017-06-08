use std::collections::VecDeque;
use interpreter::Error;
use interpreter::value::{RuntimeValue, TryInto};

/// Stack with limit.
#[derive(Debug)]
pub struct StackWithLimit<T> where T: Clone {
	/// Stack values.
	values: VecDeque<T>,
	/// Stack limit (maximal stack len).
	limit: usize,
}

impl<T> StackWithLimit<T> where T: Clone {
	pub fn with_data(data: Vec<T>, limit: usize) -> Self {
		StackWithLimit {
			values: data.into_iter().collect(),
			limit: limit,
		}
	}
	
	pub fn with_limit(limit: usize) -> Self {
		StackWithLimit {
			values: VecDeque::new(),
			limit: limit,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.values.is_empty()
	}

	pub fn len(&self) -> usize {
		self.values.len()
	}

	pub fn limit(&self) -> usize {
		self.limit
	}

	pub fn top(&self) -> Result<&T, Error> {
		self.values
			.back()
			.ok_or(Error::Stack("non-empty stack expected".into()))
	}

	pub fn top_mut(&mut self) -> Result<&mut T, Error> {
		self.values
			.back_mut()
			.ok_or(Error::Stack("non-empty stack expected".into()))
	}

	pub fn get(&self, index: usize) -> Result<&T, Error> {
		if index >= self.values.len() {
			return Err(Error::Stack(format!("trying to get value at position {} on stack of size {}", index, self.values.len())));
		}

		Ok(self.values.get(self.values.len() - 1 - index).expect("checked couple of lines above"))
	}

	pub fn push(&mut self, value: T) -> Result<(), Error> {
		if self.values.len() >= self.limit {
			return Err(Error::Stack(format!("exceeded stack limit {}", self.limit)));
		}

		self.values.push_back(value);
		Ok(())
	}

	pub fn pop(&mut self) -> Result<T, Error> {
		self.values
			.pop_back()
			.ok_or(Error::Stack("non-empty stack expected".into()))
	}

	pub fn resize(&mut self, new_size: usize, dummy: T) {
		debug_assert!(new_size <= self.values.len());
		self.values.resize(new_size, dummy);
	}
}

impl StackWithLimit<RuntimeValue> {
	pub fn pop_as<T>(&mut self) -> Result<T, Error>
		where RuntimeValue: TryInto<T, Error> {
		self.pop().and_then(TryInto::try_into)
	}

	pub fn pop_pair(&mut self) -> Result<(RuntimeValue, RuntimeValue), Error> {
		let right = self.pop()?;
		let left = self.pop()?;
		Ok((left, right))
	}

	pub fn pop_pair_as<T>(&mut self) -> Result<(T, T), Error>
		where RuntimeValue: TryInto<T, Error> {
		let right = self.pop_as()?;
		let left = self.pop_as()?;
		Ok((left, right))
	}

	pub fn pop_triple(&mut self) -> Result<(RuntimeValue, RuntimeValue, RuntimeValue), Error> {
		let right = self.pop()?;
		let mid = self.pop()?;
		let left = self.pop()?;
		Ok((left, mid, right))
	}
}
