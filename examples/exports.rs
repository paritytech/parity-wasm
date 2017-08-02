extern crate parity_wasm;

use std::env::args;

use parity_wasm::elements::{Internal, External, Type, FunctionType, Module};

fn type_by_index(module: &Module, index: usize) -> FunctionType {
    let function_section = module.function_section().expect("No function section found");
    let type_section = module.type_section().expect("No type section found");

    let import_section_len: usize = match module.import_section() {
            Some(import) => 
                import.entries().iter().filter(|entry| match entry.external() {
                    &External::Function(_) => true,
                    _ => false,
                    }).count(),
            None => 0,
        };
    let function_index_in_section = index - import_section_len;
    let func_type_ref: usize = function_section.entries()[function_index_in_section].type_ref() as usize;
    match type_section.types()[func_type_ref] {
        Type::Function(ref func_type) => func_type.clone(),
    }
}

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() < 2 {
        println!("Prints export function names with and their types");
        println!("Usage: {} <wasm file>", args[0]);
        return;
    }
    let module = parity_wasm::deserialize_file(&args[1]).expect("File to be deserialized");
    let export_section = module.export_section().expect("No export section found");
    let exports: Vec<String> = export_section.entries().iter()
        .filter_map(|entry|
            match *entry.internal() {
                Internal::Function(index) => Some((entry.field(), index as usize)),
                _ => None
            })
        .map(|(field, index)| format!("{:}: {:?}", field, type_by_index(&module, index).params())).collect();
    for export in exports {
        println!("{:}", export);
    }
}
