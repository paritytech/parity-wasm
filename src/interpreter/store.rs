// TODO: remove this
#![allow(unused)]

use std::rc::Rc;
use std::any::Any;
use std::fmt;
use std::collections::HashMap;
use elements::{FunctionType, GlobalEntry, GlobalType, InitExpr, Internal, External, Local, MemoryType,
               Module, Opcode, Opcodes, TableType, Type, ResizableLimits};
use interpreter::{Error, ExecutionParams, MemoryInstance,
                  RuntimeValue, TableInstance};
use interpreter::runner::{prepare_function_args, FunctionContext, Interpreter};
use interpreter::host::AnyFunc;
use validation::validate_module;
use common::{DEFAULT_FRAME_STACK_LIMIT, DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX,
             DEFAULT_VALUE_STACK_LIMIT};
use common::stack::StackWithLimit;

#[derive(Copy, Clone, Debug)]
pub struct TypeId(u32);

impl TypeId {
	pub fn resolve<'s>(&self, store: &'s Store) -> &'s FunctionType {
		store.types
			.get(self.0 as usize)
			.expect("ID should always be a valid index")
	}
}

#[derive(Copy, Clone, Debug)]
pub struct ModuleId(u32);

impl ModuleId {
	pub fn memory_by_index(&self, store: &Store, idx: u32) -> Option<Rc<MemoryInstance>> {
		store.resolve_module(*self)
			.memories
			.get(idx as usize)
			.cloned()
	}

	pub fn table_by_index(&self, store: &Store, idx: u32) -> Option<Rc<TableInstance>> {
		store.resolve_module(*self)
			.tables
			.get(idx as usize)
			.cloned()
	}

	pub fn global_by_index(&self, store: &Store, idx: u32) -> Option<GlobalId> {
		store.resolve_module(*self)
			.globals
			.get(idx as usize)
			.cloned()
	}

	pub fn func_by_index(&self, store: &Store, idx: u32) -> Option<Rc<FuncInstance>> {
		store.resolve_module(*self)
			.funcs
			.get(idx as usize)
			.cloned()
	}

	pub fn type_by_index(&self, store: &Store, idx: u32) -> Option<TypeId> {
		store.resolve_module(*self)
			.types
			.get(idx as usize)
			.cloned()
	}

	pub fn export_by_name(&self, store: &Store, name: &str) -> Option<ExternVal> {
		store.resolve_module(*self)
			.exports
			.get(name)
			.cloned()
	}
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalId(u32);

#[derive(Clone, Debug)]
pub enum ExternVal {
	Func(Rc<FuncInstance>),
	Table(Rc<TableInstance>),
	Memory(Rc<MemoryInstance>),
	Global(GlobalId),
}

impl ExternVal {
	pub fn as_func(&self) -> Option<Rc<FuncInstance>> {
		match *self {
			ExternVal::Func(ref func) => Some(Rc::clone(func)),
			_ => None,
		}
	}

	pub fn as_table(&self) -> Option<Rc<TableInstance>> {
		match *self {
			ExternVal::Table(ref table) => Some(Rc::clone(table)),
			_ => None,
		}
	}

	pub fn as_memory(&self) -> Option<Rc<MemoryInstance>> {
		match *self {
			ExternVal::Memory(ref memory) => Some(Rc::clone(memory)),
			_ => None,
		}
	}

	pub fn as_global(&self) -> Option<GlobalId> {
		match *self {
			ExternVal::Global(global) => Some(global),
			_ => None,
		}
	}
}

#[derive(Clone)]
pub enum FuncInstance {
	Internal {
		func_type: TypeId,
		module: ModuleId,
		body: Rc<FuncBody>,
	},
	Host {
		func_type: TypeId,
		host_func: Rc<AnyFunc>,
	},
}

impl fmt::Debug for FuncInstance {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			FuncInstance::Internal { func_type, module, .. } => {
				write!(f, "Internal {{ type={:?}, module={:?} }}", func_type, module)
			},
			FuncInstance::Host { func_type, .. } => {
				write!(f, "Host {{ type={:?} }}", func_type)
			}
		}
	}
}

impl FuncInstance {
	pub fn func_type(&self) -> TypeId {
		match *self {
			FuncInstance::Internal { func_type, .. } | FuncInstance::Host { func_type, .. } => {
				func_type
			}
		}
	}

	pub fn body(&self) -> Option<Rc<FuncBody>> {
		match *self {
			FuncInstance::Internal { ref body, .. } => Some(Rc::clone(body)),
			FuncInstance::Host { .. } => None,
		}
	}
}

