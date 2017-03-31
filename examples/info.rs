extern crate parity_wasm;

use std::env;
use parity_wasm::Section;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} somefile.wasm", args[0]);
        return;
    }

    let module = parity_wasm::deserialize_file(&args[1]).expect("Failed to load module");

    println!("Module sections: {}", module.sections().len());

    for section in module.sections() {
        match section {
            &Section::Import(ref import_section) => {
                println!("  Imports: {}", import_section.entries().len());
            },
            &Section::Export(ref exports_section) => {
                println!("  Exports: {}", exports_section.entries().len());
            },            
            &Section::Function(ref functions_section) => {
                println!("  Functions: {}", functions_section.entries().len());
            },
            &Section::Global(ref globals_section) => {
                println!("  Globals: {}", globals_section.entries().len());                
            },
            _ => {},
        }
    }
}