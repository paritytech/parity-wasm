// TODO: remove this
#![allow(unused)]

use elements::{Module, FunctionType, FuncBody, TableType, MemoryType, GlobalType, GlobalEntry, Type, Opcode};
use interpreter::{RuntimeValue, Error};
use validation::validate_module;

#[derive(Copy, Clone, Debug)]
pub struct TypeId(u32);

#[derive(Copy, Clone, Debug)]
pub struct ModuleId(u32);

#[derive(Copy, Clone, Debug)]
pub struct HostFuncId(u32);

#[derive(Copy, Clone, Debug)]
pub struct FuncId(u32);

#[derive(Copy, Clone, Debug)]
pub struct TableId(u32);

#[derive(Copy, Clone, Debug)]
pub struct MemoryId(u32);

#[derive(Copy, Clone, Debug)]
pub struct GlobalId(u32);

pub enum ExternVal {
	Func(FuncId),
	Table(TableId),
	Memory(MemoryId),
	Global(GlobalId),
}

pub enum FuncInstance {
	Defined {
		func_type: TypeId,
		module: ModuleId,
		body: FuncBody,
	},
	Host {
		func_type: TypeId,
		host_func: HostFuncId,
	}
}

impl FuncInstance {

}

pub struct MemInstance {
	data: Vec<u8>,
	max: Option<u32>,
}

impl MemInstance {
	fn new(min: u32, max: Option<u32>) -> MemInstance {
		MemInstance {
			data: vec![0; min as usize],
			max,
		}
	}
}

pub struct TableInstance {
	elements: Vec<Option<FuncId>>,
	max: Option<u32>,
}

impl TableInstance {
	fn new(min: u32, max: Option<u32>) -> TableInstance {
		TableInstance {
			elements: vec![None; min as usize],
			max,
		}
	}
}

pub struct GlobalInstance {
	val: RuntimeValue,
	mutable: bool,
}

impl GlobalInstance {
	fn new(val: RuntimeValue, mutable: bool) -> GlobalInstance {
		GlobalInstance {
			val,
			mutable,
		}
	}
}

pub struct ExportInstance {
	name: String,
	val: ExternVal,
}

#[derive(Default)]
struct ModuleInstance {
	types: Vec<TypeId>,
	funcs: Vec<FuncId>,
	tables: Vec<TableId>,
	memories: Vec<MemoryId>,
	globals: Vec<GlobalId>,
	exports: Vec<ExportInstance>,
}

impl ModuleInstance {
	fn new() -> ModuleInstance {
		ModuleInstance::default()
	}
}

#[derive(Default)]
pub struct Store {
	// TODO: u32 capped vectors.
	funcs: Vec<FuncInstance>,
	tables: Vec<TableInstance>,
	memories: Vec<MemInstance>,
	globals: Vec<GlobalInstance>,

	// These are not the part of specification of the Store.
	// However, they can be referenced in several places, so it is handy to have it here.
	modules: Vec<ModuleInstance>,
	types: Vec<FunctionType>,
}

impl Store {
	fn new() -> Store {
		Store::default()
	}

	fn resolve_module(&self, id: ModuleId) -> &ModuleInstance {
		self.modules.get(id.0 as usize).expect("ID should always be a valid index")
	}

	fn resolve_type(&self, id: TypeId) -> &FunctionType {
		self.types.get(id.0 as usize).expect("ID should always be a valid index")
	}

	fn alloc_func_type(&mut self, func_type: FunctionType) -> TypeId {
		self.types.push(func_type);
		let type_id = self.types.len() - 1;
		TypeId(type_id as u32)
	}

	fn alloc_func(&mut self, module: ModuleId, func_type: TypeId, body: FuncBody) -> FuncId {
		let func = FuncInstance::Defined {
			func_type,
			module,
			body
		};
		self.funcs.push(func);
		let func_id = self.funcs.len() - 1;
		FuncId(func_id as u32)
	}

	// TODO: alloc_host_func

	fn alloc_table(&mut self, table_type: TableType) -> TableId {
		let table = TableInstance::new(table_type.limits().initial(), table_type.limits().maximum());
		self.tables.push(table);
		let table_id = self.tables.len() - 1;
		TableId(table_id as u32)
	}

	fn alloc_memory(&mut self, mem_type: MemoryType) -> MemoryId {
		let mem = MemInstance::new(mem_type.limits().initial(), mem_type.limits().maximum());
		self.memories.push(mem);
		let mem_id = self.memories.len() - 1;
		MemoryId(mem_id as u32)
	}

	fn alloc_global(&mut self, global_type: GlobalType, val: RuntimeValue) -> GlobalId {
		let global = GlobalInstance::new(val, global_type.is_mutable());
		self.globals.push(global);
		let global_id = self.globals.len() - 1;
		GlobalId(global_id as u32)
	}

	fn alloc_module_internal(
		&mut self,
		module: Module,
		extern_vals: &[ExternVal],
		instance: &mut ModuleInstance,
		module_id: ModuleId,
	) {
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
				"due to validation func and body counts must match"
			);

			for (ty, body) in Iterator::zip(funcs.into_iter(), bodies.into_iter()) {
				let func_type = instance.types[ty.type_ref() as usize];
				let func_id = self.alloc_func(module_id, func_type, body.clone());
				instance.funcs.push(func_id);
			}
		}

		for table in module.table_section().map(|ts| ts.entries()).unwrap_or(&[]) {
			let table_id = self.alloc_table(table.clone());
			instance.tables.push(table_id);
		}

		for memory in module
			.memory_section()
			.map(|ms| ms.entries())
			.unwrap_or(&[])
		{
			let memory_id = self.alloc_memory(memory.clone());
			instance.memories.push(memory_id);
		}

		for global in module
			.global_section()
			.map(|gs| gs.entries())
			.unwrap_or(&[])
		{
			let init_val = global_init_val(global, instance, self);
			let global_id = self.alloc_global(global.global_type().clone(), init_val);
			instance.globals.push(global_id);
		}
	}

	fn alloc_module(&mut self, module: Module, extern_vals: &[ExternVal]) -> ModuleId {
		let mut instance = ModuleInstance::new();
		// Reserve the index of the module, but not yet push the module.
		let module_id = ModuleId((self.modules.len()) as u32);

		self.alloc_module_internal(module, extern_vals, &mut instance, module_id);

		self.modules.push(instance);
		module_id
	}

	fn instantiate_module(&mut self, module: Module, extern_vals: &[ExternVal]) -> Result<(), Error> {
		validate_module(&module)?;

		// TODO: check extern_vals
		let module_id = self.alloc_module(module, extern_vals);

		Ok(())
	}
}

fn global_init_val(global: &GlobalEntry, module: &ModuleInstance, store: &Store) -> RuntimeValue {
	let code = global.init_expr().code();
	debug_assert!(
		code.len() == 2,
		"due to validation `code`.len() should be 2"
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
				.expect("due to validation global should exists in module");
			let global_inst = store
				.globals
				.get(global_id.0 as usize)
				.expect("ID should always be a valid index");
			global_inst.val.clone()
		}
		_ => panic!("due to validation init should be a const expr"),
	}
}
