use parity_wasm::elements::{deserialize_buffer, Module};
use wabt::script::{Command, CommandKind, ScriptParser};

fn read_file(filename: &str) -> String {
	use std::fs::File;
	use std::io::prelude::*;

	let mut f = File::open(filename).expect("file not found");

	let mut contents = String::new();
	f.read_to_string(&mut contents)
		.expect("something went wrong reading the file");

	contents
}

pub fn spec(path: &str) {
	let mut parser = {
		let source = read_file(&format!("./testsuite/{}.wast", path));
		ScriptParser::<f32, f64>::from_str(&source).expect("Can't read spec script")
	};
	while let Some(Command { kind, line }) = parser.next().expect("Failed to iterate") {
		match kind {
			CommandKind::AssertMalformed { module, .. } => {
				match deserialize_buffer::<Module>(&module.into_vec()) {
					Ok(_) => panic!("Expected invalid module definition, got some module! at line {}", line),
					Err(e) => println!("assert_invalid at line {} - success ({:?})", line, e),
				}
			}
			CommandKind::Module { module, .. } => {
				match deserialize_buffer::<Module>(&module.into_vec()) {
					Ok(_) => println!("module at line {} - parsed ok", line),
					Err(e) => panic!("Valid module reported error ({:?})", e),
				}
			}
			_ => {
				// Skipping interpreted
			}
		}
	}
}