#[derive(Clone, Debug)]
pub struct FuncBody {
	pub locals: Vec<Local>,
	pub opcodes: Opcodes,
	pub labels: HashMap<usize, usize>,
}

#[derive(Debug)]
pub struct GlobalInstance {
	val: RuntimeValue,
	mutable: bool,
}

impl GlobalInstance {
	fn new(val: RuntimeValue, mutable: bool) -> GlobalInstance {
		GlobalInstance { val, mutable }
	}
}

pub struct ExportInstance {
	name: String,
	val: ExternVal,
}

#[derive(Default, Debug)]
pub struct ModuleInstance {
	types: Vec<TypeId>,
	tables: Vec<Rc<TableInstance>>,
	funcs: Vec<Rc<FuncInstance>>,
	memories: Vec<Rc<MemoryInstance>>,
	globals: Vec<GlobalId>,
	exports: HashMap<String, ExternVal>,
}

impl ModuleInstance {
	fn new() -> ModuleInstance {
		ModuleInstance::default()
	}

	pub fn with_exports(exports: HashMap<String, ExternVal>) -> ModuleInstance {
		ModuleInstance {
			exports, ..Default::default()
		}
	}
}

#[derive(Default, Debug)]
pub struct Store {
	// TODO: u32 capped vectors.
	funcs: Vec<FuncInstance>,
	tables: Vec<TableInstance>,
	memories: Vec<MemoryInstance>,
	globals: Vec<GlobalInstance>,

	// These are not the part of specification of the Store.
	// However, they can be referenced in several places, so it is handy to have it here.
	modules: Vec<ModuleInstance>,
	types: Vec<FunctionType>,
}

impl Store {
	pub fn new() -> Store {
		Store::default()
	}

	fn resolve_module(&self, id: ModuleId) -> &ModuleInstance {
		self.modules
			.get(id.0 as usize)
			.expect("ID should always be a valid index")
	}

	pub fn alloc_func_type(&mut self, func_type: FunctionType) -> TypeId {
		self.types.push(func_type);
		let type_id = self.types.len() - 1;
		TypeId(type_id as u32)
	}

	pub fn alloc_func(&mut self, module: ModuleId, func_type: TypeId, body: FuncBody) -> Rc<FuncInstance> {
		let func = FuncInstance::Internal {
			func_type,
			module,
			body: Rc::new(body),
		};
		Rc::new(func)
	}

	pub fn alloc_host_func(&mut self, func_type: TypeId, host_func: Rc<AnyFunc>) -> Rc<FuncInstance> {
		let func = FuncInstance::Host {
			func_type,
			host_func,
		};
		Rc::new(func)
	}

	pub fn alloc_table(&mut self, table_type: &TableType) -> Result<Rc<TableInstance>, Error> {
		let table = TableInstance::new(table_type)?;
		Ok(Rc::new(table))
	}

	pub fn alloc_memory(&mut self, mem_type: &MemoryType) -> Result<Rc<MemoryInstance>, Error> {
		let memory = MemoryInstance::new(&mem_type)?;
		Ok(Rc::new(memory))
	}

	pub fn alloc_global(&mut self, global_type: GlobalType, val: RuntimeValue) -> GlobalId {
		let global = GlobalInstance::new(val, global_type.is_mutable());
		self.globals.push(global);
		let global_id = self.globals.len() - 1;
		GlobalId(global_id as u32)
	}

