macro_rules! run_test {
    ($label: expr, $test_name: ident, fail) => (
        #[test]
        fn $test_name() {
            ::run::failing_spec($label)
        }
    );
    ($label: expr, $test_name: ident) => (
        #[test]
        fn $test_name() {
            ::run::spec($label)
        }
    );
}

run_test!("address", wasm_address);
run_test!("address-offset-range.fail", wasm_address_offset_range_fail, fail);
run_test!("endianness", wasm_endianness);
run_test!("f32", wasm_f32);
run_test!("f32_bitwise", wasm_f32_bitwise);
run_test!("f64", wasm_f64);
run_test!("f64_bitwise", wasm_f64_bitwise);
run_test!("forward", wasm_forward);
run_test!("i32", wasm_i32);
run_test!("i64", wasm_i64);
