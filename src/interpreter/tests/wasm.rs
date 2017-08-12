use elements::deserialize_file;
use elements::Module;
use interpreter::{EnvParams, ExecutionParams, DefaultProgramInstance};
use interpreter::value::RuntimeValue;
use interpreter::module::{ModuleInstanceInterface, ItemIndex};

#[test]
fn interpreter_inc_i32() {
    // Name of function contained in WASM file (note the leading underline)
    const FUNCTION_NAME: &'static str = "_inc_i32";
    // The WASM file containing the module and function
    const WASM_FILE: &str = &"res/cases/v1/inc_i32.wasm";

    let program = DefaultProgramInstance::with_env_params(
        EnvParams {
        total_stack: 128 * 1024,
        total_memory: 2 * 1024 * 1024,
        allow_memory_growth: false,
    }).expect("Failed to instanciate program");

    let module: Module =
        deserialize_file(WASM_FILE).expect("Failed to deserialize module from buffer");
    let i32_val = 42;
    // the functions expects a single i32 parameter
    let args = vec![RuntimeValue::I32(i32_val)];
    let exp_retval = Some(RuntimeValue::I32(i32_val + 1));
    let execution_params = ExecutionParams::from(args);

    let module_result = program
        .add_module("main", module, None);

    let module = module_result
        .expect("Failed to initialize module");

    let retval = module
        .execute_export(FUNCTION_NAME, execution_params)
        .expect("");
    assert_eq!(exp_retval, retval);
}

#[test]
fn interpreter_accumulate_u8() {
    // Name of function contained in WASM file (note the leading underline)
    const FUNCTION_NAME: &'static str = "_accumulate_u8";
    // The WASM file containing the module and function
    const WASM_FILE: &str = &"res/cases/v1/accumulate_u8.wasm";
    // The octet sequence being accumulated
    const BUF: &[u8] = &[9,8,7,6,5,4,3,2,1];

    // Declare the memory limits of the runtime-environment
    let program = DefaultProgramInstance::with_env_params(EnvParams {
        total_stack: 128 * 1024,
        total_memory: 2 * 1024 * 1024,
        allow_memory_growth: false,
    }).expect("Failed to instanciate program");

    // Load the module-structure from wasm-file and add to program
    let module: Module =
        deserialize_file(WASM_FILE).expect("Failed to deserialize module from buffer");
    let module = program
        .add_module("main", module, None)
        .expect("Failed to initialize module");

    // => env module is created
    let env_instance = program.module("env").unwrap();
    // => linear memory is created
    let env_memory = env_instance.memory(ItemIndex::Internal(0)).unwrap();

    // Place the octet-sequence at index 0 in linear memory
    let offset: u32 = 0;
    let _ = env_memory.set(offset, BUF);

    // Set up the function argument list and invoke the function
    let args = vec![RuntimeValue::I32(BUF.len() as i32), RuntimeValue::I32(offset as i32)];
    let execution_params = ExecutionParams::from(args);
    let retval = module
        .execute_export(FUNCTION_NAME, execution_params)
        .expect("Failed to execute function");

    // For verification, repeat accumulation using native code
    let accu = BUF.into_iter().fold(0 as i32, |a, b| a + *b as i32);
    let exp_retval: Option<RuntimeValue> = Some(RuntimeValue::I32(accu));

    // Verify calculation from WebAssembly runtime is identical to expected result
    assert_eq!(exp_retval, retval);
}
