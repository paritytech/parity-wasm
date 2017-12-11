
use std::collections::HashMap;
use elements::Module;
use interpreter::Error;
use interpreter::module::{ExecutionParams};
use interpreter::store::{Store, ModuleId};

/// Program instance. Program is a set of instantiated modules.
pub struct ProgramInstance {
	store: Store,
	modules: HashMap<String, ModuleId>,
}

impl ProgramInstance {
	/// Create new program instance.
	pub fn new() -> Self {
		ProgramInstance {
			store: Store::new(),
			modules: HashMap::new(),
		}
	}

	/// Instantiate module with validation.
	pub fn add_module<'a>(
		&mut self,
		name: &str,
		module: Module,
		start_exec_params: ExecutionParams,
	) -> Result<ModuleId, Error> {
		let mut extern_vals = Vec::new();
		for import_entry in module.import_section().map(|s| s.entries()).unwrap_or(&[]) {
			let module = self.modules[import_entry.module()];
			let extern_val = module
				.resolve_export(&self.store, import_entry.field())
				.ok_or_else(|| Error::Function(format!("Module {} doesn't have export {}", import_entry.module(), import_entry.field())))?;
			extern_vals.push(extern_val);
		}

		let module_id = self.store.instantiate_module(&module, &extern_vals, start_exec_params)?;
		self.modules.insert(name.to_string(), module_id);

		Ok(module_id)
	}

	/// Get one of the modules by name
	pub fn module(&self, name: &str) -> Option<ModuleId> {
		self.modules.get(name).cloned()
	}
}
