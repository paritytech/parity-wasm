use std::sync::Arc;
use std::collections::HashMap;

use elements::{FunctionType, Internal, ValueType};
use interpreter::Error;
use interpreter::module::{ModuleInstanceInterface, ExecutionParams, ItemIndex,
	CallerContext};
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::variable::VariableInstance;

/// Min index of native function.
pub const NATIVE_INDEX_FUNC_MIN: u32 = 10001;

/// Set of user-defined functions
pub type UserFunctions = HashMap<String, UserFunction>;

/// User function closure
pub type UserFunctionClosure = Box<UserFunctionInterface>;

/// User-defined function execution interface
pub trait UserFunctionInterface {
	/// Handles the user function invocation
	fn call(&mut self, context: CallerContext) -> Result<Option<RuntimeValue>, Error>; 
}

impl<T> UserFunctionInterface for T where T: FnMut(CallerContext) -> Result<Option<RuntimeValue>, Error> {
	fn call(&mut self, context: CallerContext) -> Result<Option<RuntimeValue>, Error> {
		(&mut *self)(context)
	}
}

/// Signature of user-defined env function
pub struct UserFunction {
	/// User function parameters (for signature matching)
	pub params: Vec<ValueType>,
	/// User function return type (for signature matching)
	pub result: Option<ValueType>,
	/// Executor of the function
	pub closure: UserFunctionClosure,
}

type UserFunctionsInternals = Vec<::std::cell::RefCell<UserFunctionClosure>>;

/// Native module instance.
pub struct NativeModuleInstance {
	env: Arc<ModuleInstanceInterface>,
	user_functions_names: HashMap<String, u32>,
	user_functions: UserFunctionsInternals,
}

impl NativeModuleInstance {
	pub fn new(env: Arc<ModuleInstanceInterface>, user_functions_names: HashMap<String, u32>, user_functions: UserFunctionsInternals) -> Result<Self, Error> {
		Ok(NativeModuleInstance {
			env: env,
			user_functions_names: user_functions_names,
			user_functions: user_functions,
		})
	}
}

impl ModuleInstanceInterface for NativeModuleInstance {
	fn execute_main(&self, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.env.execute_main(params)
	}

	fn execute_index(&self, index: u32, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.env.execute_index(index, params)
	}

	fn execute_export(&self, name: &str, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.env.execute_export(name, params)
	}

	fn export_entry(&self, name: &str) -> Result<Internal, Error> {
		if let Some(index) = self.user_functions_names.get(name) {
			return Ok(Internal::Function(*index));
		}

		self.env.export_entry(name)
	}

	fn table(&self, index: ItemIndex) -> Result<Arc<TableInstance>, Error> {
		self.env.table(index)
	}

	fn memory(&self, index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		self.env.memory(index)
	}

	fn global(&self, index: ItemIndex) -> Result<Arc<VariableInstance>, Error> {
		self.env.global(index)
	}

	fn call_function(&self, outer: CallerContext, index: ItemIndex) -> Result<Option<RuntimeValue>, Error> {
		self.env.call_function(outer, index)
	}

	fn call_function_indirect(&self, outer: CallerContext, table_index: ItemIndex, type_index: u32, func_index: u32) -> Result<Option<RuntimeValue>, Error> {
		self.env.call_function_indirect(outer, table_index, type_index, func_index)
	}

	fn call_internal_function(&self, outer: CallerContext, index: u32, function_type: Option<&FunctionType>) -> Result<Option<RuntimeValue>, Error> {
		if index < NATIVE_INDEX_FUNC_MIN {
			return self.env.call_internal_function(outer, index, function_type);
		}

		// TODO: check type
		self.user_functions
			.get((index - NATIVE_INDEX_FUNC_MIN) as usize)
			.ok_or(Error::Native(format!("trying to call native function with index {}", index)))
			.and_then(|f| f.borrow_mut().call(outer))
	}
}

/// Create wrapper for env module with given native user functions.
pub fn env_native_module(env: Arc<ModuleInstanceInterface>, user_functions: UserFunctions) -> Result<NativeModuleInstance, Error> {
	let mut funcs = user_functions;
	let mut names = HashMap::new();
	let mut internals = UserFunctionsInternals::new();
	let mut index = NATIVE_INDEX_FUNC_MIN;
	for (func_name, func) in funcs.drain() {
		internals.push(::std::cell::RefCell::new(func.closure));
		names.insert(func_name, index);
		index += 1;
	}

	NativeModuleInstance::new(env, names, internals)
}
