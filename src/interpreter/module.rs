use elements::{InitExpr, Opcode, Type, FunctionType, Internal, External, ResizableLimits, Local, ValueType, BlockType};
use interpreter::Error;
use interpreter::runner::{FunctionContext};
use interpreter::VariableType;
use interpreter::RuntimeValue;
use common::stack::StackWithLimit;

/// Maximum number of entries in value stack.
const DEFAULT_VALUE_STACK_LIMIT: usize = 16384;
/// Maximum number of entries in frame stack.
const DEFAULT_FRAME_STACK_LIMIT: usize = 1024;

/// Execution context.
pub struct ExecutionParams<'a, St: 'static> {
	/// State that can be used by host functions,
	pub state: &'a mut St,
}

/// Export type.
#[derive(Debug, Clone)]
pub enum ExportEntryType {
	/// Any type.
	Any,
	/// Type of global.
	Global(VariableType),
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