	fn alloc_module_internal(
		&mut self,
		module: &Module,
		extern_vals: &[ExternVal],
		instance: &mut ModuleInstance,
		module_id: ModuleId,
	) -> Result<(), Error> {
		let mut aux_data = validate_module(module)?;

		for &Type::Function(ref ty) in module
			.type_section()
			.map(|ts| ts.types())
			.unwrap_or(&[])
		{
			let type_id = self.alloc_func_type(ty.clone());
			instance.types.push(type_id);
		}

		{
			let imports = module.import_section().map(|is| is.entries()).unwrap_or(&[]);
			if imports.len() != extern_vals.len() {
				return Err(Error::Initialization(format!("extern_vals length is not equal to import section entries")));
			}

			for (import, extern_val) in Iterator::zip(imports.into_iter(), extern_vals.into_iter())
			{
				match (import.external(), extern_val) {
					(&External::Function(fn_type_idx), &ExternVal::Func(ref func)) => {
						let expected_fn_type = instance.types.get(fn_type_idx as usize).expect("Due to validation function type should exists").resolve(self);
						let actual_fn_type = func.func_type().resolve(self);
						if expected_fn_type != actual_fn_type {
							return Err(Error::Initialization(format!(
								"Expected function with type {:?}, but actual type is {:?} for entry {}",
								expected_fn_type,
								actual_fn_type,
								import.field(),
							)));
						}
						instance.funcs.push(Rc::clone(func))
					}
					(&External::Table(ref tt), &ExternVal::Table(ref table)) => {
						match_limits(table.limits(), tt.limits())?;
						instance.tables.push(Rc::clone(table));
					}
					(&External::Memory(ref mt), &ExternVal::Memory(ref memory)) => {
						match_limits(memory.limits(), mt.limits())?;
						instance.memories.push(Rc::clone(memory));
					}
					(&External::Global(ref gl), &ExternVal::Global(global)) => {
						// TODO: check globals
						instance.globals.push(global)
					}
					(expected_import, actual_extern_val) => {
						return Err(Error::Initialization(format!(
							"Expected {:?} type, but provided {:?} extern_val",
							expected_import,
							actual_extern_val
						)));
					}
				}
			}
		}

		{
			let funcs = module
				.function_section()
				.map(|fs| fs.entries())
				.unwrap_or(&[]);
			let bodies = module.code_section().map(|cs| cs.bodies()).unwrap_or(&[]);
			debug_assert!(
				funcs.len() == bodies.len(),
				"Due to validation func and body counts must match"
			);

			for (index, (ty, body)) in
				Iterator::zip(funcs.into_iter(), bodies.into_iter()).enumerate()
			{
				let func_type = instance.types[ty.type_ref() as usize];
				let labels = aux_data.labels.remove(&index).expect(
					"At func validation time labels are collected; Collected labels are added by index; qed",
				);
				let func_body = FuncBody {
					locals: body.locals().to_vec(),
					opcodes: body.code().clone(),
					labels: labels,
				};
				let func_id = self.alloc_func(module_id, func_type, func_body);
				instance.funcs.push(func_id);
			}
		}

		for table in module.table_section().map(|ts| ts.entries()).unwrap_or(&[]) {
			let table_id = self.alloc_table(table)?;
			instance.tables.push(table_id);
		}

		for memory in module
			.memory_section()
			.map(|ms| ms.entries())
			.unwrap_or(&[])
		{
			let memory_id = self.alloc_memory(memory)?;
			instance.memories.push(memory_id);
		}

		for global in module
			.global_section()
			.map(|gs| gs.entries())
			.unwrap_or(&[])
		{
			let init_val = eval_init_expr(global.init_expr(), instance, self);
			let global_id = self.alloc_global(global.global_type().clone(), init_val);
			instance.globals.push(global_id);
		}

		for export in module
			.export_section()
			.map(|es| es.entries())
			.unwrap_or(&[])
		{
			let field = export.field();
			let extern_val: ExternVal = match *export.internal() {
				Internal::Function(idx) => {
					let func = instance
						.funcs
						.get(idx as usize)
						.expect("Due to validation func should exists");
					ExternVal::Func(Rc::clone(func))
				}
				Internal::Global(idx) => {
					let global_id = instance
						.globals
						.get(idx as usize)
						.expect("Due to validation global should exists");
					ExternVal::Global(*global_id)
				}
				Internal::Memory(idx) => {
					let memory = instance
						.memories
						.get(idx as usize)
						.expect("Due to validation memory should exists");
					ExternVal::Memory(Rc::clone(memory))
				}
				Internal::Table(idx) => {
					let table = instance
						.tables
						.get(idx as usize)
						.expect("Due to validation table should exists");
					ExternVal::Table(Rc::clone(table))
				}
			};
			instance.exports.insert(field.into(), extern_val);
		}

		Ok(())
	}

