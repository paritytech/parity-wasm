extern crate parity_wasm;

use std::env::args;

use parity_wasm::elements::{Internal, External, Type, FunctionType, ValueType};
use parity_wasm::interpreter::{RuntimeValue, ModuleInstance, NopExternals, ImportsBuilder};
use parity_wasm::validation::validate_module;


fn main() {
    let args: Vec<_> = args().collect();
    if args.len() < 3 {
        println!("Usage: {} <wasm file> <exported func> [<arg>...]", args[0]);
        return;
    }
    let func_name = &args[2];
    let (_, program_args) = args.split_at(3);

    let module = parity_wasm::deserialize_file(&args[1]).expect("File to be deserialized");

    // Extracts call arguments from command-line arguments
    let args = {
        // Export section has an entry with a func_name with an index inside a module
        let export_section = module.export_section().expect("No export section found");
        // It's a section with function declarations (which are references to the type section entries)
        let function_section = module.function_section().expect("No function section found");
        // Type section stores function types which are referenced by function_section entries
        let type_section = module.type_section().expect("No type section found");

        // Given function name used to find export section entry which contains
        // an `internal` field which points to the index in the function index space
        let found_entry = export_section.entries().iter()
            .find(|entry| func_name == entry.field()).expect(&format!("No export with name {} found", func_name));

        // Function index in the function index space (internally-defined + imported)
        let function_index: usize = match found_entry.internal() {
            &Internal::Function(index) => index as usize,
            _ => panic!("Founded export is not a function"),
        };

        // We need to count import section entries (functions only!) to subtract it from function_index
        // and obtain the index within the function section
        let import_section_len: usize = match module.import_section() {
            Some(import) =>
                import.entries().iter().filter(|entry| match entry.external() {
                    &External::Function(_) => true,
                    _ => false,
                    }).count(),
            None => 0,
        };

        // Calculates a function index within module's function section
        let function_index_in_section = function_index - import_section_len;

        // Getting a type reference from a function section entry
        let func_type_ref: usize = function_section.entries()[function_index_in_section].type_ref() as usize;

        // Use the reference to get an actual function type
        let function_type: &FunctionType = match &type_section.types()[func_type_ref] {
            &Type::Function(ref func_type) => func_type,
        };

        // Parses arguments and constructs runtime values in correspondence of their types
        function_type.params().iter().enumerate().map(|(i, value)| match value {
            &ValueType::I32 => RuntimeValue::I32(program_args[i].parse::<i32>().expect(&format!("Can't parse arg #{} as i32", program_args[i]))),
            &ValueType::I64 => RuntimeValue::I64(program_args[i].parse::<i64>().expect(&format!("Can't parse arg #{} as i64", program_args[i]))),
            &ValueType::F32 => RuntimeValue::F32(program_args[i].parse::<f32>().expect(&format!("Can't parse arg #{} as f32", program_args[i]))),
            &ValueType::F64 => RuntimeValue::F64(program_args[i].parse::<f64>().expect(&format!("Can't parse arg #{} as f64", program_args[i]))),
        }).collect::<Vec<RuntimeValue>>()
    };

    let validated_module = validate_module(module).expect("Module to be valid");

    // Intialize deserialized module. It adds module into It expects 3 parameters:
    // - a name for the module
    // - a module declaration
    // - "main" module doesn't import native module(s) this is why we don't need to provide external native modules here
    // This test shows how to implement native module https://github.com/NikVolf/parity-wasm/blob/master/src/interpreter/tests/basics.rs#L197
    let main = ModuleInstance::new(&validated_module, &ImportsBuilder::default())
        .expect("Failed to instantiate module")
        .run_start(&mut NopExternals)
        .expect("Failed to run start function in module");

    println!("Result: {:?}", main.invoke_export(func_name, &args, &mut NopExternals).expect(""));
}
