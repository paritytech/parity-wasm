//! This module provides some of the simplest exports
//! from the Emscripten runtime, such as `STACKTOP` or `abort`.

use std::sync::{Arc, Weak};
use std::collections::HashMap;
use builder::module;
use elements::{Module, ExportEntry, Internal, GlobalEntry, GlobalType,
	ValueType, InitExpr, Opcode, Opcodes};
use interpreter::Error;
use interpreter::native::{NATIVE_INDEX_FUNC_MIN, UserFunctionExecutor, UserDefinedElements, UserFunctionDescriptor, native_module};
use interpreter::module::{ModuleInstanceInterface, ModuleInstance, ExecutionParams,
	ItemIndex, CallerContext, ExportEntryType, InternalFunctionReference, InternalFunction, FunctionSignature};
use interpreter::memory::{MemoryInstance, LINEAR_MEMORY_PAGE_SIZE};
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::variable::{VariableInstance, VariableType};

/// Memory address, at which stack base begins.
const DEFAULT_STACK_BASE: u32 = 0;
/// Memory address, at which stack begins.
const DEFAULT_STACK_TOP: u32 = 256 * 1024;
/// Memory, allocated for stack.
const DEFAULT_TOTAL_STACK: u32 = 5 * 1024 * 1024;
/// Total memory, allocated by default.
const DEFAULT_TOTAL_MEMORY: u32 = 16 * 1024 * 1024;
/// Whether memory can be enlarged, or not.
const DEFAULT_ALLOW_MEMORY_GROWTH: bool = true;
/// Default tableBase variable value.
const DEFAULT_TABLE_BASE: u32 = 0;
/// Default tableBase variable value.
const DEFAULT_MEMORY_BASE: u32 = 0;

/// Default table size.
const DEFAULT_TABLE_SIZE: u32 = 64;

/// Index of default memory.
const INDEX_MEMORY: u32 = 0;

/// Index of default table.
const INDEX_TABLE: u32 = 0;

/// Index of STACK_BASE global variable.
const INDEX_GLOBAL_STACK_BASE: u32 = 0;
/// Index of STACK_TOP global variable.
const INDEX_GLOBAL_STACK_TOP: u32 = 1;
/// Index of STACK_MAX global variable.
const INDEX_GLOBAL_STACK_MAX: u32 = 2;
/// Index of DYNAMIC_BASE global variable.
const INDEX_GLOBAL_DYNAMIC_BASE: u32 = 3;
/// Index of DYNAMICTOP_PTR global variable.
const INDEX_GLOBAL_DYNAMICTOP_PTR: u32 = 4;
/// Index of TOTAL_MEMORY global variable.
const INDEX_GLOBAL_TOTAL_MEMORY: u32 = 5;
/// Index of ABORT global variable.
const INDEX_GLOBAL_ABORT: u32 = 6;
/// Index of EXITSTATUS global variable.
const INDEX_GLOBAL_EXIT_STATUS: u32 = 7;
/// Index of tableBase global variable.
const INDEX_GLOBAL_TABLE_BASE: u32 = 8;
/// Index of memoryBase global.
const INDEX_GLOBAL_MEMORY_BASE: u32 = 9;

/// Index of abort function.
const INDEX_FUNC_ABORT: u32 = 0;
/// Index of assert function.
const INDEX_FUNC_ASSERT: u32 = 1;
/// Index of enlargeMemory function.
const INDEX_FUNC_ENLARGE_MEMORY: u32 = 2;
/// Index of getTotalMemory function.
const INDEX_FUNC_GET_TOTAL_MEMORY: u32 = 3;
/// Min index of reserver function.
const INDEX_FUNC_MIN_NONUSED: u32 = 4;
/// Max index of reserved function.
const INDEX_FUNC_MAX: u32 = NATIVE_INDEX_FUNC_MIN - 1;

