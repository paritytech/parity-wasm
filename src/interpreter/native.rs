
use std::sync::Arc;
use std::collections::HashMap;
use std::borrow::Cow;
use parking_lot::RwLock;
use elements::{Internal, ValueType};
use interpreter::Error;
use interpreter::module::{ExecutionParams, ItemIndex,
	CallerContext, ExportEntryType, InternalFunction, FunctionSignature};
use interpreter::memory::MemoryInstance;
use interpreter::table::TableInstance;
use interpreter::value::RuntimeValue;
use interpreter::variable::{VariableInstance, VariableType};

/// Min index of native function.
pub const NATIVE_INDEX_FUNC_MIN: u32 = 10001;
/// Min index of native global.
pub const NATIVE_INDEX_GLOBAL_MIN: u32 = 20001;

/// User functions executor.
pub trait UserFunctionExecutor {
	/// Execute function with given name.
	fn execute(&mut self, name: &str, context: CallerContext) -> Result<Option<RuntimeValue>, Error>;
}

/// User function descriptor
#[derive(Debug, Clone)]
pub enum UserFunctionDescriptor {
	/// Static function definition
	Static(&'static str, &'static [ValueType], Option<ValueType>),
	/// Dynamic heap function definition
	Heap(String, Vec<ValueType>, Option<ValueType>),
}

impl UserFunctionDescriptor {
	/// New function with statically known params
	pub fn statik(name: &'static str, params: &'static [ValueType], result: Option<ValueType>) -> Self {
		UserFunctionDescriptor::Static(name, params, result)
	}

	/// New function with statically unknown params
	pub fn heap(name: String, params: Vec<ValueType>, result: Option<ValueType>) -> Self {
		UserFunctionDescriptor::Heap(name, params, result)
	}

	/// Name of the function
	pub fn name(&self) -> &str {
		match self {
			&UserFunctionDescriptor::Static(name, _, _) => name,
			&UserFunctionDescriptor::Heap(ref name, _, _) => name,
		}
	}

	/// Arguments of the function
	pub fn params(&self) -> &[ValueType] {
		match self {
			&UserFunctionDescriptor::Static(_, params, _) => params,
			&UserFunctionDescriptor::Heap(_, ref params, _) => params,
		}
	}

	/// Return type of the function
	pub fn return_type(&self) -> Option<ValueType> {
		match self {
			&UserFunctionDescriptor::Static(_, _, result) => result,
			&UserFunctionDescriptor::Heap(_, _, result) => result,
		}
	}
}

/// Set of user-defined module elements.
pub struct UserDefinedElements<E: UserFunctionExecutor> {
	/// User globals list.
	pub globals: HashMap<String, Arc<VariableInstance>>,
	/// User functions list.
	pub functions: Cow<'static, [UserFunctionDescriptor]>,
	/// Functions executor.
	pub executor: Option<E>,
}

/// Native module instance.
pub struct NativeModuleInstance<E: UserFunctionExecutor> {
	/// User function executor.
	executor: RwLock<Option<E>>,
	/// By-name functions index.
	functions_by_name: HashMap<String, u32>,
	/// User functions list.
	functions: Cow<'static, [UserFunctionDescriptor]>,
	/// By-name functions index.
	globals_by_name: HashMap<String, u32>,
	/// User globals list.
	globals: Vec<Arc<VariableInstance>>,
}

impl<'a> PartialEq for UserFunctionDescriptor {
	fn eq(&self, other: &Self) -> bool {
		self.params() == other.params()
			&& self.return_type() == other.return_type()
	}
}
