use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use elements::Module;
use interpreter::Error;
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
	pub fn new() -> Self {
		ProgramInstance {
			essence: Arc::new(ProgramInstanceEssence::new()),
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
	pub fn insert_loaded_module(&self, name: &str, module_instance: Arc<ModuleInstanceInterface>) -> Result<Arc<ModuleInstanceInterface>, Error> {
		// replace existing module with the same name with new one
		self.essence.modules.write().insert(name.into(), Arc::clone(&module_instance));
		Ok(module_instance)
	}

	/// Get one of the modules by name
	pub fn module(&self, name: &str) -> Option<Arc<ModuleInstanceInterface>> {
		self.essence.module(name)
	}
}

impl ProgramInstanceEssence {
	/// Create new program essence.
	pub fn new() -> Self {
		ProgramInstanceEssence {
			modules: RwLock::new(HashMap::new()),
		}
	}

	/// Get module reference.
	pub fn module(&self, name: &str) -> Option<Arc<ModuleInstanceInterface>> {
		self.modules.read().get(name).cloned()
	}
}
