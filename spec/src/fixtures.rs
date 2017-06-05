macro_rules! run_test {
    ($label: expr, $test_name: ident) => (
        #[test]
        fn $test_name() {
            ::run::spec($label)
        }
    );
}

run_test!("i32", wasm_i32);
run_test!("f32", wasm_f32);
run_test!("f64", wasm_f64);
