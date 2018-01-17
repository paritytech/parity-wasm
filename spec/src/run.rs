#![cfg(test)]

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::collections::HashMap;

use serde_json;
use test;
use parity_wasm;
use parity_wasm::elements::{self, FunctionType, GlobalType, MemoryType, TableType, ValueType, Error as DeserializationError};
use parity_wasm::validation::{validate_module, Error as ValidationError};
use parity_wasm::interpreter::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef,
    GlobalInstance, GlobalRef, ImportResolver, ImportsBuilder,
    MemoryInstance, MemoryRef, ModuleImportResolver, ModuleInstance,
    ModuleRef, RuntimeValue, TableInstance, TableRef
};

#[derive(Debug)]
enum Error {
	Parsing(DeserializationError),
	Validation(ValidationError),
	Interpreter(InterpreterError),
}

impl From<DeserializationError> for Error {
	fn from(e: DeserializationError) -> Error {
		Error::Parsing(e)
	}
}

impl From<ValidationError> for Error {
	fn from(e: ValidationError) -> Error {
		Error::Validation(e)
	}
}

impl From<InterpreterError> for Error {
	fn from(e: InterpreterError) -> Error {
		Error::Interpreter(e)
	}
}

struct SpecModule {
    table: TableRef,
    memory: MemoryRef,
    global_i32: GlobalRef,
    global_i64: GlobalRef,
    global_f32: GlobalRef,
    global_f64: GlobalRef,
}

impl SpecModule {
    fn new() -> Self {
        SpecModule {
            table: TableInstance::alloc(10, Some(20)).unwrap(),
            memory: MemoryInstance::alloc(1, Some(2)).unwrap(),
            global_i32: GlobalInstance::alloc(RuntimeValue::I32(666), false),
            global_i64: GlobalInstance::alloc(RuntimeValue::I64(666), false),
            global_f32: GlobalInstance::alloc(RuntimeValue::F32(666.0), false),
            global_f64: GlobalInstance::alloc(RuntimeValue::F64(666.0), false),
        }
    }
}

const PRINT_FUNC_INDEX: usize = 0;

impl Externals for SpecModule {
    fn invoke_index(
        &mut self,
        index: usize,
        args: &[RuntimeValue],
    ) -> Result<Option<RuntimeValue>, InterpreterError> {
        match index {
            PRINT_FUNC_INDEX => {
                println!("print: {:?}", args);
                Ok(None)
            }
            _ => panic!("SpecModule doesn't provide function at index {}", index),
        }
    }
}

impl ModuleImportResolver for SpecModule {
    fn resolve_func(
        &self,
        field_name: &str,
        func_type: &FunctionType,
    ) -> Result<FuncRef, InterpreterError> {
        if field_name == "print" {
            if func_type.return_type().is_some() {
                return Err(InterpreterError::Instantiation(
                    "Function `print` have unit return type".into(),
                ));
            }

            let func = FuncInstance::alloc_host(func_type.clone(), PRINT_FUNC_INDEX);
            return Ok(func);
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host func import {}", field_name),
        ))
    }

    fn resolve_global(
        &self,
        field_name: &str,
        global_type: &GlobalType,
    ) -> Result<GlobalRef, InterpreterError> {
        if field_name == "global" {
            return match global_type.content_type() {
                ValueType::I32 => Ok(self.global_i32.clone()),
                ValueType::I64 => Ok(self.global_i64.clone()),
                ValueType::F32 => Ok(self.global_f32.clone()),
                ValueType::F64 => Ok(self.global_f64.clone()),
            };
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host global import {}", field_name),
        ))
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryType,
    ) -> Result<MemoryRef, InterpreterError> {
        if field_name == "memory" {
            return Ok(self.memory.clone());
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host memory import {}", field_name),
        ))
    }

    fn resolve_table(
        &self,
        field_name: &str,
        _table_type: &TableType,
    ) -> Result<TableRef, InterpreterError> {
        if field_name == "table" {
            return Ok(self.table.clone());
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host table import {}", field_name),
        ))
    }
}

struct SpecDriver {
    spec_module: SpecModule,
    instances: HashMap<String, ModuleRef>,
}

impl SpecDriver {
    fn new() -> SpecDriver {
        SpecDriver {
            spec_module: SpecModule::new(),
            instances: HashMap::new(),
        }
    }

    fn spec_module(&mut self) -> &mut SpecModule {
        &mut self.spec_module
    }

    fn add_module(&mut self, name: String, module: ModuleRef) {
        self.instances.insert(name, module);
    }

    fn module(&self, name: &str) -> Result<ModuleRef, InterpreterError> {
        self.instances.get(name).cloned().ok_or_else(|| {
            InterpreterError::Instantiation(format!("Module not registered {}", name))
        })
    }
}

