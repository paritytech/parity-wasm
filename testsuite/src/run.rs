use parity_wasm::elements::{deserialize_buffer, serialize, Module};
use wast::{
	parser::{parse, ParseBuffer},
	QuoteModule, Wast, WastDirective,
};

pub fn check(path: &str) {
	let path = path.strip_prefix("testsuite/").unwrap();
	let source = std::fs::read_to_string(path).unwrap();
	let buffer = ParseBuffer::new(&source).unwrap();
	let wast = parse::<Wast>(&buffer).unwrap();
	for kind in wast.directives {
		match kind {
			WastDirective::Module(mut module) => {
				let (line, _col) = module.span.linecol_in(&source);
				println!("Parsing module at line {}", line);
				let orig_bytes = module.encode().unwrap();
				let parsed =
					deserialize_buffer::<Module>(&orig_bytes).expect("Failed to parse module");
				serialize(parsed).expect("Failed to serialize module");
			},
			WastDirective::AssertMalformed {
				module: QuoteModule::Module(mut module),
				message,
				span,
			} => {
				let (line, _col) = span.linecol_in(&source);
				println!("Parsing assert_malformed at line {}", line);
				let parsed = deserialize_buffer::<Module>(&module.encode().unwrap());
				if parsed.is_ok() {
					panic!("Module should be malformed because: {}", message);
				}
			},
			_ => (),
		}
	}
}
