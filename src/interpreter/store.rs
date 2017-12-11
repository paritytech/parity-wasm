// TODO: remove this
#![allow(unused)]

use std::sync::Arc;
use std::any::Any;
use std::collections::HashMap;
use elements::{FunctionType, GlobalEntry, GlobalType, InitExpr, Internal, Local, MemoryType,
               Module, Opcode, Opcodes, TableType, Type};
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
	pub fn memory_by_index(&self, store: &Store, idx: u32) -> Option<MemoryId> {
		store.resolve_module(*self)
			.memories
			.get(idx as usize)
			.cloned()
	}

	pub fn table_by_index(&self, store: &Store, idx: u32) -> Option<TableId> {
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

	pub fn func_by_index(&self, store: &Store, idx: u32) -> Option<FuncId> {
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
pub struct HostFuncId(u32);

#[derive(Copy, Clone, Debug)]
pub struct FuncId(u32);

impl FuncId {
	pub fn resolve<'s>(&self, store: &'s Store) -> &'s FuncInstance {
		store
			.funcs
			.get(self.0 as usize)
			.expect("ID should be always valid")
	}
}

#[derive(Copy, Clone, Debug)]
pub struct TableId(u32);

impl TableId {
	pub fn resolve<'s>(&self, store: &'s Store) -> &'s TableInstance {
		store
			.tables
			.get(self.0 as usize)
			.expect("ID should be always valid")
	}
}

#[derive(Copy, Clone, Debug)]
pub struct MemoryId(u32);

impl MemoryId {
	pub fn resolve<'s>(&self, store: &'s Store) -> &'s MemoryInstance {
		store
			.memories
			.get(self.0 as usize)
			.expect("ID should be always valid")
	}
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalId(u32);

#[derive(Copy, Clone, Debug)]
pub enum ExternVal {
	Func(FuncId),
	Table(TableId),
	Memory(MemoryId),
	Global(GlobalId),
}

pub enum FuncInstance {
	Internal {
		func_type: TypeId,
		module: ModuleId,
		body: Arc<FuncBody>,
	},
	Host {
		func_type: TypeId,
		host_func: Arc<AnyFunc>,
	},
}

impl FuncInstance {
	pub fn func_type(&self) -> TypeId {
		match *self {
			FuncInstance::Internal { func_type, .. } | FuncInstance::Host { func_type, .. } => {
				func_type
			}
		}
	}

