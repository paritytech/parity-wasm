use std::any::{Any, TypeId};
use std::sync::Arc;
use std::marker::PhantomData;
use std::collections::HashMap;
use elements::{FunctionType, ValueType, GlobalType, MemoryType, TableType};
use interpreter::store::{Store, ExternVal, ModuleId, ModuleInstance};
use interpreter::value::RuntimeValue;
use interpreter::Error;

enum HostItem {
	Func {
		name: String,
		func_type: FunctionType,
		host_func: Arc<AnyFunc>,
	},
	Global {
		name: String,
		global_type: GlobalType,
		init_val: RuntimeValue,
	},
	Memory {
		name: String,
		memory_type: MemoryType,
	},
	Table {
		name: String,
		table_type: TableType,
	},
	ExternVal {
		name: String,
		extern_val: ExternVal,
	}
}

pub struct HostModuleBuilder<St> {
	items: Vec<HostItem>,
	_marker: PhantomData<St>,
}

impl<St: 'static> HostModuleBuilder<St> {
	pub fn new() -> Self {
		HostModuleBuilder {
			items: Vec::new(),
			_marker: PhantomData,
		}
	}

	pub fn with_func0<
		Cl: Fn(&mut Store, &mut St) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturnVal + 'static,
		F: Into<Func0<Cl, St, Ret>>,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: F,
	) {
		let func_type = Func0::<Cl, St, Ret>::derive_func_type();
		let host_func = Arc::new(f.into()) as Arc<AnyFunc>;

		self.items.push(HostItem::Func {
			name: name.into(),
			func_type,
			host_func,
		});
	}

	pub fn with_func1<
		Cl: Fn(&mut Store, &mut St, P1) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturnVal + 'static,
		P1: FromArg + 'static,
		F: Into<Func1<Cl, St, Ret, P1>>,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: F,
	) {
		let func_type = Func1::<Cl, St, Ret, P1>::derive_func_type();
		let host_func = Arc::new(f.into()) as Arc<AnyFunc>;

		self.items.push(HostItem::Func {
			name: name.into(),
			func_type,
			host_func,
		});
	}

	pub fn with_func2<
		Cl: Fn(&mut Store, &mut St, P1, P2) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturnVal + 'static,
		P1: FromArg + 'static,
		P2: FromArg + 'static,
		F: Into<Func2<Cl, St, Ret, P1, P2>>,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: F,
	) {
		let func_type = Func2::<Cl, St, Ret, P1, P2>::derive_func_type();
		let host_func = Arc::new(f.into()) as Arc<AnyFunc>;

		self.items.push(HostItem::Func {
			name: name.into(),
			func_type,
			host_func,
		});
	}

	pub fn with_global<N: Into<String>>(&mut self, name: N, global_type: GlobalType, init_val: RuntimeValue) {
		self.items.push(HostItem::Global {
			name: name.into(),
			global_type,
			init_val,
		});
	}

	pub fn with_memory<N: Into<String>>(&mut self, name: N, memory_type: MemoryType) {
		self.items.push(HostItem::Memory {
			name: name.into(),
			memory_type,
		});
	}

	pub fn with_table<N: Into<String>>(&mut self, name: N, table_type: TableType) {
		self.items.push(HostItem::Table {
			name: name.into(),
			table_type,
		});
	}

	pub fn with_extern_val<N: Into<String>>(&mut self, name: N, extern_val: ExternVal) {
		self.items.push(HostItem::ExternVal {
			name: name.into(),
			extern_val,
		});
	}

	pub fn build(self) -> HostModule {
		HostModule {
			items: self.items
		}
	}
}

pub struct HostModule {
	items: Vec<HostItem>,
}

