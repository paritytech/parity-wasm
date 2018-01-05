use std::rc::Rc;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use elements::{FunctionType, GlobalType, MemoryType, TableType, ValueType};
use interpreter::module::{ExternVal, ModuleInstance};
use interpreter::func::FuncRef;
use interpreter::global::GlobalRef;
use interpreter::memory::MemoryRef;
use interpreter::table::TableRef;
use interpreter::func::FuncInstance;
use interpreter::global::GlobalInstance;
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::{RuntimeValue, TryInto};
use interpreter::Error;
use interpreter::ImportResolver;
use interpreter::state::HostState;

pub type HostFunc = Fn(&mut HostState, &[RuntimeValue])
	-> Result<Option<RuntimeValue>, Error>;

pub struct HostModuleBuilder {
	exports: HashMap<String, ExternVal>,
}

impl HostModuleBuilder {
	pub fn new() -> Self {
		HostModuleBuilder {
			exports: HashMap::new(),
		}
	}

	pub fn insert_func0<
		Cl: Fn(&mut HostState) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type = FunctionType::new(vec![], Ret::value_type());
		let host_func = Rc::new(
			move |state: &mut HostState, args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
				assert!(args.len() == 0);
				let result = f(state)?.into_return_val();
				Ok(result)
			},
		);

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn insert_func1<
		Cl: Fn(&mut HostState, P1) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		P1: FromArg + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type = FunctionType::new(vec![P1::value_type()], Ret::value_type());
		let host_func = Rc::new(
			move |state: &mut HostState, args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
				assert!(args.len() == 1);
				let mut args = args.into_iter();
				let result = f(
					state,
					P1::from_arg(args.next().unwrap())
				)?.into_return_val();
				Ok(result)
			},
		);

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn insert_func2<
		Cl: Fn(&mut HostState, P1, P2) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		P1: FromArg + 'static,
		P2: FromArg + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type =
			FunctionType::new(vec![P1::value_type(), P2::value_type()], Ret::value_type());
		let host_func = Rc::new(
			move |state: &mut HostState, args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
				assert!(args.len() == 2);
				let mut args = args.into_iter();
				let result = f(
					state,
					P1::from_arg(args.next().unwrap()),
					P2::from_arg(args.next().unwrap()),
				)?.into_return_val();
				Ok(result)
			},
		);

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn insert_func3<
		Cl: Fn(&mut HostState, P1, P2, P3) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		P1: FromArg + 'static,
		P2: FromArg + 'static,
		P3: FromArg + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type = FunctionType::new(
			vec![P1::value_type(), P2::value_type(), P3::value_type()],
			Ret::value_type(),
		);
		let host_func = Rc::new(
			move |state: &mut HostState, args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
				assert!(args.len() == 3);
				let mut args = args.into_iter();
				let result = f(
					state,
					P1::from_arg(args.next().unwrap()),
					P2::from_arg(args.next().unwrap()),
					P3::from_arg(args.next().unwrap()),
				)?.into_return_val();
				Ok(result)
			},
		);

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn insert_func4<
		Cl: Fn(&mut HostState, P1, P2, P3, P4) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		P1: FromArg + 'static,
		P2: FromArg + 'static,
		P3: FromArg + 'static,
		P4: FromArg + 'static,
		N: Into<String>,
	>(
		&mut self,
		name: N,
		f: Cl,
	) {
		let func_type = FunctionType::new(
			vec![
				P1::value_type(),
				P2::value_type(),
				P3::value_type(),
				P4::value_type(),
			],
			Ret::value_type(),
		);
		let host_func = Rc::new(
			move |state: &mut HostState, args: &[RuntimeValue]| -> Result<Option<RuntimeValue>, Error> {
				assert!(args.len() == 4);
				let mut args = args.into_iter();
				let result = f(
					state,
					P1::from_arg(args.next().unwrap()),
					P2::from_arg(args.next().unwrap()),
					P3::from_arg(args.next().unwrap()),
					P4::from_arg(args.next().unwrap()),
				)?.into_return_val();
				Ok(result)
			},
		);

		let func = FuncInstance::alloc_host(Rc::new(func_type), host_func);
		self.insert_func(name, func);
	}

	pub fn with_func0<
		Cl: Fn(&mut HostState) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		N: Into<String>,
	>(
		mut self,
		name: N,
		f: Cl,
	) -> Self {
		self.insert_func0(name, f);
		self
	}

	pub fn with_func1<
		Cl: Fn(&mut HostState, P1) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		P1: FromArg + 'static,
		N: Into<String>,
	>(
		mut self,
		name: N,
		f: Cl,
	) -> Self {
		self.insert_func1(name, f);
		self
	}

	pub fn with_func2<
		Cl: Fn(&mut HostState, P1, P2) -> Result<Ret, Error> + 'static,
		Ret: IntoReturnVal + 'static,
		P1: FromArg + 'static,
		P2: FromArg + 'static,
		N: Into<String>,
	>(
		mut self,
		name: N,
		f: Cl,
	) -> Self {
		self.insert_func2(name, f);
		self
	}

	pub fn insert_func<N: Into<String>>(&mut self, name: N, func: FuncRef) {
		self.insert(name, ExternVal::Func(func));
	}

	pub fn insert_global<N: Into<String>>(&mut self, name: N, global: GlobalRef) {
		self.insert(name, ExternVal::Global(global));
	}

	pub fn insert_memory<N: Into<String>>(&mut self, name: N, memory: MemoryRef) {
		self.insert(name, ExternVal::Memory(memory));
	}

	pub fn insert_table<N: Into<String>>(&mut self, name: N, table: TableRef) {
		self.insert(name, ExternVal::Table(table));
	}

	pub fn with_global<N: Into<String>>(mut self, name: N, global: GlobalRef) -> Self {
		self.insert_global(name, global);
		self
	}

	pub fn with_memory<N: Into<String>>(mut self, name: N, memory: MemoryRef) -> Self {
		self.insert_memory(name, memory);
		self
	}

	pub fn with_table<N: Into<String>>(mut self, name: N, table: TableRef) -> Self {
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
		HostModule { internal_instance }
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
	) -> Result<FuncRef, Error> {
		self.internal_instance.resolve_func(field_name, func_type)
	}

	fn resolve_global(
		&self,
		field_name: &str,
		global_type: &GlobalType,
	) -> Result<GlobalRef, Error> {
		self.internal_instance
			.resolve_global(field_name, global_type)
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		memory_type: &MemoryType,
	) -> Result<MemoryRef, Error> {
		self.internal_instance
			.resolve_memory(field_name, memory_type)
	}

	fn resolve_table(
		&self,
		field_name: &str,
		table_type: &TableType,
	) -> Result<TableRef, Error> {
		self.internal_instance.resolve_table(field_name, table_type)
	}
}

