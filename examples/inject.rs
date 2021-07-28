extern crate parity_wasm;

use std::env;

use parity_wasm::{builder, elements};

pub fn inject_nop(instructions: &mut elements::Instructions) {
	use parity_wasm::elements::Instruction::*;
	let instructions = instructions.elements_mut();
	let mut position = 0;
	loop {
		let need_inject = matches!(&instructions[position], &Block(_) | &If(_));
		if need_inject {
			instructions.insert(position + 1, Nop);
		}

		position += 1;
		if position >= instructions.len() {
			break
		}
	}
}

fn main() {
	let args = env::args().collect::<Vec<_>>();
	if args.len() != 3 {
		println!("Usage: {} input_file.wasm output_file.wasm", args[0]);
		return
	}

	let mut module = parity_wasm::deserialize_file(&args[1]).unwrap();

	for section in module.sections_mut() {
		if let elements::Section::Code(ref mut code_section) = *section {
			for ref mut func_body in code_section.bodies_mut() {
				inject_nop(func_body.code_mut());
			}
		}
	}

	let mut build = builder::from_module(module);
	let import_sig = build.push_signature(
		builder::signature().param().i32().param().i32().result().i32().build_sig(),
	);
	let build = build.import().module("env").field("log").external().func(import_sig).build();

	parity_wasm::serialize_to_file(&args[2], build.build()).unwrap();
}