	pub fn instantiate_module<St: 'static>(
		&mut self,
		module: &Module,
		extern_vals: &[ExternVal],
		state: &mut St,
	) -> Result<ModuleId, Error> {
		let mut instance = ModuleInstance::new();
		// Reserve the index of the module, but not yet push the module.
		let module_id = ModuleId((self.modules.len()) as u32);

		self.alloc_module_internal(module, extern_vals, &mut instance, module_id)?;

		for element_segment in module
			.elements_section()
			.map(|es| es.entries())
			.unwrap_or(&[])
		{
			let offset_val = match eval_init_expr(element_segment.offset(), &instance, self) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation elem segment offset should evaluate to i32"),
			};

			let table_inst = instance
				.tables
				.get(DEFAULT_TABLE_INDEX as usize)
				.expect("Due to validation default table should exists");
			for (j, func_idx) in element_segment.members().into_iter().enumerate() {
				let func = instance
					.funcs
					.get(*func_idx as usize)
					.expect("Due to validation funcs from element segments should exists");

				table_inst.set(offset_val + j as u32, Rc::clone(func));
			}
		}

		for data_segment in module.data_section().map(|ds| ds.entries()).unwrap_or(&[]) {
			let offset_val = match eval_init_expr(data_segment.offset(), &instance, self) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation data segment offset should evaluate to i32"),
			};

			let memory_inst = instance
				.memories
				.get(DEFAULT_MEMORY_INDEX as usize)
				.expect("Due to validation default memory should exists");
			memory_inst.set(offset_val, data_segment.value())?;
		}

		// Finally push instance to it's place
		self.modules.push(instance);

		// And run module's start function, if any
		if let Some(start_fn_idx) = module.start_section() {
			let start_func = {
				let instance = self.resolve_module(module_id);
				instance
					.funcs
					.get(start_fn_idx as usize)
					.cloned()
					.expect("Due to validation start function should exists")
			};
			self.invoke(start_func, vec![], state)?;
		}

		Ok(module_id)
	}

	pub fn add_module_instance(&mut self, instance: ModuleInstance) -> ModuleId {
		self.modules.push(instance);
		let module_id = self.modules.len() - 1;
		ModuleId(module_id as u32)
	}

	pub fn invoke<St: 'static>(
		&mut self,
		func: Rc<FuncInstance>,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		enum InvokeKind {
			Internal(FunctionContext),
			Host(Rc<AnyFunc>, Vec<RuntimeValue>),
		}

		let result = match *func {
			FuncInstance::Internal { func_type, .. } => {
				let mut args = StackWithLimit::with_data(args, DEFAULT_VALUE_STACK_LIMIT);
				let func_signature = func_type.resolve(self);
				let args = prepare_function_args(&func_signature, &mut args)?;
				let context = FunctionContext::new(
					Rc::clone(&func),
					DEFAULT_VALUE_STACK_LIMIT,
					DEFAULT_FRAME_STACK_LIMIT,
					&func_signature,
					args,
				);
				InvokeKind::Internal(context)
			}
			FuncInstance::Host { ref host_func, .. } => InvokeKind::Host(Rc::clone(host_func), args),
		};

		match result {
			InvokeKind::Internal(ctx) => {
				let mut interpreter = Interpreter::new(self, state);
				interpreter.run_function(ctx)
			}
			InvokeKind::Host(host_func, args) => {
				host_func.call_as_any(self, state as &mut Any, &args)
			}
		}
	}

	pub fn write_global(&mut self, global: GlobalId, val: RuntimeValue) -> Result<(), Error> {
		let global_instance = self.globals
			.get_mut(global.0 as usize)
			.expect("ID should be always valid");
		if !global_instance.mutable {
			// TODO: better error message
			return Err(Error::Validation("Can't write immutable global".into()));
		}
		global_instance.val = val;
		Ok(())
	}

	pub fn read_global(&self, global: GlobalId) -> RuntimeValue {
		let global_instance = self.globals
			.get(global.0 as usize)
			.expect("ID should be always valid");
		global_instance.val.clone()
	}
}

fn eval_init_expr(init_expr: &InitExpr, module: &ModuleInstance, store: &Store) -> RuntimeValue {
	let code = init_expr.code();
	debug_assert!(
		code.len() == 2,
		"Due to validation `code`.len() should be 2"
	);
	match code[0] {
		Opcode::I32Const(v) => v.into(),
		Opcode::I64Const(v) => v.into(),
		Opcode::F32Const(v) => RuntimeValue::decode_f32(v),
		Opcode::F64Const(v) => RuntimeValue::decode_f64(v),
		Opcode::GetGlobal(idx) => {
			let global_id = module
				.globals
				.get(idx as usize)
				.expect("Due to validation global should exists in module");
			let global_inst = store
				.globals
				.get(global_id.0 as usize)
				.expect("ID should always be a valid index");
			global_inst.val.clone()
		}
		_ => panic!("Due to validation init should be a const expr"),
	}
}

fn match_limits(l1: &ResizableLimits, l2: &ResizableLimits) -> Result<(), Error> {
	if l1.initial() < l2.initial() {
		return Err(Error::Initialization(format!(
			"trying to import with limits l1.initial={} and l2.initial={}",
			l1.initial(),
			l2.initial()
		)));
	}

	match (l1.maximum(), l2.maximum()) {
		(_, None) => (),
		(Some(m1), Some(m2)) if m1 <= m2 => (),
		_ => {
			return Err(Error::Initialization(format!(
				"trying to import with limits l1.max={:?} and l2.max={:?}",
				l1.maximum(),
				l2.maximum()
			)))
		}
	}

	Ok(())
}