pub trait FromArg
where
	Self: Sized,
{
	fn from_arg(arg: &RuntimeValue) -> Self;
	fn value_type() -> ValueType;
}

macro_rules! impl_from_arg {
	($ty: ident, $val_ty: ident) => {
		impl FromArg for $ty {
			fn from_arg(arg: &RuntimeValue) -> Self {
				arg
					.try_into()
					.expect(
						concat!("Due to validation, arg expected to be ", stringify!($val_ty))
					)
			}

			fn value_type() -> ValueType {
				use self::ValueType::*;
				$val_ty
			}
		}
	}
}

impl_from_arg!(i32, I32);
impl_from_arg!(u32, I32);
impl_from_arg!(i64, I64);
impl_from_arg!(u64, I64);
impl_from_arg!(f32, F32);
impl_from_arg!(f64, F64);

pub trait IntoReturnVal {
	fn into_return_val(self) -> Option<RuntimeValue>;
	fn value_type() -> Option<ValueType>;
}

macro_rules! impl_into_return_val {
	($ty: ident, $val_ty: ident) => {
		impl IntoReturnVal for $ty {
			fn into_return_val(self) -> Option<RuntimeValue> {
				Some(self.into())
			}

			fn value_type() -> Option<ValueType> {
				use self::ValueType::*;
				Some($val_ty)
			}
		}
	}
}

impl_into_return_val!(i32, I32);
impl_into_return_val!(u32, I32);
impl_into_return_val!(i64, I64);
impl_into_return_val!(u64, I64);
impl_into_return_val!(f32, F32);
impl_into_return_val!(f64, F64);

impl IntoReturnVal for () {
	fn into_return_val(self) -> Option<RuntimeValue> {
		None
	}

	fn value_type() -> Option<ValueType> {
		None
	}
}

trait Externals {
	fn invoke_index(
		&mut self,
		index: u32,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error>;

	// TODO: or check signature?
	fn signature(&self, index: usize) -> &FunctionType;

	fn memory_by_index(&self, index: usize) -> &MemoryInstance;
	fn table_by_index(&self, index: usize) -> &TableInstance;
	fn global_by_index(&self, index: usize) -> &GlobalInstance;
}


