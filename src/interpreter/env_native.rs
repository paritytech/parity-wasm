use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use elements::{FunctionType, Internal, ValueType, Opcode};
use interpreter::Error;
use interpreter::module::{ModuleInstanceInterface, ExecutionParams, ItemIndex,
	CallerContext, ExportEntryType, InternalFunctionReference, InternalFunction};
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::variable::{VariableInstance, VariableType};

/// Min index of native function.
pub const NATIVE_INDEX_FUNC_MIN: u32 = 10001;

/// User functions executor.
pub trait UserFunctionExecutor {
	/// Execute function with given name.
	fn execute(&mut self, name: &str, context: CallerContext) -> Result<Option<RuntimeValue>, Error>;
}

/// User function type.
pub struct UserFunction {
	/// User function name.
	pub name: String,
	/// User function parameters (for signature matching).
	pub params: Vec<ValueType>,
	/// User function return type (for signature matching).
	pub result: Option<ValueType>,
}

/// Set of user-defined functions
pub struct UserFunctions<'a> {
	/// Functions list.
	pub functions: Vec<UserFunction>,
	/// Functions executor.
	pub executor: &'a mut UserFunctionExecutor,
}

/// Native module instance.
pub struct NativeModuleInstance<'a> {
	/// Underllying module reference.
	env: Arc<ModuleInstanceInterface>,
	/// User function executor.
	executor: RwLock<&'a mut UserFunctionExecutor>,
	/// By-name functions index.
	by_name: HashMap<String, u32>,
	/// User functions list.
	functions: Vec<UserFunction>,
}

impl<'a> NativeModuleInstance<'a> {
	/// Create new native module
	pub fn new(env: Arc<ModuleInstanceInterface>, functions: UserFunctions<'a>) -> Result<Self, Error> {
		Ok(NativeModuleInstance {
			env: env,
			executor: RwLock::new(functions.executor),
			by_name: functions.functions.iter().enumerate().map(|(i, f)| (f.name.clone(), i as u32)).collect(),
			functions: functions.functions,
		})
	}
}

impl<'a> ModuleInstanceInterface for NativeModuleInstance<'a> {
	fn instantiate<'b>(&self, is_user_module: bool, externals: Option<&'b HashMap<String, Arc<ModuleInstanceInterface + 'b>>>) -> Result<(), Error> {
		self.env.instantiate(is_user_module, externals)
	}

	fn execute_index(&self, index: u32, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.env.execute_index(index, params)
	}

	fn execute_export(&self, name: &str, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.env.execute_export(name, params)
	}

	fn export_entry<'b>(&self, name: &str, externals: Option<&'b HashMap<String, Arc<ModuleInstanceInterface + 'b>>>, required_type: &ExportEntryType) -> Result<Internal, Error> {
		if let Some(index) = self.by_name.get(name) {
			return Ok(Internal::Function(NATIVE_INDEX_FUNC_MIN + *index));
		}

		self.env.export_entry(name, externals, required_type)
	}

	fn table(&self, index: ItemIndex) -> Result<Arc<TableInstance>, Error> {
		self.env.table(index)
	}

	fn memory(&self, index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		self.env.memory(index)
	}

	fn global(&self, index: ItemIndex, variable_type: Option<VariableType>) -> Result<Arc<VariableInstance>, Error> {
		self.env.global(index, variable_type)
	}

	fn function_type<'b>(&self, function_index: ItemIndex, externals: Option<&'b HashMap<String, Arc<ModuleInstanceInterface + 'b>>>) -> Result<FunctionType, Error> {
		let index = match function_index {
			ItemIndex::IndexSpace(index) | ItemIndex::Internal(index) => index,
			ItemIndex::External(_) => unreachable!("trying to call function, exported by native env module"),
		};

		if index < NATIVE_INDEX_FUNC_MIN {
			return self.env.function_type(function_index, externals);
		}

		self.functions
			.get((index - NATIVE_INDEX_FUNC_MIN) as usize)
			.ok_or(Error::Native(format!("missing native env function with index {}", index)))
			.map(|f| FunctionType::new(f.params.clone(), f.result.clone()))
	}

	fn function_type_by_index<'b>(&self, type_index: u32) -> Result<FunctionType, Error> {
		self.function_type(ItemIndex::Internal(type_index), None)
	}

	fn function_reference<'b>(&self, index: ItemIndex, externals: Option<&'b HashMap<String, Arc<ModuleInstanceInterface + 'b>>>) -> Result<InternalFunctionReference<'b>, Error> {
		self.env.function_reference(index, externals)
	}

	fn function_reference_indirect<'b>(&self, table_idx: u32, type_idx: u32, func_idx: u32, externals: Option<&'b HashMap<String, Arc<ModuleInstanceInterface + 'b>>>) -> Result<InternalFunctionReference<'b>, Error> {
		self.env.function_reference_indirect(table_idx, type_idx, func_idx, externals)
	}

	fn function_body<'b>(&'b self, _internal_index: u32) -> Result<Option<InternalFunction<'b>>, Error> {
		Ok(None)
	}

	fn call_internal_function(&self, outer: CallerContext, index: u32) -> Result<Option<RuntimeValue>, Error> {
		if index < NATIVE_INDEX_FUNC_MIN {
			return self.env.call_internal_function(outer, index);
		}

		self.functions
			.get((index - NATIVE_INDEX_FUNC_MIN) as usize)
			.ok_or(Error::Native(format!("trying to call native function with index {}", index)))
			.and_then(|f| self.executor.write().execute(&f.name, outer))
	}
}

/// Create wrapper for env module with given native user functions.
pub fn env_native_module(env: Arc<ModuleInstanceInterface>, user_functions: UserFunctions) -> Result<NativeModuleInstance, Error> {
	NativeModuleInstance::new(env, user_functions)
}
