#![cfg(test)]

use std::{self, env};
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::sync::Arc;

use serde_json;
use test;
use parity_wasm;
use parity_wasm::interpreter::{
    ProgramInstance, ModuleInstance, ModuleInstanceInterface, 
    Error as InterpreterError,
};

fn setup_program(base_dir: &str, test_module_path: &str) -> (ProgramInstance, Arc<ModuleInstance>) {
    let mut wasm_path = PathBuf::from(base_dir.clone());
    wasm_path.push(test_module_path);
    let module = parity_wasm::deserialize_file(&wasm_path)
        .expect(&format!("Wasm file {} failed to load", wasm_path.to_string_lossy()));
	let program = ProgramInstance::new().expect("Failed creating program");
	let module_instance = program.add_module("test", module).expect("Failed adding module");
    (program, module_instance)
}

fn runtime_value(test_val: &test::RuntimeValue) -> parity_wasm::RuntimeValue {
    match test_val.value_type.as_ref() {
        "i32" => {
            let unsigned: u32 = test_val.value.parse().expect("Literal parse error");
            parity_wasm::RuntimeValue::I32(unsigned as i32)
        },
        _ => panic!("Unknwon runtime value type"),
    }
}

fn runtime_values(test_vals: &[test::RuntimeValue]) -> Vec<parity_wasm::RuntimeValue> {
    test_vals.iter().map(runtime_value).collect::<Vec<parity_wasm::RuntimeValue>>()
}

fn run_action(module: &ModuleInstance, action: &test::Action) 
    -> Result<Option<parity_wasm::RuntimeValue>, InterpreterError> 
{
    match *action {
        test::Action::Invoke { ref field, ref args} => {
            module.execute_export(field, runtime_values(args).into())
        }
    }
}

fn run_spec(spec_name: &str) {
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
    let spec: test::Spec = serde_json::from_reader(&mut f).unwrap();

    let first_command = &spec.commands[0];
    let (mut program, mut module) = match first_command {
        &test::Command::Module { line, ref filename } => {
            setup_program(&outdir, filename)
        },
        _ => {
            panic!("First command supposed to specify module");
        }
    };

    for command in spec.commands.iter().skip(1) {
        match command {
            &test::Command::Module { line, ref filename } => {
                let (new_program, new_module) = setup_program(&outdir, &filename);
                program = new_program;
                module = new_module;
            },
            &test::Command::AssertReturn { line, ref action, ref expected } => {
                let result = run_action(&*module, action);
                match result {
                    Ok(result) => {
                        let spec_expected = runtime_values(expected);
                        let actual_result = result.into_iter().collect::<Vec<parity_wasm::RuntimeValue>>();
                        assert_eq!(actual_result, spec_expected);
                        println!("assert_return at line {} - success", line);
                    },
                    Err(e) => {
                        panic!("Expected action to return value, got error: {:?}", e);
                    }
                }
            },
            &test::Command::AssertTrap { line, ref action, .. } => {
                let result = run_action(&*module, action);
                match result {
                    Ok(result) => {
                        panic!("Expected action to result in a trap, got result: {:?}", result);
                    },
                    Err(e) => {
                        println!("assert_trap at line {} - success ({:?})", line, e);                    
                    }
                }
            }
        }
    }
}

#[test]
fn wast_i32() {
    run_spec("i32");
}