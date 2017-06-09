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
run_test!("binary", wasm_binary);
run_test!("endianness", wasm_endianness);
run_test!("f32", wasm_f32);
run_test!("f32_bitwise", wasm_f32_bitwise);
run_test!("f64", wasm_f64);
run_test!("f64_bitwise", wasm_f64_bitwise);
run_test!("forward", wasm_forward);
run_test!("i32", wasm_i32);
run_test!("i64", wasm_i64);
run_test!("memory", wasm_memory);
// TODO: fix comparison??? run_test!("names", wasm_names);
run_test!("nop", wasm_nop);
run_test!("of_string-overflow-hex-u32.fail", wasm_of_string_overflow_hex_u32_fail, fail);
run_test!("of_string-overflow-hex-u64.fail", wasm_of_string_overflow_hex_u64_fail, fail);
run_test!("of_string-overflow-s32.fail", wasm_of_string_overflow_s32_fail, fail);
run_test!("of_string-overflow-s64.fail", wasm_of_string_overflow_s64_fail, fail);
run_test!("of_string-overflow-u32.fail", wasm_of_string_overflow_u32_fail, fail);
run_test!("of_string-overflow-u64.fail", wasm_of_string_overflow_u64_fail, fail);
run_test!("resizing", wasm_resizing);
run_test!("return", wasm_return);
run_test!("select", wasm_select);
run_test!("set_local", wasm_set_local);
run_test!("skip-stack-guard-page", wasm_skip_stack_guard_page);
run_test!("stack", wasm_stack);
run_test!("start", wasm_start);
run_test!("store_retval", wasm_store_retval);
run_test!("store-align-0.fail", wasm_store_align_0_fail, fail);
run_test!("store-align-big.fail", wasm_store_align_big_fail, fail);
run_test!("store-align-odd.fail", wasm_store_align_odd_fail, fail);
run_test!("switch", wasm_switch);
run_test!("tee_local", wasm_tee_local);
run_test!("traps", wasm_traps);
run_test!("typecheck", wasm_typecheck);
run_test!("unreachable", wasm_unreachable);
run_test!("unreached-invalid", wasm_unreached_invalid);
run_test!("unwind", wasm_unwind);
run_test!("utf8-custom-selection-id", wasm_utf8_custom_selection_id);
run_test!("utf8-import-field", wasm_utf8_import_field);
run_test!("utf8-import-module", wasm_utf8_import_module);
