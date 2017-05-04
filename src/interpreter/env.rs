use std::sync::Arc;
use builder::module;
use elements::{Module, FunctionType, ExportEntry, Internal, MemoryType, GlobalEntry, GlobalType,
	ValueType, InitExpr, TableType, Opcode};
use interpreter::Error;
use interpreter::module::{ModuleInstanceInterface, ItemIndex, CallerContext};
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::variable::{VariableType, VariableInstance};

const MEMORY_INDEX: u32 = 0;
const MEMORY_LIMIT_MIN: u32 = 1;

const STACKTOP_INDEX: u32 = 0;
const STACKTOP_DEFAULT: i32 = 0;
const TABLE_BASE_INDEX: u32 = 0;
const TABLE_BASE_DEFAULT: i32 = 0;

const INVOKE_VI_INDEX: u32 = 0;		// (i32, i32) -> ()
const INVOKE_INDEX: u32 = 1;		// (i32) -> ()

const TABLE_SIZE: u32 = 1024;
const TABLE_INDEX: u32 = 0;

pub struct EnvModuleInstance {
	module: Module,
	memory: Arc<MemoryInstance>,
	stacktop: Arc<VariableInstance>,
	table: Arc<TableInstance>,
}

impl EnvModuleInstance {
	pub fn new(module: Module) -> Result<Self, Error> {
		Ok(EnvModuleInstance {
			module: module,
			memory: MemoryInstance::new(&MemoryType::new(MEMORY_LIMIT_MIN, None))?,
			stacktop: Arc::new(VariableInstance::new(true, VariableType::I32, RuntimeValue::I32(STACKTOP_DEFAULT))?),
			table: TableInstance::new(VariableType::AnyFunc, &TableType::new(TABLE_SIZE, None))?,
		})
	}
}

impl ModuleInstanceInterface for EnvModuleInstance {
	fn execute_main(&self, _args: Vec<RuntimeValue>) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn execute_index(&self, _index: u32, _args: Vec<RuntimeValue>) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn execute_export(&self, _name: &str, _args: Vec<RuntimeValue>) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn module(&self) -> &Module {
		&self.module
	}

	fn table(&self, index: ItemIndex) -> Result<Arc<TableInstance>, Error> {
		match &index {
			&ItemIndex::Internal(TABLE_INDEX) => Ok(self.table.clone()),
			_ => Err(Error::Env(format!("trying to get table with index {:?} from env module", index))),
		}
	}

	fn memory(&self, index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		match &index {
			&ItemIndex::Internal(MEMORY_INDEX) => Ok(self.memory.clone()),
			_ => Err(Error::Env(format!("trying to get memory with index {:?} from env module", index))),
		}
	}

	fn global(&self, index: ItemIndex) -> Result<Arc<VariableInstance>, Error> {
		match &index {
			&ItemIndex::Internal(STACKTOP_INDEX) => Ok(self.stacktop.clone()),
			_ => Err(Error::Env(format!("trying to get global with index {:?} from env module", index))),
		}
	}

	fn call_function(&self, _outer: CallerContext, _index: ItemIndex) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn call_function_indirect(&self, _outer: CallerContext, _table_index: ItemIndex, _type_index: u32, _func_index: u32) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}

	fn call_internal_function(&self, _outer: CallerContext, _index: u32, _function_type: Option<&FunctionType>) -> Result<Option<RuntimeValue>, Error> {
		unimplemented!()
	}
}

pub fn env_module() -> Result<EnvModuleInstance, Error> {
	let module = module()
		// memory regions
		.memory().with_min(MEMORY_LIMIT_MIN).build()
		.with_export(ExportEntry::new("memory".into(), Internal::Memory(MEMORY_INDEX)))
		// tables
		.table().with_min(TABLE_SIZE).build()
		.with_export(ExportEntry::new("table".into(), Internal::Table(TABLE_INDEX)))
		// globals
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(STACKTOP_DEFAULT)])))
		.with_export(ExportEntry::new("STACKTOP".into(), Internal::Global(STACKTOP_INDEX)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const(TABLE_BASE_DEFAULT)])))
		.with_export(ExportEntry::new("tableBase".into(), Internal::Global(TABLE_BASE_INDEX)))
		// functions
		.with_export(ExportEntry::new("invoke_vi".into(), Internal::Function(INVOKE_VI_INDEX)))
		.with_export(ExportEntry::new("invoke".into(), Internal::Function(INVOKE_INDEX)))
		.build();

	EnvModuleInstance::new(module)
}
/*
  (import "env" "STACKTOP" (global (;0;) i32))
  (import "env" "invoke_vi" (func (;0;) (type 3)))
  (import "env" "invoke_v" (func (;1;) (type 1)))
  (import "env" "_storage_size" (func (;2;) (type 2)))
  (import "env" "_storage_write" (func (;3;) (type 4)))
  (import "env" "_abort" (func (;4;) (type 0)))
  (import "env" "_emscripten_memcpy_big" (func (;5;) (type 4)))
  (import "env" "___resumeException" (func (;6;) (type 1)))
  (import "env" "___cxa_find_matching_catch_2" (func (;7;) (type 2)))
  (import "env" "___gxx_personality_v0" (func (;8;) (type 6)))
  (import "env" "memory" (memory (;0;) 256 256))
  (import "env" "table" (table (;0;) 6 6 anyfunc))
  (import "env" "tableBase" (global (;1;) i32))
  (import "env" "gas" (func (;9;) (type 10)))
  (import "env" "_free" (func (;10;) (type 1)))
  (import "env" "_malloc" (func (;11;) (type 7)))
*/