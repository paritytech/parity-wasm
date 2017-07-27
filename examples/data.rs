extern crate parity_wasm;

use std::env;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} somefile.wasm", args[0]);
        return;
    }

    let module = parity_wasm::deserialize_file(&args[1]).expect("Failed to load module");

    let data_section = module.data_section().expect("no data section in module");

    println!("Data segments: {}", data_section.entries().len());

    let mut index = 0;
    for entry in data_section.entries() {
        println!("  Entry #{}", index);
        println!("    init: {}", entry.offset().code()[0]);
        println!("    size: {}", entry.value().len());
        index += 1;
    }
}