	pub fn body(&self) -> Option<Arc<FuncBody>> {
		match *self {
			FuncInstance::Internal { ref body, .. } => Some(Arc::clone(body)),
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

#[derive(Default)]
pub struct ModuleInstance {
	types: Vec<TypeId>,
	funcs: Vec<FuncId>,
	tables: Vec<TableId>,
	memories: Vec<MemoryId>,
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

#[derive(Default)]
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

	pub fn alloc_func(&mut self, module: ModuleId, func_type: TypeId, body: FuncBody) -> FuncId {
		let func = FuncInstance::Internal {
			func_type,
			module,
			body: Arc::new(body),
		};
		self.funcs.push(func);
		let func_id = self.funcs.len() - 1;
		FuncId(func_id as u32)
	}

	pub fn alloc_host_func(&mut self, func_type: TypeId, host_func: Arc<AnyFunc>) -> FuncId {
		let func = FuncInstance::Host {
			func_type,
			host_func,
		};
		self.funcs.push(func);
		let func_id = self.funcs.len() - 1;
		FuncId(func_id as u32)
	}

	pub fn alloc_table(&mut self, table_type: &TableType) -> Result<TableId, Error> {
		let table = TableInstance::new(table_type)?;
		self.tables.push(table);
		let table_id = self.tables.len() - 1;
		Ok(TableId(table_id as u32))
	}

	pub fn alloc_memory(&mut self, mem_type: &MemoryType) -> Result<MemoryId, Error> {
		let mem = MemoryInstance::new(&mem_type)?;
		self.memories.push(mem);
		let mem_id = self.memories.len() - 1;
		Ok(MemoryId(mem_id as u32))
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

		for extern_val in extern_vals {
			match *extern_val {
				ExternVal::Func(func) => instance.funcs.push(func),
				ExternVal::Table(table) => instance.tables.push(table),
				ExternVal::Memory(memory) => instance.memories.push(memory),
				ExternVal::Global(global) => instance.globals.push(global),
			}
		}

		for type_ in module
			.type_section()
			.map(|ts| ts.types())
			.unwrap_or(&[])
			.into_iter()
			.map(|&Type::Function(ref ty)| ty)
		{
			let type_id = self.alloc_func_type(type_.clone());
			instance.types.push(type_id);
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
					let func_id = instance
						.funcs
						.get(idx as usize)
						.expect("Due to validation func should exists");
					ExternVal::Func(*func_id)
				}
				Internal::Global(idx) => {
					let global_id = instance
						.globals
						.get(idx as usize)
						.expect("Due to validation global should exists");
					ExternVal::Global(*global_id)
				}
				Internal::Memory(idx) => {
					let memory_id = instance
						.memories
						.get(idx as usize)
						.expect("Due to validation memory should exists");
					ExternVal::Memory(*memory_id)
				}
				Internal::Table(idx) => {
					let table_id = instance
						.tables
						.get(idx as usize)
						.expect("Due to validation table should exists");
					ExternVal::Table(*table_id)
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

		// TODO: assert module is valid with extern_vals.

		for element_segment in module
			.elements_section()
			.map(|es| es.entries())
			.unwrap_or(&[])
		{
			let offset_val = match eval_init_expr(element_segment.offset(), &instance, self) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation elem segment offset should evaluate to i32"),
			};

			let table_id = instance
				.tables
				.get(DEFAULT_TABLE_INDEX as usize)
				.expect("Due to validation default table should exists");
			let table_inst = self.tables
				.get_mut(table_id.0 as usize)
				.expect("ID should be always valid");

			for (j, func_idx) in element_segment.members().into_iter().enumerate() {
				let func_id = instance
					.funcs
					.get(*func_idx as usize)
					.expect("Due to validation funcs from element segments should exists");

				table_inst.set(offset_val + j as u32, *func_id);
			}
		}

		for data_segment in module.data_section().map(|ds| ds.entries()).unwrap_or(&[]) {
			let offset_val = match eval_init_expr(data_segment.offset(), &instance, self) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation data segment offset should evaluate to i32"),
			};

			let memory_id = instance
				.memories
				.get(DEFAULT_MEMORY_INDEX as usize)
				.expect("Due to validation default memory should exists");
			let memory_inst = self.memories
				.get_mut(memory_id.0 as usize)
				.expect("ID should be always valid");

			memory_inst.set(offset_val, data_segment.value())?;
		}

		// Finally push instance to it's place
		self.modules.push(instance);

		// And run module's start function, if any
		if let Some(start_fn_idx) = module.start_section() {
			let start_func = {
				let instance = self.resolve_module(module_id);
				*instance
					.funcs
					.get(start_fn_idx as usize)
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
		func: FuncId,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		enum InvokeKind {
			Internal(FunctionContext),
			Host(Arc<AnyFunc>),
		}

		let result = match *func.resolve(self) {
			FuncInstance::Internal { func_type, .. } => {
				let mut args = StackWithLimit::with_data(args, DEFAULT_VALUE_STACK_LIMIT);
				let func_signature = func_type.resolve(self);
				let args = prepare_function_args(&func_signature, &mut args)?;
				let context = FunctionContext::new(
					self,
					func,
					DEFAULT_VALUE_STACK_LIMIT,
					DEFAULT_FRAME_STACK_LIMIT,
					&func_signature,
					args,
				);
				InvokeKind::Internal(context)
			}
			FuncInstance::Host { ref host_func, .. } => InvokeKind::Host(Arc::clone(host_func)),
		};

		match result {
			InvokeKind::Internal(ctx) => {
				let mut interpreter = Interpreter::new(self, state);
				interpreter.run_function(ctx)
			}
			InvokeKind::Host(host_func) => {
				// host_func.call_as_any();
				panic!()
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