impl ImportResolver for SpecDriver {
    fn resolve_func(
        &self,
        module_name: &str,
        field_name: &str,
        func_type: &FunctionType,
    ) -> Result<FuncRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_func(field_name, func_type)
        } else {
            self.module(module_name)?
                .resolve_func(field_name, func_type)
        }
    }

    fn resolve_global(
        &self,
        module_name: &str,
        field_name: &str,
        global_type: &GlobalType,
    ) -> Result<GlobalRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_global(field_name, global_type)
        } else {
            self.module(module_name)?
                .resolve_global(field_name, global_type)
        }
    }

    fn resolve_memory(
        &self,
        module_name: &str,
        field_name: &str,
        memory_type: &MemoryType,
    ) -> Result<MemoryRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_memory(field_name, memory_type)
        } else {
            self.module(module_name)?
                .resolve_memory(field_name, memory_type)
        }
    }

    fn resolve_table(
        &self,
        module_name: &str,
        field_name: &str,
        table_type: &TableType,
    ) -> Result<TableRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_table(field_name, table_type)
        } else {
            self.module(module_name)?
                .resolve_table(field_name, table_type)
        }
    }
}

fn load_module(
    base_dir: &str,
    path: &str,
    name: &Option<String>,
    spec_driver: &mut SpecDriver,
) -> ModuleRef {
    let module =
        try_deserialize(base_dir, path).expect(&format!("Wasm file {} failed to load", path));
    let validated_module = validate_module(module).expect("Validation failed");
    let instance = ModuleInstance::new(&validated_module, spec_driver)
        .expect("Instantiation failed")
        .run_start(spec_driver.spec_module())
        .expect("Run start failed");

    let module_name = name.as_ref()
        .map(|s| s.as_ref())
        .unwrap_or("wasm_test")
        .trim_left_matches('$');
    spec_driver.add_module(module_name.to_owned(), instance.clone());

    instance
}

fn try_deserialize(base_dir: &str, module_path: &str) -> Result<elements::Module, Error> {
    let mut wasm_path = PathBuf::from(base_dir.clone());
    wasm_path.push(module_path);
    Ok(parity_wasm::deserialize_file(&wasm_path)?)
}

fn try_load(
    base_dir: &str,
    module_path: &str,
    spec_driver: &mut SpecDriver,
) -> Result<(), Error> {
    let module = try_deserialize(base_dir, module_path)?;
    let validated_module = validate_module(module)?;
    let instance = ModuleInstance::new(&validated_module, &ImportsBuilder::default())?;
    instance.run_start(spec_driver.spec_module())?;
    Ok(())
}

fn runtime_value(test_val: &test::RuntimeValue) -> RuntimeValue {
    match test_val.value_type.as_ref() {
        "i32" => {
            let unsigned: u32 = test_val.value.parse().expect("Literal parse error");
            RuntimeValue::I32(unsigned as i32)
        }
        "i64" => {
            let unsigned: u64 = test_val.value.parse().expect("Literal parse error");
            RuntimeValue::I64(unsigned as i64)
        }
        "f32" => {
            let unsigned: u32 = test_val.value.parse().expect("Literal parse error");
            RuntimeValue::decode_f32(unsigned)
        }
        "f64" => {
            let unsigned: u64 = test_val.value.parse().expect("Literal parse error");
            RuntimeValue::decode_f64(unsigned)
        }
        _ => panic!("Unknwon runtime value type"),
    }
}

fn runtime_values(test_vals: &[test::RuntimeValue]) -> Vec<RuntimeValue> {
    test_vals
        .iter()
        .map(runtime_value)
        .collect::<Vec<RuntimeValue>>()
}

