#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate parity_wasm;
extern crate mktemp;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn wasm_opt() -> PathBuf {
    let bin = PathBuf::from(env!("OUT_DIR")).join("bin").join("wasm-opt");
    assert!(
        bin.exists(),
        format!(
            "could not find wasm-opt at location installed by build.rs: {:?}",
            wasm_opt()
        )
    );
    bin
}

fuzz_target!(|data: &[u8]| {
    let seed = mktemp::Temp::new_file().expect("mktemp file to store fuzzer input");
    let mut seedfile =
        File::create(seed.as_ref()).expect("open temporary file for writing to store fuzzer input");
    seedfile.write_all(data).expect(
        "write fuzzer input to temporary file",
    );
    seedfile.flush().expect(
        "flush fuzzer input to temporary file before starting wasm-opt",
    );

    let wasm = mktemp::Temp::new_file().expect("mktemp file to store wasm-opt output");
    let opt_fuzz = Command::new(wasm_opt())
        .arg("--translate-to-fuzz")
        .arg(seed.as_ref())
        .arg("-o")
        .arg(wasm.as_ref())
        .output()
        .expect("execute wasm-opt installed by build.rs");

    assert!(
        opt_fuzz.status.success(),
        format!(
            "wasm-opt failed with: {}",
            String::from_utf8_lossy(&opt_fuzz.stderr)
        )
    );

    let _module: parity_wasm::elements::Module = parity_wasm::deserialize_file(wasm.as_ref())
        .expect(
            "deserialize output of wasm-opt, indicating possible bug in deserializer",
        );
});
