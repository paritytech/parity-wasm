use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use elements::Module;
use interpreter::Error;
use interpreter::emscripten::{self, env_module};
use interpreter::module::{ModuleInstance, ModuleInstanceInterface};

/// Program instance. Program is a set of instantiated modules.
pub struct ProgramInstance {
	/// Shared data reference.
	essence: Arc<ProgramInstanceEssence>,
}

/// Program instance essence.
pub struct ProgramInstanceEssence {
	/// Loaded modules.
	modules: RwLock<HashMap<String, Arc<ModuleInstanceInterface>>>,
}

impl ProgramInstance {
	/// Create new program instance.
	pub fn new() -> Result<Self, Error> {
		ProgramInstance::with_env_params(emscripten::EnvParams::default())
	}

	/// Create new program instance with custom env module params (mostly memory)
	pub fn with_env_params(params: emscripten::EnvParams) -> Result<Self, Error> {
		Ok(ProgramInstance {
			essence: Arc::new(ProgramInstanceEssence::with_env_params(params)?),
		})
	}

	/// Create a new program instance with a custom env module
	pub fn with_env_module(env_module: Arc<ModuleInstanceInterface>) -> Self {
		ProgramInstance {
			essence: Arc::new(ProgramInstanceEssence::with_env_module(env_module)),
		}
	}

	/// Instantiate module with validation.
	pub fn add_module<'a>(&self, name: &str, module: Module, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>) -> Result<Arc<ModuleInstance>, Error> {
		let mut module_instance = ModuleInstance::new(Arc::downgrade(&self.essence), name.into(), module)?;
		module_instance.instantiate(externals)?;

		let module_instance = Arc::new(module_instance);
		self.essence.modules.write().insert(name.into(), module_instance.clone());
		module_instance.run_start_function()?;
		Ok(module_instance)
	}

	/// Insert instantiated module.
	pub fn insert_loaded_module(&self, name: &str, module_instance: Arc<ModuleInstance>) -> Result<Arc<ModuleInstance>, Error> {
		// replace existing module with the same name with new one
		self.essence.modules.write().insert(name.into(), module_instance.clone());
		Ok(module_instance)
	}

	/// Get one of the modules by name
	pub fn module(&self, name: &str) -> Option<Arc<ModuleInstanceInterface>> {
		self.essence.module(name)
	}
}

impl ProgramInstanceEssence {
	/// Create new program essence.
	pub fn new() -> Result<Self, Error> {
		ProgramInstanceEssence::with_env_params(emscripten::EnvParams::default())
	}

	pub fn with_env_params(env_params: emscripten::EnvParams) -> Result<Self, Error> {
		let env_mod = env_module(env_params)?;
		Ok(ProgramInstanceEssence::with_env_module(Arc::new(env_mod)))
	}

	pub fn with_env_module(env_module: Arc<ModuleInstanceInterface>) -> Self {
		let mut modules = HashMap::new();
		modules.insert("env".into(), env_module);
		ProgramInstanceEssence {
			modules: RwLock::new(modules),
		}
	}

	/// Get module reference.
	pub fn module(&self, name: &str) -> Option<Arc<ModuleInstanceInterface>> {
		self.modules.read().get(name).cloned()
	}
}