fn run_action(
    program: &mut SpecDriver,
    action: &test::Action,
) -> Result<Option<RuntimeValue>, InterpreterError> {
    match *action {
        test::Action::Invoke {
            ref module,
            ref field,
            ref args,
        } => {
            let module = module.clone().unwrap_or("wasm_test".into());
            let module = module.trim_left_matches('$');
            let module = program.module(&module).expect(&format!(
                "Expected program to have loaded module {}",
                module
            ));
            module.invoke_export(
                &jstring_to_rstring(field),
                &runtime_values(args),
                program.spec_module(),
            )
        }
        test::Action::Get {
            ref module,
            ref field,
            ..
        } => {
            let module = module.clone().unwrap_or("wasm_test".into());
            let module = module.trim_left_matches('$');
            let module = program.module(&module).expect(&format!(
                "Expected program to have loaded module {}",
                module
            ));
            let field = jstring_to_rstring(&field);

            let global = module
                .export_by_name(&field)
                .ok_or_else(|| {
                    InterpreterError::Global(format!("Expected to have export with name {}", field))
                })?
                .as_global()
                .ok_or_else(|| {
                    InterpreterError::Global(format!("Expected export {} to be a global", field))
                })?;
            Ok(Some(global.get()))
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
                println!(
                    "wasm2wast stdout: {}",
                    String::from_utf8_lossy(&wast2wasm_output.stdout)
                );
                println!(
                    "wasm2wast stderr: {}",
                    String::from_utf8_lossy(&wast2wasm_output.stderr)
                );
                true
            } else {
                false
            }
        },
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

    let mut f =
        File::open(&fixture.json).expect(&format!("Failed to load json file {}", &fixture.json));
    let spec: test::Spec =
        serde_json::from_reader(&mut f).expect("Failed to deserialize JSON file");

    let mut spec_driver = SpecDriver::new();
    let mut last_module = None;
    for command in &spec.commands {
        println!("command {:?}", command);
        match command {
            &test::Command::Module {
                ref name,
                ref filename,
                ..
            } => {
                last_module = Some(load_module(&outdir, &filename, &name, &mut spec_driver));
            }
            &test::Command::AssertReturn {
                line,
                ref action,
                ref expected,
            } => {
                let result = run_action(&mut spec_driver, action);
                match result {
                    Ok(result) => {
                        let spec_expected = runtime_values(expected);
                        let actual_result = result.into_iter().collect::<Vec<RuntimeValue>>();
                        for (actual_result, spec_expected) in
                            actual_result.iter().zip(spec_expected.iter())
                        {
                            assert_eq!(actual_result.value_type(), spec_expected.value_type());
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
                    }
                    Err(e) => {
                        panic!("Expected action to return value, got error: {:?}", e);
                    }
                }
            }
            &test::Command::AssertReturnCanonicalNan { line, ref action } |
            &test::Command::AssertReturnArithmeticNan { line, ref action } => {
                let result = run_action(&mut spec_driver, action);
                match result {
                    Ok(result) => {
                        for actual_result in result.into_iter().collect::<Vec<RuntimeValue>>() {
                            match actual_result {
                                RuntimeValue::F32(val) => if !val.is_nan() {
                                    panic!("Expected nan value, got {:?}", val)
                                },
                                RuntimeValue::F64(val) => if !val.is_nan() {
                                    panic!("Expected nan value, got {:?}", val)
                                },
                                val @ _ => {
                                    panic!("Expected action to return float value, got {:?}", val)
                                }
                            }
                        }
                        println!("assert_return_nan at line {} - success", line);
                    }
                    Err(e) => {
                        panic!("Expected action to return value, got error: {:?}", e);
                    }
                }
            }
            &test::Command::AssertExhaustion {
                line, ref action, ..
            } => {
                let result = run_action(&mut spec_driver, action);
                match result {
                    Ok(result) => panic!("Expected exhaustion, got result: {:?}", result),
                    Err(e) => println!("assert_exhaustion at line {} - success ({:?})", line, e),
                }
            }
            &test::Command::AssertTrap {
                line, ref action, ..
            } => {
                let result = run_action(&mut spec_driver, action);
                match result {
                    Ok(result) => {
                        panic!(
                            "Expected action to result in a trap, got result: {:?}",
                            result
                        );
                    }
                    Err(e) => {
                        println!("assert_trap at line {} - success ({:?})", line, e);
                    }
                }
            }
            &test::Command::AssertInvalid {
                line, ref filename, ..
            } |
            &test::Command::AssertMalformed {
                line, ref filename, ..
            } |
            &test::Command::AssertUnlinkable {
                line, ref filename, ..
            } => {
                let module_load = try_load(&outdir, filename, &mut spec_driver);
                match module_load {
                    Ok(_) => panic!("Expected invalid module definition, got some module!"),
                    Err(e) => println!("assert_invalid at line {} - success ({:?})", line, e),
                }
            }
            &test::Command::AssertUninstantiable {
                line, ref filename, ..
            } => match try_load(&outdir, &filename, &mut spec_driver) {
                Ok(_) => panic!("Expected error running start function at line {}", line),
                Err(e) => println!("assert_uninstantiable - success ({:?})", e),
            },
            &test::Command::Register {
                ref name,
                ref as_name,
                ..
            } => {
                match name {
                    &Some(ref name) => assert_eq!(name.trim_left_matches('$'), as_name), // we have already registered this module without $ prefix
                    &None => spec_driver.add_module(
                        as_name.clone(),
                        last_module
                            .take()
                            .expect("Last module must be set for this command"),
                    ),
                }
            }
            &test::Command::Action { line, ref action } => {
                match run_action(&mut spec_driver, action) {
                    Ok(_) => {}
                    Err(e) => panic!("Failed to invoke action at line {}: {:?}", line, e),
                }
            }
        }
    }
}

// Convert json string to correct rust UTF8 string.
// The reason is that, for example, rust character "\u{FEEF}" (3-byte UTF8 BOM) is represented as "\u00ef\u00bb\u00bf" in spec json.
// It is incorrect. Correct BOM representation in json is "\uFEFF" => we need to do a double utf8-parse here.
// This conversion is incorrect in general case (casting char to u8)!!!
fn jstring_to_rstring(jstring: &str) -> String {
    let jstring_chars: Vec<u8> = jstring.chars().map(|c| c as u8).collect();
    let rstring = String::from_utf8(jstring_chars).unwrap();
    rstring
}
