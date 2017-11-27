#![feature(test)]

extern crate test;
extern crate parity_wasm;

use test::Bencher;
use parity_wasm::builder::module;
use parity_wasm::elements::{ExportEntry, Internal, ImportEntry, External, Opcodes, Opcode};
use parity_wasm::interpreter::{ProgramInstance, ModuleInstanceInterface, RuntimeValue};

#[bench]
fn export_entry_performance(b: &mut Bencher) {
	// create module with 1000 functions
	const NUM_FUNCTIONS: u32 = 1000;
	let mut callee_module = module();
	for i in 0..NUM_FUNCTIONS {
		callee_module = callee_module
			.with_export(ExportEntry::new(format!("func{}", i), Internal::Function(i)))
			.function()
				.signature().return_type().i32().build()
				.body().with_opcodes(Opcodes::new(vec![
					Opcode::I32Const(i as i32),
					Opcode::End,
				])).build()
				.build();
	}
	let callee_module = callee_module.build();

	// create module which call one of 1000 functions
	let caller_module = module()
		.with_import(ImportEntry::new("callee_module".into(), "func500".into(), External::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::Call(0),
				Opcode::I32Const(1000),
				Opcode::I32Add,
				Opcode::End,
			])).build()
			.build()
		.build();

	// add both modules to program
	let program = ProgramInstance::new();
	program.add_module("callee_module", callee_module, None).unwrap();
	let caller_module = program.add_module("caller_module", caller_module, None).unwrap();

	// run bench
	b.iter(||
		assert_eq!(caller_module.execute_index(1, vec![].into()).unwrap(), Some(RuntimeValue::I32(1500)))
	);

	// test export_entry_performance ... bench:       3,497 ns/iter (+/- 200)
}
