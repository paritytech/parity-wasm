extern crate parity_wasm;

use std::env::args;

use parity_wasm::{interpreter, ModuleInstanceInterface};

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 3 {
        println!("Usage: {} <wasm file> <arg>", args[0]);
        println!("    wasm file should contain exported `_call` function with single I32 argument");
        return;
    }

    let program = parity_wasm::ProgramInstance::with_env_params(
        interpreter::EnvParams {
            total_stack: 128*1024,
            total_memory: 2*1024*1024,
            allow_memory_growth: false,
        }
    ).expect("Failed to load program");
    let module = parity_wasm::deserialize_file(&args[1]).expect("Failed to load module");
    let module = program.add_module("main", module, None).expect("Failed to initialize module");
    let argument: i32 = args[2].parse().expect("Integer argument required");
    println!("Result: {:?}", module.execute_export("_call", vec![parity_wasm::RuntimeValue::I32(argument)].into()));
}
