extern crate parity_wasm;

use std::env::args;

use parity_wasm::{interpreter, ModuleInstanceInterface, RuntimeValue};
use parity_wasm::elements::ExportEntry;
use parity_wasm::elements::ExportSection;
use parity_wasm::elements::FunctionSection;
use parity_wasm::elements::TypeSection;
use parity_wasm::elements::Internal;
use parity_wasm::elements::Type;
use parity_wasm::elements::FunctionType;
use parity_wasm::elements::ValueType;

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() < 3 {
        println!("Usage: {} <wasm file> <exported func> [<arg>...]", args[0]);
        return;
    }
    let func_name = &args[2];
    let (_, program_args) = args.split_at(3);

    let program = parity_wasm::ProgramInstance::with_env_params(
        interpreter::EnvParams {
            total_stack: 128*1024,
            total_memory: 2*1024*1024,
            allow_memory_growth: false,
        }
    ).expect("Failed to load program");

    let module_def = parity_wasm::deserialize_file(&args[1]).expect("Failed to load module");
    let execution_params = {
        let export_section: &ExportSection = module_def.export_section().expect("No export section found");
        let function_section: &FunctionSection = module_def.function_section().expect("No function section found");
        let type_section: &TypeSection = module_def.type_section().expect("No type section found");
        let found_entry: &ExportEntry = export_section.entries().iter()
            .find(|entry| func_name.eq(entry.field())).unwrap();
        let function_index = match found_entry.internal() {
            &Internal::Function(index) => index,
            _ => panic!("Founded export is not a function"),
        };
        let func_type_ref: u32 = function_section.entries()[function_index as usize].type_ref();
        let type_: &Type = &type_section.types()[func_type_ref as usize];
        let function_type: &FunctionType = match type_ {
            &Type::Function(ref func_type) => func_type,
        };

        let args: Vec<RuntimeValue> = function_type.params().iter().enumerate().map(|(i, value)| match value {
            &ValueType::I32 => RuntimeValue::I32(program_args[i].parse::<i32>().unwrap()),
            &ValueType::I64 => RuntimeValue::I64(program_args[i].parse::<i64>().unwrap()),
            &ValueType::F32 => RuntimeValue::F32(program_args[i].parse::<f32>().unwrap()),
            &ValueType::F64 => RuntimeValue::F64(program_args[i].parse::<f64>().unwrap()),
        }).collect();

        interpreter::ExecutionParams::from(args)
    };
    
    let module_def = program.add_module("main", module_def, None).expect("Failed to initialize module");

    println!("Result: {:?}", module_def.execute_export(func_name, execution_params));
}
