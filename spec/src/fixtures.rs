macro_rules! run_test {
    ($label: expr, $test_name: ident) => (
        #[test]
        fn $test_name() {
            ::run::spec($label)
        }
    );
}

run_test!("br_if", wasm_br_if);
run_test!("block", wasm_block);
run_test!("i32", wasm_i32);
run_test!("address", wasm_address);