impl HostModule {
	pub(crate) fn allocate(self, store: &mut Store) -> Result<ModuleId, Error> {
		let mut exports = HashMap::new();

		for item in self.items {
			match item {
				HostItem::Func { name, func_type, host_func } => {
					let type_id = store.alloc_func_type(func_type);
					let func_id = store.alloc_host_func(type_id, host_func);
					exports.insert(name, ExternVal::Func(func_id));
				},
				HostItem::Global { name, global_type, init_val } => {
					let global_id = store.alloc_global(global_type, init_val);
					exports.insert(name, ExternVal::Global(global_id));
				},
				HostItem::Memory { name, memory_type } => {
					let memory_id = store.alloc_memory(&memory_type)?;
					exports.insert(name, ExternVal::Memory(memory_id));
				},
				HostItem::Table { name, table_type } => {
					let table_id = store.alloc_table(&table_type)?;
					exports.insert(name, ExternVal::Table(table_id));
				}
				HostItem::ExternVal { name, extern_val } => {
					exports.insert(name, extern_val);
				}
			}
		}

		let host_module_instance = ModuleInstance::with_exports(exports);
		let module_id = store.add_module_instance(host_module_instance);

		Ok(module_id)
	}
}

pub trait AnyFunc {
	fn call_as_any(
		&self,
		store: &mut Store,
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
			unexpected => panic!("Expected I32, got {:?}", unexpected),
		}
	}

	fn value_type() -> ValueType {
		ValueType::I32
	}
}

pub trait AsReturnVal {
	fn as_return_val(self) -> Option<RuntimeValue>;
	fn value_type() -> Option<ValueType>;
}

impl AsReturnVal for i32 {
	fn as_return_val(self) -> Option<RuntimeValue> {
		Some(self.into())
	}

	fn value_type() -> Option<ValueType> {
		Some(ValueType::I32)
	}
}

impl AsReturnVal for () {
	fn as_return_val(self) -> Option<RuntimeValue> {
		None
	}

	fn value_type() -> Option<ValueType> {
		None
	}
}

pub struct Func0<Cl: Fn(&mut Store, &mut St) -> Result<Option<Ret>, Error>, St, Ret: AsReturnVal> {
	closure: Cl,
	_marker: PhantomData<(St, Ret)>,
}

impl<
	St: 'static,
	Ret: AsReturnVal,
	Cl: Fn(&mut Store, &mut St) -> Result<Option<Ret>, Error>,
