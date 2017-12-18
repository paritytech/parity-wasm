use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;
use std::borrow::Cow;
use elements::{FunctionType, Local, Opcodes};
use interpreter::{Error, ModuleInstance};
use interpreter::runner::{prepare_function_args, FunctionContext, Interpreter};
use interpreter::host::HostFunc;
use interpreter::value::RuntimeValue;
use common::stack::StackWithLimit;
use common::{DEFAULT_FRAME_STACK_LIMIT, DEFAULT_VALUE_STACK_LIMIT};

#[derive(Clone)]
pub enum FuncInstance<St> {
	Internal {
		func_type: Rc<FunctionType>,
		module: Rc<ModuleInstance<St>>,
		body: Rc<FuncBody>,
	},
	Host {
		func_type: Rc<FunctionType>,
		host_func: Rc<HostFunc<St>>,
	},
}

impl<St: fmt::Debug> fmt::Debug for FuncInstance<St> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&FuncInstance::Internal {
				ref func_type,
				ref module,
				..
			} => {
				write!(
					f,
					"Internal {{ type={:?}, module={:?} }}",
					func_type,
					module
				)
			}
			&FuncInstance::Host { ref func_type, .. } => {
				write!(f, "Host {{ type={:?} }}", func_type)
			}
		}
	}
}

impl<St> FuncInstance<St> {
	pub fn alloc_internal(
		module: Rc<ModuleInstance<St>>,
		func_type: Rc<FunctionType>,
		body: FuncBody,
	) -> Rc<Self> {
		let func = FuncInstance::Internal {
			func_type,
			module: module,
			body: Rc::new(body),
		};
		Rc::new(func)
	}

	pub fn alloc_host(func_type: Rc<FunctionType>, host_func: Rc<HostFunc<St>>) -> Rc<Self> {
		let func = FuncInstance::Host {
			func_type,
			host_func,
		};
		Rc::new(func)
	}

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

	pub fn invoke(
		func: Rc<FuncInstance<St>>,
		args: Cow<[RuntimeValue]>,
		state: &St,
	) -> Result<Option<RuntimeValue>, Error> {
		enum InvokeKind<'a, St> {
			Internal(FunctionContext<St>),
			Host(Rc<HostFunc<St>>, &'a [RuntimeValue]),
		}

		let result = match *func {
			FuncInstance::Internal { ref func_type, .. } => {
				let mut stack =
					StackWithLimit::with_data(args.into_iter().cloned(), DEFAULT_VALUE_STACK_LIMIT);
				let args = prepare_function_args(func_type, &mut stack)?;
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
				InvokeKind::Host(Rc::clone(host_func), &*args)
			}
		};

		match result {
			InvokeKind::Internal(ctx) => {
				let mut interpreter = Interpreter::new(state);
				interpreter.run_function(ctx)
			}
			InvokeKind::Host(host_func, args) => host_func(state, args),
		}
	}
}

#[derive(Clone, Debug)]
pub struct FuncBody {
	pub locals: Vec<Local>,
	pub opcodes: Opcodes,
	pub labels: HashMap<usize, usize>,
}
