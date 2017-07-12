extern crate parity_wasm;

use std::env::args;

use parity_wasm::{interpreter, ModuleInstanceInterface, RuntimeValue};

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() < 3 {
        println!("Usage: {} <wasm file> <exported func> [<arg>...]", args[0]);
        return;
    }
    let func_name = &args[2];
    let (_, program_args) = args.split_at(3);
    let program_args: Vec<_>  = program_args.iter().enumerate()
        .map(|(i, arg)| RuntimeValue::I32(arg.parse().expect(&format!("Invalid i32 arg at index {}", i))))
        .collect();

    let program = parity_wasm::ProgramInstance::with_env_params(
        interpreter::EnvParams {
            total_stack: 128*1024,
            total_memory: 2*1024*1024,
            allow_memory_growth: false,
        }
    ).expect("Failed to load program");
    let module = parity_wasm::deserialize_file(&args[1]).expect("Failed to load module");
    let module = program.add_module("main", module, None).expect("Failed to initialize module");

    println!("Result: {:?}", module.execute_export(func_name, program_args.into()));
}
