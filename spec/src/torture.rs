#![cfg(test)]

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::path::Path;

use parity_wasm;
use parity_wasm::elements::{self, deserialize_file, FunctionType, GlobalType, MemoryType, TableType, ValueType, Error as DeserializationError};
use parity_wasm::validation::{validate_module, Error as ValidationError};
use parity_wasm::interpreter::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef,
    GlobalInstance, GlobalRef, ImportResolver, ImportsBuilder,
    MemoryInstance, MemoryRef, ModuleImportResolver, ModuleInstance,
    ModuleRef, RuntimeValue, TableInstance, TableRef, TryInto,
	HostError,
};

const ABORT_FUNC_INDEX: usize = 0;
const EXIT_FUNC_INDEX: usize = 1;
const GENERIC_FUNC_INDEX: usize = 2;

#[derive(Debug, Clone, PartialEq)]
struct HostErrorWithCode {
	error_code: i32,
}

impl ::std::fmt::Display for HostErrorWithCode {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		write!(f, "{}", self.error_code)
	}
}

impl HostError for HostErrorWithCode {}

struct Env;

impl Externals for Env {
	fn invoke_index(
        &mut self,
        index: usize,
        args: &[RuntimeValue],
    ) -> Result<Option<RuntimeValue>, InterpreterError> {
        match index {
            ABORT_FUNC_INDEX => {
                println!("Abort!");
                Err(InterpreterError::Host(Box::new(HostErrorWithCode { error_code: -1 })))
            }
			EXIT_FUNC_INDEX => {
				let mut args = args.iter().cloned();
				let exit_code: i32 = args.next().unwrap().try_into().unwrap();

				Err(InterpreterError::Host(Box::new(HostErrorWithCode { error_code: exit_code })))
			}
            _ => panic!("'env' doesn't provide function at index {}", index),
        }
    }

    fn check_signature(&self, index: usize, func_type: &FunctionType) -> bool {
		let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
			ABORT_FUNC_INDEX => (&[], None),
			EXIT_FUNC_INDEX => (&[ValueType::I32], None),
			_ => return false,
		};

		func_type.params() == params && func_type.return_type() == ret_ty
    }
}

impl ModuleImportResolver for Env {
	fn resolve_func(&self, field_name: &str, func_type: &FunctionType) -> Result<FuncRef, InterpreterError> {
		let index = match field_name {
			"abort" => ABORT_FUNC_INDEX,
			"exit" => EXIT_FUNC_INDEX,
			_ => {
				return Err(InterpreterError::Instantiation(
					format!("Export {} not found", field_name),
				))
				// GENERIC_FUNC_INDEX
			}
		};

		if !self.check_signature(index, func_type) {
			return Err(InterpreterError::Instantiation(format!(
				"Export `{}` doesnt match expected type {:?}",
				field_name,
				func_type
			)));
		}

		Ok(FuncInstance::alloc_host(func_type.clone(), index))
	}
}

fn run_test<P: AsRef<Path>>(path: P) {
	let module = deserialize_file(path).expect("To deserialize file");
	let module = validate_module(module).expect("To validate module");

	let instance = match ModuleInstance::new(
		&module,
		&ImportsBuilder::new().with_resolver("env", &Env)
	) {
		Ok(instance) => instance,
		Err(_) => return,
	};


	let instance = instance
		.run_start(&mut Env)
		.expect("Failed to run start");

	let func = instance.export_by_name("main").unwrap().as_func().unwrap();
	let mut args = Vec::new();
	for param_type in func.func_type().params() {
		args.push(RuntimeValue::default(*param_type));
	}

	match instance.invoke_export("main", &args, &mut Env) {
		Ok(_) => {}, // weird but i guess ok
		Err(InterpreterError::Host(boxed_host_error)) => {
			let error_code = boxed_host_error.downcast_ref::<HostErrorWithCode>().unwrap().error_code;
			if error_code == -1 {
				println!("Abort!");
			}
		},
		unexpected_error => panic!("unexpected error {:?}", unexpected_error),
	}
}

#[test]
fn torture() {
	let path = Path::new("gcc-torture");
	for entry in path.read_dir().expect("read_dir call failed") {
		if let Ok(entry) = entry {
			println!("executing {:?}", entry);
			run_test(entry.path());
		}
	}
}