/// Emscripten environment parameters.
#[derive(Clone)]
pub struct EnvParams {
	/// Stack size in bytes.
	pub total_stack: u32,
	/// Total memory size in bytes.
	pub total_memory: u32,
	/// Allow memory growth.
	pub allow_memory_growth: bool,
	/// Table size.
	pub table_size: u32,
	/// Static reserve, if any
	pub static_size: Option<u32>,
}

struct EmscriptenFunctionExecutor {
	params: EnvParams,
}

impl<'a> UserFunctionExecutor for EmscriptenFunctionExecutor {
	 fn execute(
		 &mut self,
		 name: &str,
		 context: CallerContext,
	 ) -> Result<Option<RuntimeValue>, Error> {
		 match name {
			 _ => Err(Error::Trap("not implemented".into()).into()),
		 }
	 }
}

struct EmscriptenModuleInstance {
	_params: EnvParams,
	instance: ModuleInstance,
}

impl EmscriptenModuleInstance {
	pub fn new(params: EnvParams, module: Module) -> Result<Self, Error> {
		let mut instance = ModuleInstance::new(Weak::default(), "env".into(), module)?;
		instance.instantiate(None)?;

		Ok(EmscriptenModuleInstance {
			_params: params,
			instance: instance,
		})
	}
}

impl ModuleInstanceInterface for EmscriptenModuleInstance {
	fn execute_index(&self, index: u32, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.instance.execute_index(index, params)
	}

	fn execute_export(&self, name: &str, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.instance.execute_export(name, params)
	}

	fn export_entry<'a>(&self, name: &str, required_type: &ExportEntryType) -> Result<Internal, Error> {
		self.instance.export_entry(name, required_type)
	}

	fn table(&self, index: ItemIndex) -> Result<Arc<TableInstance>, Error> {
		self.instance.table(index)
	}

	fn memory(&self, index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		self.instance.memory(index)
	}

	fn global<'a>(&self, index: ItemIndex, variable_type: Option<VariableType>, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>) -> Result<Arc<VariableInstance>, Error> {
		self.instance.global(index, variable_type, externals)
	}

	fn function_type(&self, function_index: ItemIndex) -> Result<FunctionSignature, Error> {
		self.instance.function_type(function_index)
	}

	fn function_type_by_index(&self, type_index: u32) -> Result<FunctionSignature, Error> {
		self.instance.function_type_by_index(type_index)
	}

	fn function_reference<'a>(&self, index: ItemIndex, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>) -> Result<InternalFunctionReference<'a>, Error> {
		self.instance.function_reference(index, externals)
	}

	fn function_reference_indirect<'a>(&self, table_idx: u32, type_idx: u32, func_idx: u32, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>) -> Result<InternalFunctionReference<'a>, Error> {
		self.instance.function_reference_indirect(table_idx, type_idx, func_idx, externals)
	}

	fn function_body<'a>(&'a self, _internal_index: u32) -> Result<Option<InternalFunction<'a>>, Error> {
		Ok(None)
	}

	fn call_internal_function(&self, outer: CallerContext, index: u32) -> Result<Option<RuntimeValue>, Error> {
		// to make interpreter independent of *SCRIPTEN runtime, just make abort/assert = interpreter Error
		match index {
			INDEX_FUNC_ABORT => self.global(ItemIndex::IndexSpace(INDEX_GLOBAL_ABORT), Some(VariableType::I32), None)
				.and_then(|g| g.set(RuntimeValue::I32(1)))
				.and_then(|_| Err(Error::Trap("abort".into())))
				.map_err(Into::into),
			INDEX_FUNC_ASSERT => outer.value_stack.pop_as::<i32>()
				.and_then(|condition| if condition == 0 {
					self.global(ItemIndex::IndexSpace(INDEX_GLOBAL_ABORT), Some(VariableType::I32), None)
						.and_then(|g| g.set(RuntimeValue::I32(1)))
						.and_then(|_| Err(Error::Trap("assertion failed".into())))
				} else {
					Ok(None)
				})
				.map_err(Into::into),
			INDEX_FUNC_ENLARGE_MEMORY => Ok(Some(RuntimeValue::I32(0))), // TODO: support memory enlarge
			INDEX_FUNC_GET_TOTAL_MEMORY => self.global(ItemIndex::IndexSpace(INDEX_GLOBAL_TOTAL_MEMORY), Some(VariableType::I32), None)
				.map(|g| g.get())
				.map(Some)
				.map_err(Into::into),
			INDEX_FUNC_MIN_NONUSED ... INDEX_FUNC_MAX => Err(Error::Trap("unimplemented".into()).into()),
			_ => Err(Error::Trap(format!("trying to call function with index {} in env module", index)).into()),
		}
	}
}

