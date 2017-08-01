use std::collections::VecDeque;
use interpreter::{Error, CustomUserError};
use interpreter::value::{RuntimeValue, TryInto};

/// Stack with limit.
#[derive(Debug)]
pub struct StackWithLimit<T, E> where T: Clone, E: CustomUserError {
	/// Stack values.
	values: VecDeque<T>,
	/// Stack limit (maximal stack len).
	limit: usize,
	/// Dummy to avoid compilation error.
	_dummy: ::std::marker::PhantomData<E>,
}

impl<T, E> StackWithLimit<T, E> where T: Clone, E: CustomUserError {
	pub fn with_data(data: Vec<T>, limit: usize) -> Self {
		StackWithLimit {
			values: data.into_iter().collect(),
			limit: limit,
			_dummy: Default::default(),
		}
	}
	
	pub fn with_limit(limit: usize) -> Self {
		StackWithLimit {
			values: VecDeque::new(),
			limit: limit,
			_dummy: Default::default(),
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

	pub fn values(&self) -> &VecDeque<T> {
		&self.values
	}

	pub fn top(&self) -> Result<&T, Error<E>> {
		self.values
			.back()
			.ok_or(Error::Stack("non-empty stack expected".into()))
	}

	pub fn top_mut(&mut self) -> Result<&mut T, Error<E>> {
		self.values
			.back_mut()
			.ok_or(Error::Stack("non-empty stack expected".into()))
	}

	pub fn get(&self, index: usize) -> Result<&T, Error<E>> {
		if index >= self.values.len() {
			return Err(Error::Stack(format!("trying to get value at position {} on stack of size {}", index, self.values.len())));
		}

		Ok(self.values.get(self.values.len() - 1 - index).expect("checked couple of lines above"))
	}

	pub fn push(&mut self, value: T) -> Result<(), Error<E>> {
		if self.values.len() >= self.limit {
			return Err(Error::Stack(format!("exceeded stack limit {}", self.limit)));
		}

		self.values.push_back(value);
		Ok(())
	}

	pub fn push_penultimate(&mut self, value: T) -> Result<(), Error<E>> {
		if self.values.is_empty() {
			return Err(Error::Stack("trying to insert penultimate element into empty stack".into()));
		}
		self.push(value)?;

		let last_index = self.values.len() - 1;
		let penultimate_index = last_index - 1;
		self.values.swap(last_index, penultimate_index);

		Ok(())
	}

	pub fn pop(&mut self) -> Result<T, Error<E>> {
		self.values
			.pop_back()
			.ok_or(Error::Stack("non-empty stack expected".into()))
	}

	pub fn resize(&mut self, new_size: usize, dummy: T) {
		debug_assert!(new_size <= self.values.len());
		self.values.resize(new_size, dummy);
	}
}

impl<E> StackWithLimit<RuntimeValue, E> where E: CustomUserError {
	pub fn pop_as<T>(&mut self) -> Result<T, Error<E>>
		where RuntimeValue: TryInto<T, Error<E>> {
		self.pop().and_then(TryInto::try_into)
	}

	pub fn pop_pair(&mut self) -> Result<(RuntimeValue, RuntimeValue), Error<E>> {
		let right = self.pop()?;
		let left = self.pop()?;
		Ok((left, right))
	}

	pub fn pop_pair_as<T>(&mut self) -> Result<(T, T), Error<E>>
		where RuntimeValue: TryInto<T, Error<E>> {
		let right = self.pop_as()?;
		let left = self.pop_as()?;
		Ok((left, right))
	}

	pub fn pop_triple(&mut self) -> Result<(RuntimeValue, RuntimeValue, RuntimeValue), Error<E>> {
		let right = self.pop()?;
		let mid = self.pop()?;
		let left = self.pop()?;
		Ok((left, mid, right))
	}
}
