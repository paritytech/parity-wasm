#![cfg(test)]

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::sync::Arc;

use serde_json;
use test;
use parity_wasm::{self, elements, builder};
use parity_wasm::interpreter::{
    RuntimeValue,
    ProgramInstance, ModuleInstance, ModuleInstanceInterface, 
    Error as InterpreterError,
};

fn spec_test_module() -> elements::Module {
    builder::module()
        .function()
            .signature().build()
            .body().build()
            .build()
        .global().value_type().i32().init_expr(elements::Opcode::I32Const(0)).build()
        .export().field("print").internal().func(0).build()
        .export().field("global").internal().global(0).build()
        .build()
}

fn setup_program(base_dir: &str, test_module_path: &str) -> (ProgramInstance, Arc<ModuleInstance>) {
    let mut wasm_path = PathBuf::from(base_dir.clone());
    wasm_path.push(test_module_path);
    let module = parity_wasm::deserialize_file(&wasm_path)
        .expect(&format!("Wasm file {} failed to load", wasm_path.to_string_lossy()));

	let program = ProgramInstance::new().expect("Failed creating program");
    program.add_module("spectest", spec_test_module()).expect("Failed adding 'spectest' module");

	let module_instance = program.add_module("test", module).expect("Failed adding module");
    (program, module_instance)
}

fn try_load(base_dir: &str, module_path: &str) -> Result<(), parity_wasm::interpreter::Error> {
    let mut wasm_path = PathBuf::from(base_dir.clone());
    wasm_path.push(module_path);
    let module = parity_wasm::deserialize_file(&wasm_path).map_err(|e| parity_wasm::interpreter::Error::Program(format!("{:?}", e)))?;

    let program = ProgramInstance::new().expect("Failed creating program");
    program.add_module("try_load", module).map(|_| ())
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

pub struct FixtureParams {
    failing: bool,
    json: String,
}

pub fn run_wast2wasm(name: &str) -> FixtureParams {
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

    FixtureParams {
        json: json_spec_path.to_str().unwrap().to_owned(),
        failing: {
            if !wast2wasm_output.status.success() {
                println!("wasm2wast error code: {}", wast2wasm_output.status);
                println!("wasm2wast stdout: {}", String::from_utf8_lossy(&wast2wasm_output.stdout));
                println!("wasm2wast stderr: {}", String::from_utf8_lossy(&wast2wasm_output.stderr));
                true
            } else {
                false
            }     
        }
    }
}

pub fn failing_spec(name: &str) {
    let fixture = run_wast2wasm(name);
    if !fixture.failing {
         panic!("wasm2wast expected to fail, but terminated normally");
    }
}

pub fn spec(name: &str) {
    let outdir = env::var("OUT_DIR").unwrap();

    let fixture = run_wast2wasm(name);
    if fixture.failing {
         panic!("wasm2wast terminated abnormally, expected to success");        
    }

    let mut f = File::open(&fixture.json)
        .expect(&format!("Failed to load json file {}", &fixture.json));
    let spec: test::Spec = serde_json::from_reader(&mut f).expect("Failed to deserialize JSON file");

    let mut _program = None;
    let mut module = None;
    for command in &spec.commands {
        println!("command {:?}", command);
        match command {
            &test::Command::Module { ref filename, .. } => {
                let (new_program, new_module) = setup_program(&outdir, &filename);
                _program = Some(new_program);
                module = Some(new_module);
            },
            &test::Command::AssertReturn { line, ref action, ref expected } => {
                let result = run_action(&*module.as_ref().unwrap(), action);
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
                let result = run_action(&*module.as_ref().unwrap(), action);
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
            &test::Command::AssertExhaustion { line, ref action, .. } => {
                let result = run_action(&*module.as_ref().unwrap(), action);
                match result {
                    Ok(result) => panic!("Expected exhaustion, got result: {:?}", result),
                    Err(e) => println!("assert_exhaustion at line {} - success ({:?})", line, e),
                }
            },
            &test::Command::AssertTrap { line, ref action, .. } => {
                let result = run_action(&*module.as_ref().unwrap(), action);
                match result {
                    Ok(result) => {
                        panic!("Expected action to result in a trap, got result: {:?}", result);
                    },
                    Err(e) => {
                        println!("assert_trap at line {} - success ({:?})", line, e);                    
                    }
                }
            },
            &test::Command::AssertInvalid { line, ref filename, .. }
            | &test::Command::AssertMalformed { line, ref filename, .. }
            | &test::Command::AssertUnlinkable { line, ref filename, .. }
                => {
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
            &test::Command::AssertUninstantiable { line, ref filename, .. } => {
                match try_load(&outdir, &filename) {
                    Ok(_) => panic!("Expected error running start function at line {}", line),
                    Err(e) => println!("assert_uninstantiable - success ({:?})", e),
                }
            },
            &test::Command::Action { line, ref action } => {
                match run_action(&*module.as_ref().unwrap(), action) {
                    Ok(_) => { },
                    Err(e) => {
                        panic!("Failed to invoke action at line {}: {:?}", line, e)
                    }
                }
            },          
        }
    }
}
