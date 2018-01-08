use std::collections::HashMap;
use std::borrow::Cow;
use elements::Module;
use interpreter::Error;
use interpreter::module::{ModuleInstance, ModuleRef};
use interpreter::func::{FuncInstance, FuncRef};
use interpreter::value::RuntimeValue;
use interpreter::imports::{Imports, ImportResolver};
use interpreter::host::Externals;
use validation::validate_module;

/// Program instance. Program is a set of instantiated modules.
#[deprecated]
pub struct ProgramInstance {
	modules: HashMap<String, ModuleRef>,
	resolvers: HashMap<String, Box<ImportResolver>>,
}

impl ProgramInstance {
	/// Create new program instance.
	pub fn new() -> ProgramInstance {
		ProgramInstance {
			modules: HashMap::new(),
			resolvers: HashMap::new(),
		}
	}

	/// Instantiate module with validation.
	pub fn add_module<'a, E: Externals>(
		&mut self,
		name: &str,
		module: Module,
		externals: &'a mut E,
	) -> Result<ModuleRef, Error> {
		let module_instance = {
			let mut imports = Imports::new();
			for (module_name, module_instance) in self.modules.iter() {
				imports.push_resolver(&**module_name, &**module_instance);
			}
			for (module_name, import_resolver) in self.resolvers.iter() {
				imports.push_resolver(&**module_name, &**import_resolver);
			}
			let validate_module = validate_module(module)?;
			ModuleInstance::new(&validate_module)
				.with_imports(imports)
				.run_start(externals)?
		};
		self.modules.insert(name.to_owned(), module_instance.clone());

		Ok(module_instance)
	}

	pub fn add_import_resolver(
		&mut self,
		name: &str,
		import_resolver: Box<ImportResolver>,
	) {
		self.resolvers.insert(name.to_owned(), import_resolver);
	}

	pub fn insert_loaded_module(&mut self, name: &str, module: ModuleRef) {
		self.modules.insert(name.to_owned(), module);
	}

	pub fn invoke_export<'a, E: Externals>(
		&mut self,
		module_name: &str,
		func_name: &str,
		args: &[RuntimeValue],
		externals: &'a mut E,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_instance = self.modules.get(module_name).ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		module_instance.invoke_export(func_name, args, externals)
	}

	pub fn invoke_index<'a, E: Externals>(
		&mut self,
		module_name: &str,
		func_idx: u32,
		args: &[RuntimeValue],
		externals: &'a mut E,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_instance = self.modules.get(module_name).cloned().ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		module_instance.invoke_index(func_idx, args, externals)
	}

	pub fn invoke_func<'a, E: Externals>(
		&mut self,
		func_instance: FuncRef,
		args: &[RuntimeValue],
		state: &'a mut E,
	) -> Result<Option<RuntimeValue>, Error> {
		FuncInstance::invoke(func_instance.clone(), Cow::Borrowed(args), state)
	}

	pub fn resolver(&self, name: &str) -> Option<&ImportResolver> {
		self.modules
			.get(name)
			.map(|x| &**x as &ImportResolver)
			.or_else(|| self.resolvers.get(name).map(|x| &**x))
	}

	pub fn module(&self, name: &str) -> Option<ModuleRef> {
		self.modules.get(name).cloned()
	}
}
