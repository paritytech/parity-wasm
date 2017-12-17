use std::rc::Rc;
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

pub type HostFunc<St> = Fn(&St, &[RuntimeValue]) -> Result<Option<RuntimeValue>, Error>;

pub struct HostModuleBuilder<St> {
	exports: HashMap<String, ExternVal<St>>,
}

impl<St> HostModuleBuilder<St> {
	pub fn new() -> Self {
		HostModuleBuilder {
			exports: HashMap::new(),
		}
	}

	pub fn with_func0<
		Cl: Fn(&St) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturnVal + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type = FunctionType::new(vec![], Ret::value_type());
		let host_func = Rc::new(move |state: &St, _args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
			let result = f(state);
			result.map(|r| r.and_then(|r| r.as_return_val()))
		});

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn with_func1<
		Cl: Fn(&St, P1) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturnVal + 'static,
		P1: FromArg + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type = FunctionType::new(vec![P1::value_type()], Ret::value_type());
		let host_func = Rc::new(move |state: &St, args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
			let arg0 = P1::from_arg(&args[0]);
			let result = f(state, arg0);
			result.map(|r| r.and_then(|r| r.as_return_val()))
		});

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn with_func2<
		Cl: Fn(&St, P1, P2) -> Result<Option<Ret>, Error> + 'static,
		Ret: AsReturnVal + 'static,
		P1: FromArg + 'static,
		P2: FromArg + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type = FunctionType::new(vec![P1::value_type(), P2::value_type()], Ret::value_type());
		let host_func = Rc::new(move |state: &St, args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
			let p1 = P1::from_arg(&args[0]);
			let p2 = P2::from_arg(&args[1]);
			let result = f(state, p1, p2);
			result.map(|r| r.and_then(|r| r.as_return_val()))
		});

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn insert_func<N: Into<String>>(&mut self, name: N, func: Rc<FuncInstance<St>>) {
		self.insert(name, ExternVal::Func(func));
	}

	pub fn insert_global<N: Into<String>>(&mut self, name: N, global: Rc<GlobalInstance>) {
		self.insert(name, ExternVal::Global(global));
	}

	pub fn insert_memory<N: Into<String>>(&mut self, name: N, memory: Rc<MemoryInstance>) {
		self.insert(name, ExternVal::Memory(memory));
	}

	pub fn insert_table<N: Into<String>>(&mut self, name: N, table: Rc<TableInstance<St>>) {
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

	pub fn with_table<N: Into<String>>(mut self, name: N, table: Rc<TableInstance<St>>) -> Self {
		self.insert_table(name, table);
		self
	}

	fn insert<N: Into<String>>(&mut self, name: N, extern_val: ExternVal<St>) {
		match self.exports.entry(name.into()) {
			Entry::Vacant(v) => v.insert(extern_val),
			Entry::Occupied(o) => panic!("Duplicate export name {}", o.key()),
		};
	}

	pub fn build(self) -> HostModule<St> {
		let internal_instance = Rc::new(ModuleInstance::with_exports(self.exports));
		HostModule {
			internal_instance
		}
	}
}

pub struct HostModule<St> {
	internal_instance: Rc<ModuleInstance<St>>,
}

impl<St> HostModule<St> {
	pub fn export_by_name(&self, name: &str) -> Option<ExternVal<St>> {
		self.internal_instance.export_by_name(name)
	}
}

impl<St> ImportResolver<St> for HostModule<St> {
	fn resolve_func(
		&self,
		field_name: &str,
		func_type: &FunctionType,
	) -> Result<Rc<FuncInstance<St>>, Error> {
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
	) -> Result<Rc<TableInstance<St>>, Error> {
		self.internal_instance.resolve_table(field_name, table_type)
	}
}

pub trait AnyFunc<St> {
	fn call_as_any(
		&self,
		state: &St,
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

impl<St, Ret: AsReturnVal> AnyFunc<St> for Fn(&St) -> Result<Option<Ret>, Error> {
	fn call_as_any(
		&self,
		state: &St,
		_args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let result = self(state);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}

impl<St, Ret: AsReturnVal, P1: FromArg, P2: FromArg> AnyFunc<St> for Fn(&St, P1, P2) -> Result<Option<Ret>, Error> {
	fn call_as_any(
		&self,
		state: &St,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		let p1 = P1::from_arg(&args[0]);
		let p2 = P2::from_arg(&args[1]);
		let result = self(state, p1, p2);
		result.map(|r| r.and_then(|r| r.as_return_val()))
	}
}
