#![cfg(test)]

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::sync::Arc;

use serde_json;
use test;
use parity_wasm;
use parity_wasm::interpreter::{
    RuntimeValue,
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

fn try_load(base_dir: &str, module_path: &str) -> Result<parity_wasm::elements::Module, parity_wasm::elements::Error> {
    let mut wasm_path = PathBuf::from(base_dir.clone());
    wasm_path.push(module_path);
    parity_wasm::deserialize_file(&wasm_path)   
}

fn runtime_value(test_val: &test::RuntimeValue) -> parity_wasm::RuntimeValue {
    match test_val.value_type.as_ref() {
        "i32" => {
            let unsigned: u32 = test_val.value.parse().expect("Literal parse error");
            parity_wasm::RuntimeValue::I32(unsigned as i32)
        },
        "i64" => {
            let unsigned: u64 = test_val.value.parse().expect("Literal parse error");
            parity_wasm::RuntimeValue::I64(unsigned as i64)
        },
        "f32" => {
            let unsigned: u32 = test_val.value.parse().expect("Literal parse error");
            parity_wasm::RuntimeValue::decode_f32(unsigned)
        },
        "f64" => {
            let unsigned: u64 = test_val.value.parse().expect("Literal parse error");
            parity_wasm::RuntimeValue::decode_f64(unsigned)
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

pub fn spec(name: &str) {
    let outdir = env::var("OUT_DIR").unwrap();

    let mut wast2wasm_path = PathBuf::from(outdir.clone());
    wast2wasm_path.push("bin");
    wast2wasm_path.push("wast2wasm");

    let mut json_spec_path = PathBuf::from(outdir.clone());
    json_spec_path.push(&format!("{}.json", name));

    let wast2wasm_output = Command::new(wast2wasm_path)
        .arg("--spec")
        .arg("-o")
        .arg(&json_spec_path)
        .arg(&format!("./wabt/third_party/testsuite/{}.wast", name))
        .output()
        .expect("Failed to execute process");

    if !wast2wasm_output.status.success() {
        println!("wasm2wast error code: {}", wast2wasm_output.status);
        println!("wasm2wast stdout: {}", String::from_utf8_lossy(&wast2wasm_output.stdout));
        println!("wasm2wast stderr: {}", String::from_utf8_lossy(&wast2wasm_output.stderr));
        panic!("wasm2wast exited with status {}", wast2wasm_output.status);
    }

    let mut f = File::open(&json_spec_path)
        .expect(&format!("Failed to load json file {}", &json_spec_path.to_string_lossy()));
    let spec: test::Spec = serde_json::from_reader(&mut f).expect("Failed to deserialize JSON file");

    let first_command = &spec.commands[0];
    let (mut _program, mut module) = match first_command {
        &test::Command::Module { ref filename, .. } => {
            setup_program(&outdir, filename)
        },
        _ => {
            panic!("First command supposed to specify module");
        }
    };

    for command in spec.commands.iter().skip(1) {
        println!("command {:?}", command);
        match command {
            &test::Command::Module { ref filename, .. } => {
                let (_new_program, new_module) = setup_program(&outdir, &filename);
                module = new_module;
            },
            &test::Command::AssertReturn { line, ref action, ref expected } => {
                let result = run_action(&*module, action);
                match result {
                    Ok(result) => {
                        let spec_expected = runtime_values(expected);
                        let actual_result = result.into_iter().collect::<Vec<parity_wasm::RuntimeValue>>();
                        for (actual_result, spec_expected) in actual_result.iter().zip(spec_expected.iter()) {
                            assert_eq!(actual_result.variable_type(), spec_expected.variable_type());
                            // f32::NAN != f32::NAN
                            match spec_expected {
                                &RuntimeValue::F32(val) if val.is_nan() => match actual_result {
                                    &RuntimeValue::F32(val) => assert!(val.is_nan()),
                                    _ => unreachable!(), // checked above that types are same
                                },
                                &RuntimeValue::F64(val) if val.is_nan() => match actual_result {
                                    &RuntimeValue::F64(val) => assert!(val.is_nan()),
                                    _ => unreachable!(), // checked above that types are same
                                },
                                spec_expected @ _ => assert_eq!(actual_result, spec_expected),
                            }
                        }
                        println!("assert_return at line {} - success", line);
                    },
                    Err(e) => {
                        panic!("Expected action to return value, got error: {:?}", e);
                    }
                }
            },
            &test::Command::AssertReturnCanonicalNan { line, ref action } | &test::Command::AssertReturnArithmeticNan { line, ref action } => {
                let result = run_action(&*module, action);
                match result {
                    Ok(result) => {
                        for actual_result in result.into_iter().collect::<Vec<parity_wasm::RuntimeValue>>() {
                            match actual_result {
                                RuntimeValue::F32(val) => if !val.is_nan() { panic!("Expected nan value, got {:?}", val) },
                                RuntimeValue::F64(val) => if !val.is_nan() { panic!("Expected nan value, got {:?}", val) },
                                val @ _ => panic!("Expected action to return float value, got {:?}", val),
                            }
                        }
                        println!("assert_return_nan at line {} - success", line);
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
            },
            &test::Command::AssertInvalid { line, ref filename, .. } => {
                let module_load = try_load(&outdir, filename);
                match module_load {
                    Ok(_) => {
                        panic!("Expected invalid module definition, got some module!")
                    },
                    Err(e) => {
                        println!("assert_invalid at line {} - success ({:?})", line, e)
                    }
                }
            },
            &test::Command::Action { line, ref action } => {
                match run_action(&*module, action) {
                    Ok(_) => { },
                    Err(e) => {
                        panic!("Failed to invoke action at line {}: {:?}", line, e)
                    }
                }
            },          
        }
    }
}
