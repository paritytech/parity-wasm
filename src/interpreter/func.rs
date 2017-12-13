use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;
use std::any::Any;
use elements::{FunctionType, Opcodes, Local};
use interpreter::{Error, ModuleInstance};
use interpreter::runner::{prepare_function_args, FunctionContext, Interpreter};
use interpreter::host::AnyFunc;
use interpreter::value::RuntimeValue;
use common::stack::StackWithLimit;
use common::{DEFAULT_FRAME_STACK_LIMIT, DEFAULT_VALUE_STACK_LIMIT};

#[derive(Clone)]
pub enum FuncInstance {
	Internal {
		func_type: Rc<FunctionType>,
		module: Rc<ModuleInstance>,
		body: Rc<FuncBody>,
	},
	Host {
		func_type: Rc<FunctionType>,
		host_func: Rc<AnyFunc>,
	},
}

impl fmt::Debug for FuncInstance {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&FuncInstance::Internal {
				ref func_type,
				ref module,
				..
			} => write!(
				f,
				"Internal {{ type={:?}, module={:?} }}",
				func_type,
				module
			),
			&FuncInstance::Host { ref func_type, .. } => {
				write!(f, "Host {{ type={:?} }}", func_type)
			}
		}
	}
}

impl FuncInstance {
	pub fn func_type(&self) -> Rc<FunctionType> {
		match *self {
			FuncInstance::Internal { ref func_type, .. } |
			FuncInstance::Host { ref func_type, .. } => Rc::clone(func_type),
		}
	}

	pub fn body(&self) -> Option<Rc<FuncBody>> {
		match *self {
			FuncInstance::Internal { ref body, .. } => Some(Rc::clone(body)),
			FuncInstance::Host { .. } => None,
		}
	}
}

impl FuncInstance {
	pub fn invoke<St: 'static>(
		func: Rc<FuncInstance>,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		enum InvokeKind {
			Internal(FunctionContext),
			Host(Rc<AnyFunc>, Vec<RuntimeValue>),
		}

		let result = match *func {
			FuncInstance::Internal { ref func_type, .. } => {
				let mut args = StackWithLimit::with_data(args, DEFAULT_VALUE_STACK_LIMIT);
				let args = prepare_function_args(func_type, &mut args)?;
				let context = FunctionContext::new(
					Rc::clone(&func),
					DEFAULT_VALUE_STACK_LIMIT,
					DEFAULT_FRAME_STACK_LIMIT,
					func_type,
					args,
				);
				InvokeKind::Internal(context)
			}
			FuncInstance::Host { ref host_func, .. } => {
				InvokeKind::Host(Rc::clone(host_func), args)
			}
		};

		match result {
			InvokeKind::Internal(ctx) => {
				let mut interpreter = Interpreter::new(state);
				interpreter.run_function(ctx)
			}
			InvokeKind::Host(host_func, args) => host_func.call_as_any(state as &mut Any, &args),
		}
	}
}

#[derive(Clone, Debug)]
pub struct FuncBody {
	pub locals: Vec<Local>,
	pub opcodes: Opcodes,
	pub labels: HashMap<usize, usize>,
}
