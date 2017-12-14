use std::any::Any;
use std::rc::Rc;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use elements::{FunctionType, ValueType, GlobalType, MemoryType, TableType};
use interpreter::module::{ExternVal, ModuleInstance};
use interpreter::func::FuncInstance;
use interpreter::global::GlobalInstance;
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::Error;
use interpreter::ImportResolver;

pub struct HostModuleBuilder<St> {
	exports: HashMap<String, ExternVal>,
	_marker: PhantomData<St>,
}

impl<St: 'static> HostModuleBuilder<St> {
	pub fn new() -> Self {
		HostModuleBuilder {
			exports: HashMap::new(),
			_marker: PhantomData,
		}
	}

	pub fn insert_func0<
		Cl: Fn(&mut St) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturnVal + 'static,
		F: Into<Func0<Cl, St, Ret>>,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: F,
	) {
		let func_type = Func0::<Cl, St, Ret>::derive_func_type();
		let host_func = Rc::new(f.into()) as Rc<AnyFunc>;
		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn with_func1<
		Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error> + 'static,
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
		let host_func = Rc::new(f.into()) as Rc<AnyFunc>;
		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn with_func2<
		Cl: Fn(&mut St, P1, P2) -> Result<Option<Ret>, Error> + 'static,
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
		let host_func = Rc::new(f.into()) as Rc<AnyFunc>;
		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn insert_func<N: Into<String>>(&mut self, name: N, func: Rc<FuncInstance>) {
		self.insert(name, ExternVal::Func(func));
	}

	pub fn insert_global<N: Into<String>>(&mut self, name: N, global: Rc<GlobalInstance>) {
		self.insert(name, ExternVal::Global(global));
	}

	pub fn insert_memory<N: Into<String>>(&mut self, name: N, memory: Rc<MemoryInstance>) {
		self.insert(name, ExternVal::Memory(memory));
	}

	pub fn insert_table<N: Into<String>>(&mut self, name: N, table: Rc<TableInstance>) {
		self.insert(name, ExternVal::Table(table));
	}

	pub fn with_global<N: Into<String>>(mut self, name: N, global: Rc<GlobalInstance>) -> Self {
		self.insert_global(name, global);
		self
	}

	pub fn with_memory<N: Into<String>>(mut self, name: N, memory: Rc<MemoryInstance>) -> Self {
		self.insert_memory(name, memory);
		self
	}

	pub fn with_table<N: Into<String>>(mut self, name: N, table: Rc<TableInstance>) -> Self {
		self.insert_table(name, table);
		self
	}

	fn insert<N: Into<String>>(&mut self, name: N, extern_val: ExternVal) {
		match self.exports.entry(name.into()) {
			Entry::Vacant(v) => v.insert(extern_val),
			Entry::Occupied(o) => panic!("Duplicate export name {}", o.key()),
		};
	}

	pub fn build(self) -> HostModule {
		let internal_instance = Rc::new(ModuleInstance::with_exports(self.exports));
		HostModule {
			internal_instance
		}
	}
}

pub struct HostModule {
	internal_instance: Rc<ModuleInstance>,
}

impl HostModule {
	pub fn export_by_name(&self, name: &str) -> Option<ExternVal> {
		self.internal_instance.export_by_name(name)
	}
}

impl ImportResolver for HostModule {
	fn resolve_func(
		&self,
		field_name: &str,
		func_type: &FunctionType,
	) -> Result<Rc<FuncInstance>, Error> {
		self.internal_instance.resolve_func(field_name, func_type)
	}

	fn resolve_global(
		&self,
		field_name: &str,
		global_type: &GlobalType,
	) -> Result<Rc<GlobalInstance>, Error> {
		self.internal_instance.resolve_global(field_name, global_type)
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		memory_type: &MemoryType,
	) -> Result<Rc<MemoryInstance>, Error> {
		self.internal_instance.resolve_memory(field_name, memory_type)
	}

	fn resolve_table(
		&self,
		field_name: &str,
		table_type: &TableType,
	) -> Result<Rc<TableInstance>, Error> {
		self.internal_instance.resolve_table(field_name, table_type)
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

pub struct Func0<Cl: Fn(&mut St) -> Result<Option<Ret>, Error>, St, Ret: AsReturnVal> {
	closure: Cl,
	_marker: PhantomData<(St, Ret)>,
}

impl<
	St: 'static,
	Ret: AsReturnVal,
	Cl: Fn(&mut St) -> Result<Option<Ret>, Error>,
> AnyFunc for Func0<Cl, St, Ret> {
	fn call_as_any(
		&self,
		state: &mut Any,
		_args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let state = state.downcast_mut::<St>().unwrap();
		let result = (self.closure)(state);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}

impl<St: 'static, Ret: AsReturnVal, Cl: Fn(&mut St) -> Result<Option<Ret>, Error>> From<Cl>
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
	Cl: Fn(&mut St) -> Result<Option<Ret>, Error>,
> Func0<Cl, St, Ret> {
	fn derive_func_type() -> FunctionType {
		FunctionType::new(vec![], Ret::value_type())
	}
}

pub struct Func1<Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error>, St, Ret: AsReturnVal, P1: FromArg> {
	closure: Cl,
	_marker: PhantomData<(St, Ret, P1)>,
}

impl<
	St: 'static,
	Ret: AsReturnVal,
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

impl<St: 'static, Ret: AsReturnVal, P1: FromArg, Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error>> From<Cl>
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
	Cl: Fn(&mut St, P1) -> Result<Option<Ret>, Error>,
> Func1<Cl, St, Ret, P1> {
	fn derive_func_type() -> FunctionType {
		FunctionType::new(vec![P1::value_type()], Ret::value_type())
	}
}

pub struct Func2<Cl: Fn(&mut St, P1, P2) -> Result<Option<Ret>, Error>, St, Ret: AsReturnVal, P1: FromArg, P2: FromArg> {
	closure: Cl,
	_marker: PhantomData<(St, Ret, P1, P2)>,
}

impl<
	St: 'static,
	Ret: AsReturnVal,
	P1: FromArg,
	P2: FromArg,
	Cl: Fn(&mut St, P1, P2) -> Result<Option<Ret>, Error>,
> AnyFunc for Func2<Cl, St, Ret, P1, P2> {
	fn call_as_any(
		&self,
		state: &mut Any,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let state = state.downcast_mut::<St>().unwrap();
		let p1 = P1::from_arg(&args[0]);
		let p2 = P2::from_arg(&args[1]);
		let result = (self.closure)(state, p1, p2);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}

impl<St: 'static, Ret: AsReturnVal, P1: FromArg, P2: FromArg, Cl: Fn(&mut St, P1, P2) -> Result<Option<Ret>, Error>> From<Cl>
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
	Cl: Fn(&mut St, P1, P2) -> Result<Option<Ret>, Error>,
> Func2<Cl, St, Ret, P1, P2> {
	fn derive_func_type() -> FunctionType {
		FunctionType::new(vec![P1::value_type(), P2::value_type()], Ret::value_type())
	}
}
