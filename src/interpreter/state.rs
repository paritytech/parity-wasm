
use std::any::{TypeId, Any};
use std::collections::HashMap;

pub unsafe trait StateKey {
    type Static: ?Sized + 'static;
}

/// Returns the `TypeId` for `T::Static`
pub fn type_id<T>() -> TypeId
    where T: StateKey, T::Static: Any
{
    TypeId::of::<T::Static>()
}

#[derive(Default)]
pub struct HostState<'a> {
	data: HashMap<TypeId, *mut ()>,
	_marker: ::std::marker::PhantomData<&'a ()>,
}

impl<'a> HostState<'a> {
	pub fn new() -> Self {
		HostState {
			data: HashMap::default(),
			_marker: ::std::marker::PhantomData,
		}
	}

	pub fn insert<V: StateKey>(&mut self, val: &'a mut V) {
		let ty_id = type_id::<V>();
		let ptr = val as *mut V as *mut ();
		let existing = self.data.insert(ty_id, ptr);
		assert!(existing.is_none());
	}

	pub fn with_state<V: StateKey, R, F: FnOnce(&mut V) -> R>(&mut self, f: F) -> R {
		let ty_id = type_id::<V>();
		let ptr = self.data.get_mut(&ty_id).unwrap();
		unsafe {
			let val_ref = &mut * { *ptr as *mut V };
			f(val_ref)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct MyState<'a>(&'a mut i32);
	impl<'a> MyState<'a> {
		fn inc(&mut self) {
			*self.0 += 1;
		}

		fn get(&self) -> i32 {
			*self.0
		}
	}

	unsafe impl<'a> StateKey for MyState<'a> {
		type Static = MyState<'static>;
	}

	#[test]
	fn it_works() {
		let mut counter = 33i32;

		let new_value = {
			let mut my_state = MyState(&mut counter);
			let mut host_state = HostState::new();
			host_state.insert::<MyState>(&mut my_state);
			host_state.with_state(|my_state: &mut MyState| {
				my_state.inc();
				my_state.get()
			})
		};

		assert_eq!(new_value, counter);
	}
}