pub fn env_module(params: EnvParams) -> Result<Arc<ModuleInstanceInterface>, Error> {
	debug_assert!(params.total_stack < params.total_memory);
	debug_assert!((params.total_stack % LINEAR_MEMORY_PAGE_SIZE) == 0);
	debug_assert!((params.total_memory % LINEAR_MEMORY_PAGE_SIZE) == 0);
	let builder = module()
		// memory regions
		.memory()
			.with_min(params.total_memory / LINEAR_MEMORY_PAGE_SIZE)
			.with_max(params.max_memory().map(|m| m / LINEAR_MEMORY_PAGE_SIZE))
			.build()
			.with_export(ExportEntry::new("memory".into(), Internal::Memory(INDEX_MEMORY)))
		// tables
		.table()
			.with_min(params.table_size)
			.build()
			.with_export(ExportEntry::new("table".into(), Internal::Table(INDEX_TABLE)));
	let module = builder.build();

	let mut function_executor = EmscriptenFunctionExecutor {
		params: params.clone(),
	};

	const SIGNATURES: &'static [UserFunctionDescriptor] = &[
		UserFunctionDescriptor::Static(
			"getTotalMemory",
			&[],
			Some(ValueType::I32),
		),
		UserFunctionDescriptor::Static(
			"enlargeMemory",
			&[],
			Some(ValueType::I32),
		),
		UserFunctionDescriptor::Static(
			"assert",
			&[ValueType::I32],
			None,
		),
		UserFunctionDescriptor::Static(
			"abort",
			&[],
			None,
		),
	];

	let stack_top = params.static_size.unwrap_or(DEFAULT_STACK_TOP);

	let elements = UserDefinedElements {
		executor: Some(function_executor),
		globals: vec![
			("STACK_BASE".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(stack_top as i32)).unwrap())),
			("STACKTOP".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(stack_top as i32)).unwrap())),
			("STACK_MAX".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32((stack_top + params.total_stack) as i32)).unwrap())),
			("DYNAMIC_BASE".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32((stack_top + params.total_stack) as i32)).unwrap())),
			("DYNAMICTOP_PTR".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32((stack_top + params.total_stack) as i32)).unwrap())),
			("TOTAL_MEMORY".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(params.total_memory as i32)).unwrap())),
			("ABORT".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(0)).unwrap())),
			("EXITSTATUS".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(0)).unwrap())),
			("tableBase".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(DEFAULT_TABLE_BASE as i32)).unwrap())),
			("memoryBase".into(), Arc::new(VariableInstance::new(false, VariableType::I32, RuntimeValue::I32(DEFAULT_MEMORY_BASE as i32)).unwrap())),
		].into_iter().collect(),
		functions: ::std::borrow::Cow::from(SIGNATURES),
	};

	let mut instance = ModuleInstance::new(Weak::default(), "env".into(), module)?;
	instance.instantiate(None)?;

	Ok(native_module(Arc::new(instance), elements)?)
}

impl Default for EnvParams {
	fn default() -> Self {
		EnvParams {
			total_stack: DEFAULT_TOTAL_STACK,
			total_memory: DEFAULT_TOTAL_MEMORY,
			allow_memory_growth: DEFAULT_ALLOW_MEMORY_GROWTH,
			table_size: DEFAULT_TABLE_SIZE,
			static_size: None,
		}
	}
}

impl EnvParams {
	fn max_memory(&self) -> Option<u32> {
		if self.allow_memory_growth { None } else { Some(self.total_memory) }
	}
}
