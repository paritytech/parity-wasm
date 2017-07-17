extern crate parity_wasm;

use std::env::args;

use parity_wasm::{interpreter, ModuleInstanceInterface, RuntimeValue};
use parity_wasm::elements::{Internal, External, Type, FunctionType, ValueType};


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
    ).expect("Program instance to load");

    let module = parity_wasm::deserialize_file(&args[1]).expect("File to be deserialized");
    let execution_params = {
        let export_section = module.export_section().expect("No export section found");
        let function_section = module.function_section().expect("No function section found");
        let type_section = module.type_section().expect("No type section found");
        
        let found_entry = export_section.entries().iter()
            .find(|entry| func_name == entry.field()).expect(&format!("No export with name {} found", func_name));

        // Function index with imported functions
        let function_index: usize = match found_entry.internal() {
            &Internal::Function(index) => index as usize,
            _ => panic!("Founded export is not a function"),
        };
        let import_section_len: usize = match module.import_section() {
            Some(import) => 
                import.entries().iter().filter(|entry| match entry.external() {
                    &External::Function(_) => true,
                    _ => false,
                    }).collect::<Vec<_>>().len(),
            None => 0,
        };

        let function_index_in_section = function_index - import_section_len;
        let func_type_ref: usize = function_section.entries()[function_index_in_section].type_ref() as usize;
        let function_type: &FunctionType = match &type_section.types()[func_type_ref] {
            &Type::Function(ref func_type) => func_type,
        };

        let args: Vec<RuntimeValue> = function_type.params().iter().enumerate().map(|(i, value)| match value {
            &ValueType::I32 => RuntimeValue::I32(program_args[i].parse::<i32>().expect(&format!("Can't parse arg #{} as i32", program_args[i]))),
            &ValueType::I64 => RuntimeValue::I64(program_args[i].parse::<i64>().expect(&format!("Can't parse arg #{} as i64", program_args[i]))),
            &ValueType::F32 => RuntimeValue::F32(program_args[i].parse::<f32>().expect(&format!("Can't parse arg #{} as f32", program_args[i]))),
            &ValueType::F64 => RuntimeValue::F64(program_args[i].parse::<f64>().expect(&format!("Can't parse arg #{} as f64", program_args[i]))),
        }).collect();

        interpreter::ExecutionParams::from(args)
    };
    
    let module = program.add_module("main", module, None).expect("Failed to initialize module");

    println!("Result: {:?}", module.execute_export(func_name, execution_params).expect(""));
}
