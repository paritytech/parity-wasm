extern crate parity_wasm;

use std::env;

use parity_wasm::elements;

pub fn inject_nop(opcodes: &mut elements::Opcodes) {
    use parity_wasm::elements::Opcode::*;
    for opcode in opcodes.elements_mut().iter_mut() {
        match opcode {
            &mut Block(_, ref mut block) | &mut If(_, ref mut block) => {
                inject_nop(block)
            },
            _ => { }
        }
    }

    opcodes.elements_mut().insert(0, Nop);
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        println!("Usage: {} input_file.wasm output_file.wasm", args[0]);
        return;
    }

    let mut module = parity_wasm::deserialize_file(&args[1]).unwrap();

    for section in module.sections_mut() {
        match section {
            &mut elements::Section::Code(ref mut code_section) => {
                for ref mut func_body in code_section.bodies_mut() {
                    inject_nop(func_body.code_mut());
                }
            },
            _ => { }
        }
    }

    parity_wasm::serialize_to_file(&args[2], module).unwrap();
}