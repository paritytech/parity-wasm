
use std::collections::HashMap;
use elements::Module;
use interpreter::Error;
use interpreter::store::{ModuleId, Store, ExternVal};
use interpreter::host::HostModule;
use interpreter::value::RuntimeValue;

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
				.ok_or_else(|| {
					Error::Program(format!(
						"Module {} doesn't have export {}",
						import_entry.module(),
						import_entry.field()
					))
				})?;
			extern_vals.push(extern_val);
		}

		let module_id = self.store.instantiate_module(&module, &extern_vals, state)?;
		self.modules.insert(name.to_owned(), module_id);

		Ok(module_id)
	}

	pub fn add_host_module(
		&mut self,
		name: &str,
		host_module: HostModule,
	) -> Result<ModuleId, Error> {
		let module_id = host_module.allocate(&mut self.store)?;
		self.modules.insert(name.to_owned(), module_id);
		Ok(module_id)
	}

	pub fn invoke_export<St: 'static>(
		&mut self,
		module_name: &str,
		func_name: &str,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_id = self.modules.get(module_name).ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		let extern_val = module_id
			.resolve_export(&self.store, func_name)
			.ok_or_else(|| {
				Error::Program(format!(
					"Module {} doesn't have export {}",
					module_name,
					func_name
				))
			})?;

		let func_id = match extern_val {
			ExternVal::Func(func_id) => func_id,
			unexpected => {
				return Err(Error::Program(format!(
					"Export {} is not a function, but {:?}",
					func_name,
					unexpected
				)))
			}
		};

		self.store.invoke(func_id, args, state)
	}

	pub fn invoke_index<St: 'static>(
		&mut self,
		module_name: &str,
		func_idx: u32,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_id = self.modules.get(module_name).ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		let func_id = module_id.resolve_func(&self.store, func_idx);

		self.store.invoke(func_id, args, state)
	}
}
