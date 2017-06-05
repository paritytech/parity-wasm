macro_rules! run_test {
    ($label: expr, $test_name: ident) => (
        #[test]
        fn $test_name() {
            ::run::spec($label)
        }
    );
}

run_test!("i32", wasm_i32);
run_test!("endianness", wasm_endianness);
run_test!("i64", wasm_i64);
