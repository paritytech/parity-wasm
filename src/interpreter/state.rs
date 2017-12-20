
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub unsafe trait StateKey {
	type Static: ?Sized + 'static;
}

pub fn type_id<T>() -> TypeId
where
	T: StateKey,
	T::Static: Any,
{
	TypeId::of::<T::Static>()
}

#[derive(Default)]
pub struct HostState<'a> {
	data: HashMap<TypeId, *mut ()>,
	_marker: ::std::marker::PhantomData<&'a mut ()>,
}

impl<'a> HostState<'a> {
	pub fn new() -> Self {
		HostState {
			data: HashMap::default(),
			_marker: ::std::marker::PhantomData,
		}
	}

	pub fn insert<V: StateKey + 'a>(&mut self, val: &'a mut V) {
		let ty_id = type_id::<V>();
		let ptr = val as *mut V as *mut ();
		let existing = self.data.insert(ty_id, ptr);
		assert!(existing.is_none());
	}

	pub fn with<V: StateKey + 'a, R, F: FnOnce(&V) -> R>(&self, f: F) -> R {
		let ptr = self.data.get(&type_id::<V>()).unwrap();
		unsafe {
			let val_ref = &*(*ptr as *const V);
			f(val_ref)
		}
	}

	pub fn with_mut<V: StateKey + 'a, R, F: FnOnce(&mut V) -> R + 'static>(&mut self, f: F) -> R {
		let ptr = self.data.get_mut(&type_id::<V>()).unwrap();
		unsafe {
			let val_ref = &mut *(*ptr as *mut V);
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

	/// Safety of this impl should be ensured by a macro.
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
			host_state.with_mut(|my_state: &mut MyState| {
				my_state.inc();
				my_state.get()
			})
		};

		assert_eq!(new_value, counter);
	}

	struct MyImmutableState<'a>(&'a i32);
	/// Safety of this impl should be ensured by a macro.
	unsafe impl<'a> StateKey for MyImmutableState<'a> {
		type Static = MyImmutableState<'static>;
	}

	struct StaticState(i32);

	impl StaticState {
		fn inc(&mut self) {
			self.0 += 1;
		}

		fn get(&self) -> i32 {
			self.0
		}
	}

	/// Safety of this impl should be ensured by a macro.
	unsafe impl<'a> StateKey for StaticState {
		type Static = StaticState;
	}

	#[test]
	fn compiles_with_static() {
		let mut static_state = StaticState(45);
		let mut host_state = HostState::new();
		host_state.insert::<StaticState>(&mut static_state);
		host_state.with_mut(|my_state: &mut StaticState| {
			my_state.inc();
		});
		host_state.with_mut(|my_state: &mut StaticState| {
			my_state.inc();
			assert_eq!(47, my_state.get());
		})
	}

	#[test]
	#[should_panic]
	fn doesnt_allow_dups() {
		let mut static_state_1 = StaticState(45);
		let mut static_state_2 = StaticState(45);
		let mut host_state = HostState::new();
		host_state.insert::<StaticState>(&mut static_state_1);
		host_state.insert::<StaticState>(&mut static_state_2);
	}
}
