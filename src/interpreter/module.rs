use std::collections::HashMap;
use std::iter::repeat;
use std::sync::{Arc, Weak};
use std::fmt;
use elements::{Module, InitExpr, Opcode, Type, FunctionType, Internal, External, ResizableLimits, Local, ValueType, BlockType};
use interpreter::Error;
use interpreter::native::UserFunctionDescriptor;
use interpreter::memory::MemoryInstance;
use interpreter::runner::{FunctionContext, prepare_function_args};
use interpreter::table::TableInstance;
use interpreter::value::{RuntimeValue, TryInto};
use interpreter::variable::{VariableInstance, VariableType};
use common::stack::StackWithLimit;
use interpreter::store::FuncId;

/// Maximum number of entries in value stack.
const DEFAULT_VALUE_STACK_LIMIT: usize = 16384;
/// Maximum number of entries in frame stack.
const DEFAULT_FRAME_STACK_LIMIT: usize = 1024;

/// Execution context.
#[derive(Clone)]
pub struct ExecutionParams<St: 'static> {
	/// Arguments.
	pub args: Vec<RuntimeValue>,

	/// State that can be used by host functions,
	pub state: St,
}

/// Export type.
#[derive(Debug, Clone)]
pub enum ExportEntryType<'a> {
	/// Any type.
	Any,
	/// Type of function.
	Function(FunctionSignature<'a>),
	/// Type of global.
	Global(VariableType),
}

/// Function signature.
#[derive(Debug, Clone)]
pub enum FunctionSignature<'a> {
	/// Module function reference.
	Module(&'a FunctionType),
	/// Native user function refrence.
	User(&'a UserFunctionDescriptor),
}

/// Item index in items index space.
#[derive(Debug, Clone, Copy)]
pub enum ItemIndex {
	/// Index in index space.
	IndexSpace(u32),
	/// Internal item index (i.e. index of item in items section).
	Internal(u32),
	/// External module item index (i.e. index of item in the import section).
	External(u32),
}

/// Caller context.
pub struct CallerContext<'a> {
	/// Value stack limit
	pub value_stack_limit: usize,
	/// Frame stack limit
	pub frame_stack_limit: usize,
	/// Stack of the input parameters
	pub value_stack: &'a mut StackWithLimit<RuntimeValue>,
}

/// Internal function ready for interpretation.
pub struct InternalFunction<'a> {
	/// Function locals.
	pub locals: &'a [Local],
	/// Function body.
	pub body: &'a [Opcode],
	/// Function labels.
	pub labels: &'a HashMap<usize, usize>,
}

impl<St> ExecutionParams<St> {
	/// Add argument.
	pub fn add_argument(mut self, arg: RuntimeValue) -> Self {
		self.args.push(arg);
		self
	}
}

impl<St: Default> Default for ExecutionParams<St> {
	fn default() -> Self {
		ExecutionParams {
			args: Vec::default(),
			state: St::default(),
		}
	}
}

impl<'a, St: Default> From<Vec<RuntimeValue>> for ExecutionParams<St> {
	fn from(args: Vec<RuntimeValue>) -> ExecutionParams<St> {
		ExecutionParams {
			args: args,
			state: St::default(),
		}
	}
}

impl<'a> CallerContext<'a> {
	/// Top most args
	pub fn topmost(args: &'a mut StackWithLimit<RuntimeValue>) -> Self {
		CallerContext {
			value_stack_limit: DEFAULT_VALUE_STACK_LIMIT,
			frame_stack_limit: DEFAULT_FRAME_STACK_LIMIT,
			value_stack: args,
		}
	}

	/// Nested context
	pub fn nested(outer: &'a mut FunctionContext) -> Self {
		CallerContext {
			value_stack_limit: outer.value_stack().limit() - outer.value_stack().len(),
			frame_stack_limit: outer.frame_stack().limit() - outer.frame_stack().len(),
			value_stack: &mut outer.value_stack,
		}
	}
}

pub fn check_limits(limits: &ResizableLimits) -> Result<(), Error> {
	if let Some(maximum) = limits.maximum() {
		if maximum < limits.initial() {
			return Err(Error::Validation(format!("maximum limit {} is lesser than minimum {}", maximum, limits.initial())));
		}
	}

	Ok(())
}

impl<'a> FunctionSignature<'a> {
	/// Get return type of this function.
	pub fn return_type(&self) -> Option<ValueType> {
		match self {
			&FunctionSignature::Module(ft) => ft.return_type(),
			&FunctionSignature::User(fd) => fd.return_type(),
		}
	}

	/// Get parameters of this function.
	pub fn params(&self) -> &[ValueType] {
		match self {
			&FunctionSignature::Module(ft) => ft.params(),
			&FunctionSignature::User(fd) => fd.params(),
		}
	}
}

impl<'a> PartialEq for FunctionSignature<'a> {
	fn eq<'b>(&self, other: &FunctionSignature<'b>) -> bool {
		match self {
			&FunctionSignature::Module(ft1) => match other {
				&FunctionSignature::Module(ft2) => ft1 == ft2,
				&FunctionSignature::User(ft2) => ft1.params() == ft2.params() && ft1.return_type() == ft2.return_type(),
			},
			&FunctionSignature::User(ft1) => match other {
				&FunctionSignature::User(ft2) => ft1 == ft2,
				&FunctionSignature::Module(ft2) => ft1.params() == ft2.params() && ft1.return_type() == ft2.return_type(),
			},
		}
	}
}
impl<'a> From<&'a FunctionType> for FunctionSignature<'a> {
	fn from(other: &'a FunctionType) -> Self {
		FunctionSignature::Module(other)
	}
}
