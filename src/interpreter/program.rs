
use std::collections::HashMap;
use elements::Module;
use interpreter::Error;
use interpreter::store::{Store, ModuleId};
use interpreter::host::HostModule;

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
	pub fn add_module<'a, St: 'static>(
		&mut self,
		name: &str,
		module: Module,
		state: &mut St,
	) -> Result<ModuleId, Error> {
		let mut extern_vals = Vec::new();
		for import_entry in module.import_section().map(|s| s.entries()).unwrap_or(&[]) {
			let module = self.modules[import_entry.module()];
			let extern_val = module
				.resolve_export(&self.store, import_entry.field())
				.ok_or_else(|| Error::Function(format!("Module {} doesn't have export {}", import_entry.module(), import_entry.field())))?;
			extern_vals.push(extern_val);
		}

		let module_id = self.store.instantiate_module(&module, &extern_vals, state)?;
		self.modules.insert(name.to_owned(), module_id);

		Ok(module_id)
	}

	pub fn add_host_module(&mut self, name: &str, host_module: HostModule) -> Result<ModuleId, Error> {
		let module_id = host_module.allocate(&mut self.store)?;
		self.modules.insert(name.to_owned(), module_id);
		Ok(module_id)
	}

	/// Get one of the modules by name
	pub fn module(&self, name: &str) -> Option<ModuleId> {
		self.modules.get(name).cloned()
	}
}
