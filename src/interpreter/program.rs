use std::rc::Rc;
use std::collections::HashMap;
use elements::Module;
use interpreter::Error;
use interpreter::store::{Store, ExternVal, FuncInstance, ModuleInstance};
use interpreter::host::HostModule;
use interpreter::value::RuntimeValue;

/// Program instance. Program is a set of instantiated modules.
pub struct ProgramInstance {
	store: Store,
	modules: HashMap<String, Rc<ModuleInstance>>,
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
	) -> Result<Rc<ModuleInstance>, Error> {
		let mut extern_vals = Vec::new();
		for import_entry in module.import_section().map(|s| s.entries()).unwrap_or(&[]) {
			let module = self.modules.get(import_entry.module()).ok_or_else(|| Error::Program(format!("Module {} not found", import_entry.module())))?;
			let extern_val = module
				.export_by_name(import_entry.field())
				.ok_or_else(|| {
					Error::Program(format!(
						"Module {} doesn't have export {}",
						import_entry.module(),
						import_entry.field()
					))
				})?;
			extern_vals.push(extern_val);
		}

		let module_instance = self.store.instantiate_module(&module, &extern_vals, state)?;
		self.modules.insert(name.to_owned(), Rc::clone(&module_instance));

		Ok(module_instance)
	}

	pub fn add_host_module(
		&mut self,
		name: &str,
		host_module: HostModule,
	) -> Result<Rc<ModuleInstance>, Error> {
		let module_instance = host_module.allocate()?;
		self.modules.insert(name.to_owned(), Rc::clone(&module_instance));
		Ok(module_instance)
	}

	pub fn insert_loaded_module(&mut self, name: &str, module: Rc<ModuleInstance>) {
		self.modules.insert(name.to_owned(), module);
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
			.export_by_name(func_name)
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
		let module_id = self.modules.get(module_name).cloned().ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		let func_id = module_id.func_by_index(func_idx).ok_or_else(|| {
			Error::Program(format!("Module doesn't contain function at index {}", func_idx))
		})?;
		self.invoke_func(func_id, args, state)
	}

	pub fn invoke_func<St: 'static>(
		&mut self,
		func: Rc<FuncInstance>,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		self.store.invoke(func, args, state)
	}

	pub fn store(&self) -> &Store {
		&self.store
	}

	pub fn module(&self, name: &str) -> Option<Rc<ModuleInstance>> {
		self.modules.get(name).cloned()
	}
}
