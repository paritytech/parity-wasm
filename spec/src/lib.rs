#![cfg(test)]

extern crate wabt;
extern crate parity_wasm;

mod run;

macro_rules! run_test {
	($label: expr, $test_name: ident) => (
		#[test]
		fn $test_name() {
			self::run::spec($label)
		}
	);
}

run_test!("address", wasm_address);
run_test!("align", wasm_align);
run_test!("binary", wasm_binary);
run_test!("block", wasm_block);
run_test!("call_indirect", wasm_call_indirect);
run_test!("const", wasm_const);
run_test!("custom_section", wasm_custom_section);
run_test!("float_literals", wasm_float_literals);
run_test!("func", wasm_func);
run_test!("globals", wasm_globals);
run_test!("if", wasm_if);
run_test!("imports", wasm_imports);
run_test!("int_literals", wasm_int_literals);
run_test!("loop", wasm_loop);
run_test!("memory", wasm_memory);
run_test!("token", wasm_token);
run_test!("type", wasm_type);
run_test!("utf8-custom-section-id", wasm_utf8_custom_section_id);
run_test!("utf8-import-field", wasm_import_field);
run_test!("utf8-import-module", wasm_import_module);
run_test!("utf8-invalid-encoding", wasm_invalid_encoding);