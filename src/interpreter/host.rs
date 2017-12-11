use std::any::Any;
use std::sync::Arc;
use std::marker::PhantomData;
use std::collections::HashMap;
use elements::{FunctionType, ValueType, GlobalType, MemoryType, TableType};
use interpreter::store::{Store, ExternVal};
use interpreter::value::RuntimeValue;
use interpreter::Error;

pub struct HostModuleBuilder<'a, St> {
	store: &'a mut Store,
	exports: HashMap<String, ExternVal>,
	_marker: PhantomData<St>,
}

impl<'a, St: 'static> HostModuleBuilder<'a, St> {
	pub fn new(store: &'a mut Store) -> Self {
		HostModuleBuilder {
			store: store,
			exports: HashMap::new(),
			_marker: PhantomData,
		}
	}

	pub fn with_func1<
		Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturn + 'static,
		P1: FromArg + 'static,
		F: Into<Func1<Cl, St, Ret, P1>>,
	>(
		&mut self,
		name: &str,
		f: F,
	) {
		let func_type = Func1::<Cl, St, Ret, P1>::derive_func_type();
		let type_id = self.store.alloc_func_type(func_type);

		let anyfunc = Arc::new(f.into()) as Arc<AnyFunc>;

		let func_id = self.store.alloc_host_func(type_id, anyfunc);
		self.exports.insert(name.to_owned(), ExternVal::Func(func_id));
	}

	pub fn with_global(&mut self, name: &str, global_type: GlobalType, val: RuntimeValue) {
		let global_id = self.store.alloc_global(global_type, val);
		self.exports.insert(name.to_owned(), ExternVal::Global(global_id));
	}

	pub fn with_memory(&mut self, name: &str, memory_type: &MemoryType) -> Result<(), Error> {
		let memory_id = self.store.alloc_memory(memory_type)?;
		self.exports.insert(name.to_owned(), ExternVal::Memory(memory_id));
		Ok(())
	}

	pub fn with_table(&mut self, name: &str, table_type: &TableType) {
		let table_id = self.store.alloc_table(table_type);
	}
}

pub trait AnyFunc {
	fn call_as_any(
		&self,
		state: &mut Any,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error>;
}

pub trait FromArg {
	fn from_arg(arg: &RuntimeValue) -> Self;
	fn value_type() -> ValueType;
}

impl FromArg for i32 {
	fn from_arg(arg: &RuntimeValue) -> Self {
		match arg {
			&RuntimeValue::I32(v) => v,
			unexpected => panic!("Unexpected runtime value {:?}", unexpected)
		}
	}

	fn value_type() -> ValueType {
		ValueType::I32
	}
}

pub trait AsReturn {
	fn as_return_val(self) -> Option<RuntimeValue>;
	fn value_type() -> Option<ValueType>;
}

impl AsReturn for i32 {
	fn as_return_val(self) -> Option<RuntimeValue> {
		Some(self.into())
	}

	fn value_type() -> Option<ValueType> {
		Some(ValueType::I32)
	}
}

impl AsReturn for () {
	fn as_return_val(self) -> Option<RuntimeValue> {
		None
	}

	fn value_type() -> Option<ValueType> {
		None
	}
}

pub struct Func1<Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error>, St, Ret: AsReturn, P1: FromArg> {
	closure: Cl,
	_marker: PhantomData<(St, Ret, P1)>,
}

impl<
	St: 'static,
	Ret: AsReturn,
	P1: FromArg,
	Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error>,
> AnyFunc for Func1<Cl, St, Ret, P1> {
	fn call_as_any(
		&self,
		state: &mut Any,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let state = state.downcast_mut::<St>().unwrap();
		let p1 = P1::from_arg(&args[0]);
		let result = (self.closure)(state, p1);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}

impl<St: 'static, Ret: AsReturn, P1: FromArg, Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error>> From<Cl>
	for Func1<Cl, St, Ret, P1> {
	fn from(cl: Cl) -> Self {
		Func1 {
			closure: cl,
			_marker: PhantomData,
		}
	}
}

impl<
	St: 'static,
	Ret: AsReturn,
	P1: FromArg,
	Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error>,
> Func1<Cl, St, Ret, P1> {
	fn derive_func_type() -> FunctionType {
		FunctionType::new(vec![P1::value_type()], Ret::value_type())
	}
}
