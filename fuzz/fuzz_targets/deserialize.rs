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
        format!("could not find wasm-opt at: {:?}", wasm_opt())
    );
    bin
}

fuzz_target!(|data: &[u8]| {
    let seed = mktemp::Temp::new_file().unwrap();
    let mut seedfile = File::create(seed.as_ref()).unwrap();
    seedfile.write_all(data).unwrap();
    seedfile.flush().unwrap();

    let wasm = mktemp::Temp::new_file().unwrap();
    let opt_fuzz = Command::new(wasm_opt())
        .arg("--translate-to-fuzz")
        .arg(seed.as_ref())
        .arg("-o")
        .arg(wasm.as_ref())
        .output()
        .unwrap();

    assert!(
        opt_fuzz.status.success(),
        format!(
            "wasm-opt failed with: {}",
            String::from_utf8_lossy(&opt_fuzz.stderr)
        )
    );

    let _module: parity_wasm::elements::Module = parity_wasm::deserialize_file(wasm.as_ref())
        .unwrap();
});