> AnyFunc for Func0<Cl, St, Ret> {
	fn call_as_any(
		&self,
		store: &mut Store,
		state: &mut Any,
		_args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let state = state.downcast_mut::<St>().unwrap();
		let result = (self.closure)(store, state);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}

impl<St: 'static, Ret: AsReturnVal, Cl: Fn(&mut Store, &mut St) -> Result<Option<Ret>, Error>> From<Cl>
	for Func0<Cl, St, Ret> {
	fn from(cl: Cl) -> Self {
		Func0 {
			closure: cl,
			_marker: PhantomData,
		}
	}
}

impl<
	St: 'static,
	Ret: AsReturnVal,
	Cl: Fn(&mut Store, &mut St) -> Result<Option<Ret>, Error>,
> Func0<Cl, St, Ret> {
	fn derive_func_type() -> FunctionType {
		FunctionType::new(vec![], Ret::value_type())
	}
}

pub struct Func1<Cl: Fn(&mut Store, &mut St, P1) -> Result<Option<Ret>, Error>, St, Ret: AsReturnVal, P1: FromArg> {
	closure: Cl,
	_marker: PhantomData<(St, Ret, P1)>,
}

impl<
	St: 'static,
	Ret: AsReturnVal,
	P1: FromArg,
	Cl: Fn(&mut Store, &mut St, P1) -> Result<Option<Ret>, Error>,
> AnyFunc for Func1<Cl, St, Ret, P1> {
	fn call_as_any(
		&self,
		store: &mut Store,
		state: &mut Any,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let state = state.downcast_mut::<St>().unwrap();
		let p1 = P1::from_arg(&args[0]);
		let result = (self.closure)(store, state, p1);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}

impl<St: 'static, Ret: AsReturnVal, P1: FromArg, Cl: Fn(&mut Store, &mut St, P1) -> Result<Option<Ret>, Error>> From<Cl>
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
	Ret: AsReturnVal,
	P1: FromArg,
	Cl: Fn(&mut Store, &mut St, P1) -> Result<Option<Ret>, Error>,
> Func1<Cl, St, Ret, P1> {
	fn derive_func_type() -> FunctionType {
		FunctionType::new(vec![P1::value_type()], Ret::value_type())
	}
}

pub struct Func2<Cl: Fn(&mut Store, &mut St, P1, P2) -> Result<Option<Ret>, Error>, St, Ret: AsReturnVal, P1: FromArg, P2: FromArg> {
	closure: Cl,
	_marker: PhantomData<(St, Ret, P1, P2)>,
}

impl<
	St: 'static,
	Ret: AsReturnVal,
	P1: FromArg,
	P2: FromArg,
	Cl: Fn(&mut Store, &mut St, P1, P2) -> Result<Option<Ret>, Error>,
> AnyFunc for Func2<Cl, St, Ret, P1, P2> {
	fn call_as_any(
		&self,
		store: &mut Store,
		state: &mut Any,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let state = state.downcast_mut::<St>().unwrap();
		let p1 = P1::from_arg(&args[0]);
		let p2 = P2::from_arg(&args[1]);
		let result = (self.closure)(store, state, p1, p2);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}

impl<St: 'static, Ret: AsReturnVal, P1: FromArg, P2: FromArg, Cl: Fn(&mut Store, &mut St, P1, P2) -> Result<Option<Ret>, Error>> From<Cl>
	for Func2<Cl, St, Ret, P1, P2> {
	fn from(cl: Cl) -> Self {
		Func2 {
			closure: cl,
			_marker: PhantomData,
		}
	}
}

impl<
	St: 'static,
	Ret: AsReturnVal,
	P1: FromArg,
	P2: FromArg,
	Cl: Fn(&mut Store, &mut St, P1, P2) -> Result<Option<Ret>, Error>,
> Func2<Cl, St, Ret, P1, P2> {
	fn derive_func_type() -> FunctionType {
		FunctionType::new(vec![P1::value_type(), P2::value_type()], Ret::value_type())
	}
}

use interpreter::UserError;
use interpreter::store::MemoryId;

// custom user error
#[derive(Debug, Clone, PartialEq)]
struct UserErrorWithCode {
	error_code: i32,
}

impl ::std::fmt::Display for UserErrorWithCode {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		write!(f, "{}", self.error_code)
	}
}

impl UserError for UserErrorWithCode {}

// TODO: Rename to state
// user function executor
struct FunctionExecutor {
	pub memory: MemoryId,
	pub values: Vec<i32>,
}

// TODO: Remove this stuff
fn build_env_module() -> HostModule {
	let mut builder = HostModuleBuilder::<FunctionExecutor>::new();
	builder.with_func2("add", |store: &mut Store, state: &mut FunctionExecutor, arg: i32, unused: i32| {
		let memory_value = state.memory.resolve(store).get(0, 1).unwrap()[0];
		let fn_argument_unused = unused as u8;
		let fn_argument = arg as u8;
		assert_eq!(fn_argument_unused, 0);

		let sum = memory_value + fn_argument;
		state.memory.resolve(store).set(0, &vec![sum]).unwrap();
		state.values.push(sum as i32);
		Ok(Some(sum as i32))
	});
	builder.with_func2("sub", |store: &mut Store, state: &mut FunctionExecutor, arg: i32, unused: i32| {
		let memory_value = state.memory.resolve(store).get(0, 1).unwrap()[0];
		let fn_argument_unused = unused as u8;
		let fn_argument = arg as u8;
		assert_eq!(fn_argument_unused, 0);

		let diff = memory_value - fn_argument;
		state.memory.resolve(store).set(0, &vec![diff]).unwrap();
		state.values.push(diff as i32);
		Ok(Some(diff as i32))
	});
	builder.with_func0("err", |store: &mut Store, state: &mut FunctionExecutor| -> Result<Option<i32>, Error> {
		Err(Error::User(Box::new(UserErrorWithCode { error_code: 777 })))
	});
	builder.with_memory("memory", MemoryType::new(256, None));
	builder.build()
}
