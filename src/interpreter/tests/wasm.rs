use elements::deserialize_file;
use elements::Module;
use interpreter::EnvParams;
use interpreter::ExecutionParams;
use interpreter::module::ModuleInstanceInterface;
use interpreter::program::ProgramInstance;
use interpreter::value::RuntimeValue;

// Name of function contained in WASM file (note the leading underline)
const FUNCTION_NAME: &'static str = "_inc_i32";

// The WASM file containing the module and function
const WASM_FILE: &str = &"res/cases/v1/inc_i32.wasm";

#[test]
fn interpreter_inc_i32() {
    let program = ProgramInstance::with_env_params(EnvParams {
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
    let module = program
        .add_module("main", module, None)
        .expect("Failed to initialize module");
    let retval = module
        .execute_export(FUNCTION_NAME, execution_params)
        .expect("");
    assert_eq!(exp_retval, retval);
}
