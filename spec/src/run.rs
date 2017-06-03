#![cfg(test)]

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;

use serde_json;
use test;

#[test]
fn i32_tests() {
    let outdir = env::var("OUT_DIR").unwrap();
    println!("outdir {}", outdir);

    let spec_name = "i32";

    let mut wast2wasm_path = PathBuf::from(outdir.clone());
    wast2wasm_path.push("bin");
    wast2wasm_path.push("wast2wasm");

    let mut json_spec_path = PathBuf::from(outdir.clone());
    json_spec_path.push(&format!("{}.json", spec_name));

    let _output = Command::new(wast2wasm_path)
        .arg("--spec")
        .arg("-o")
        .arg(&json_spec_path)
        .arg(&format!("./testsuite/{}.wast", spec_name))
        .output()
        .expect("Failed to execute process");

    let mut f = File::open(&format!("{}/{}.json", outdir, spec_name)).unwrap();
    let commands: test::Commands = serde_json::from_reader(&mut f).unwrap();
}