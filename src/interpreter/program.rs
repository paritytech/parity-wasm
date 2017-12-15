use std::rc::Rc;
use std::collections::HashMap;
use elements::Module;
use interpreter::Error;
use interpreter::module::ModuleInstance;
use interpreter::func::FuncInstance;
use interpreter::host::HostModule;
use interpreter::value::RuntimeValue;
use interpreter::imports::{Imports, ImportResolver};

/// Program instance. Program is a set of instantiated modules.
pub struct ProgramInstance<St=()> {
	modules: HashMap<String, Rc<ModuleInstance<St>>>,
	resolvers: HashMap<String, Box<ImportResolver<St>>>,
}

impl<St> ProgramInstance<St> {
	/// Create new program instance.
	pub fn new() -> ProgramInstance<St> {
		ProgramInstance {
			modules: HashMap::new(),
			resolvers: HashMap::new(),
		}
	}

	/// Instantiate module with validation.
	pub fn add_module(
		&mut self,
		name: &str,
		module: Module,
		state: &mut St,
	) -> Result<Rc<ModuleInstance<St>>, Error> {
		let module_instance = {
			let mut imports = Imports::new();
			for (module_name, module_instance) in self.modules.iter() {
				imports.push_resolver(&**module_name, &**module_instance);
			}
			for (module_name, import_resolver) in self.resolvers.iter() {
				imports.push_resolver(&**module_name, &**import_resolver);
			}
			ModuleInstance::instantiate(&module)
				.with_imports(imports)
				.run_start(state)?
		};
		self.modules.insert(name.to_owned(), Rc::clone(&module_instance));

		Ok(module_instance)
	}

	pub fn add_import_resolver(
		&mut self,
		name: &str,
		import_resolver: Box<ImportResolver<St>>,
	) {
		self.resolvers.insert(name.to_owned(), import_resolver);
	}

	pub fn add_host_module(
		&mut self,
		name: &str,
		host_module: HostModule<St>,
	) where St: 'static {
		self.resolvers.insert(name.to_owned(), Box::new(host_module) as Box<ImportResolver<St>>);
	}

	pub fn insert_loaded_module(&mut self, name: &str, module: Rc<ModuleInstance<St>>) {
		self.modules.insert(name.to_owned(), module);
	}

	pub fn invoke_export(
		&mut self,
		module_name: &str,
		func_name: &str,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_instance = self.modules.get(module_name).ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		module_instance.invoke_export(func_name, args, state)
	}

	pub fn invoke_index(
		&mut self,
		module_name: &str,
		func_idx: u32,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_instance = self.modules.get(module_name).cloned().ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		module_instance.invoke_index(func_idx, args, state)
	}

	pub fn invoke_func(
		&mut self,
		func_instance: Rc<FuncInstance<St>>,
		args: Vec<RuntimeValue>,
		state: &mut St,
	) -> Result<Option<RuntimeValue>, Error> {
		FuncInstance::invoke(Rc::clone(&func_instance), args, state)
	}

	pub fn resolver(&self, name: &str) -> Option<&ImportResolver<St>> {
		self.modules
			.get(name)
			.map(|x| &**x as &ImportResolver<St>)
			.or_else(|| self.resolvers.get(name).map(|x| &**x))
	}

	pub fn module(&self, name: &str) -> Option<Rc<ModuleInstance<St>>> {
		self.modules.get(name).cloned()
	}
}
