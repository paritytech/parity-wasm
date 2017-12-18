use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::collections::HashMap;
use std::borrow::Cow;
use elements::{External, FunctionType, GlobalType, InitExpr, Internal, MemoryType, Module, Opcode,
			   ResizableLimits, TableType, Type};
use interpreter::{Error, MemoryInstance, RuntimeValue, TableInstance};
use interpreter::imports::{ImportResolver, Imports};
use interpreter::global::GlobalInstance;
use interpreter::func::{FuncBody, FuncInstance};
use validation::validate_module;
use common::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};

pub enum ExternVal<St> {
	Func(Rc<FuncInstance<St>>),
	Table(Rc<TableInstance<St>>),
	Memory(Rc<MemoryInstance>),
	Global(Rc<GlobalInstance>),
}

impl<St> Clone for ExternVal<St> {
	fn clone(&self) -> Self {
		match *self {
			ExternVal::Func(ref func) => ExternVal::Func(Rc::clone(func)),
			ExternVal::Table(ref table) => ExternVal::Table(Rc::clone(table)),
			ExternVal::Memory(ref memory) => ExternVal::Memory(Rc::clone(memory)),
			ExternVal::Global(ref global) => ExternVal::Global(Rc::clone(global)),
		}
	}
}

impl<St> fmt::Debug for ExternVal<St> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"ExternVal {{ {} }}",
			match *self {
				ExternVal::Func(_) => "Func",
				ExternVal::Table(_) => "Table",
				ExternVal::Memory(_) => "Memory",
				ExternVal::Global(_) => "Global",
			}
		)
	}
}

impl<St> ExternVal<St> {
	pub fn as_func(&self) -> Option<Rc<FuncInstance<St>>> {
		match *self {
			ExternVal::Func(ref func) => Some(Rc::clone(func)),
			_ => None,
		}
	}

	pub fn as_table(&self) -> Option<Rc<TableInstance<St>>> {
		match *self {
			ExternVal::Table(ref table) => Some(Rc::clone(table)),
			_ => None,
		}
	}

	pub fn as_memory(&self) -> Option<Rc<MemoryInstance>> {
		match *self {
			ExternVal::Memory(ref memory) => Some(Rc::clone(memory)),
			_ => None,
		}
	}

	pub fn as_global(&self) -> Option<Rc<GlobalInstance>> {
		match *self {
			ExternVal::Global(ref global) => Some(Rc::clone(global)),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub struct ModuleInstance<St> {
	types: RefCell<Vec<Rc<FunctionType>>>,
	tables: RefCell<Vec<Rc<TableInstance<St>>>>,
	funcs: RefCell<Vec<Rc<FuncInstance<St>>>>,
	memories: RefCell<Vec<Rc<MemoryInstance>>>,
	globals: RefCell<Vec<Rc<GlobalInstance>>>,
	exports: RefCell<HashMap<String, ExternVal<St>>>,
}

impl<St> Default for ModuleInstance<St> {
	fn default() -> Self {
		ModuleInstance {
			types: RefCell::new(Vec::new()),
			tables: RefCell::new(Vec::new()),
			funcs: RefCell::new(Vec::new()),
			memories: RefCell::new(Vec::new()),
			globals: RefCell::new(Vec::new()),
			exports: RefCell::new(HashMap::new()),
		}
	}
}

impl<St> ModuleInstance<St> {
	pub fn with_exports(exports: HashMap<String, ExternVal<St>>) -> Self {
		let mut instance = Self::default();
		instance.exports = RefCell::new(exports);
		instance
	}

	pub fn memory_by_index(&self, idx: u32) -> Option<Rc<MemoryInstance>> {
		self.memories.borrow().get(idx as usize).cloned()
	}

	pub fn table_by_index(&self, idx: u32) -> Option<Rc<TableInstance<St>>> {
		self.tables.borrow().get(idx as usize).cloned()
	}

	pub fn global_by_index(&self, idx: u32) -> Option<Rc<GlobalInstance>> {
		self.globals.borrow().get(idx as usize).cloned()
	}

	pub fn func_by_index(&self, idx: u32) -> Option<Rc<FuncInstance<St>>> {
		self.funcs.borrow().get(idx as usize).cloned()
	}

	pub fn type_by_index(&self, idx: u32) -> Option<Rc<FunctionType>> {
		self.types.borrow().get(idx as usize).cloned()
	}

	pub fn export_by_name(&self, name: &str) -> Option<ExternVal<St>> {
		self.exports.borrow().get(name).cloned()
	}

	fn push_func(&self, func: Rc<FuncInstance<St>>) {
		self.funcs.borrow_mut().push(func);
	}

	fn push_type(&self, func_type: Rc<FunctionType>) {
		self.types.borrow_mut().push(func_type)
	}

	fn push_memory(&self, memory: Rc<MemoryInstance>) {
		self.memories.borrow_mut().push(memory)
	}

	fn push_table(&self, table: Rc<TableInstance<St>>) {
		self.tables.borrow_mut().push(table)
	}

	fn push_global(&self, global: Rc<GlobalInstance>) {
		self.globals.borrow_mut().push(global)
	}

	fn insert_export<N: Into<String>>(&self, name: N, extern_val: ExternVal<St>) {
		self.exports.borrow_mut().insert(name.into(), extern_val);
	}

	fn alloc_module(
		module: &Module,
		extern_vals: &[ExternVal<St>],
		instance: &Rc<ModuleInstance<St>>,
	) -> Result<(), Error> {
		let mut aux_data = validate_module(module)?;

		for &Type::Function(ref ty) in module.type_section().map(|ts| ts.types()).unwrap_or(&[]) {
			let type_id = alloc_func_type(ty.clone());
			instance.push_type(type_id);
		}

		{
			let imports = module.import_section().map(|is| is.entries()).unwrap_or(
				&[],
			);
			if imports.len() != extern_vals.len() {
				return Err(Error::Instatiation(format!(
					"extern_vals length is not equal to import section entries"
				)));
			}

			for (import, extern_val) in
				Iterator::zip(imports.into_iter(), extern_vals.into_iter())
			{
				match (import.external(), extern_val) {
					(&External::Function(fn_type_idx), &ExternVal::Func(ref func)) => {
						let expected_fn_type = instance.type_by_index(fn_type_idx).expect(
							"Due to validation function type should exists",
						);
						let actual_fn_type = func.func_type();
						if expected_fn_type != actual_fn_type {
							return Err(Error::Instatiation(format!(
								"Expected function with type {:?}, but actual type is {:?} for entry {}",
								expected_fn_type,
								actual_fn_type,
								import.field(),
							)));
						}
						instance.push_func(Rc::clone(func))
					}
					(&External::Table(ref tt), &ExternVal::Table(ref table)) => {
						match_limits(table.limits(), tt.limits())?;
						instance.push_table(Rc::clone(table));
					}
					(&External::Memory(ref mt), &ExternVal::Memory(ref memory)) => {
						match_limits(memory.limits(), mt.limits())?;
						instance.push_memory(Rc::clone(memory));
					}
					(&External::Global(ref gl), &ExternVal::Global(ref global)) => {
						if gl.content_type() != global.value_type() {
							return Err(Error::Instatiation(format!(
								"Expect global with {:?} type, but provided global with {:?} type",
								gl.content_type(),
								global.value_type(),
							)));
						}
						instance.push_global(Rc::clone(global))
					}
					(expected_import, actual_extern_val) => {
						return Err(Error::Instatiation(format!(
							"Expected {:?} type, but provided {:?} extern_val",
							expected_import,
							actual_extern_val
						)));
					}
				}
			}
		}

		{
			let funcs = module.function_section().map(|fs| fs.entries()).unwrap_or(
				&[],
			);
			let bodies = module.code_section().map(|cs| cs.bodies()).unwrap_or(&[]);
			debug_assert!(
				funcs.len() == bodies.len(),
				"Due to validation func and body counts must match"
			);

			for (index, (ty, body)) in
				Iterator::zip(funcs.into_iter(), bodies.into_iter()).enumerate()
			{
				let func_type = instance.type_by_index(ty.type_ref()).expect(
					"Due to validation type should exists",
				);
				let labels = aux_data.labels.remove(&index).expect(
					"At func validation time labels are collected; Collected labels are added by index; qed",
				);
				let func_body = FuncBody {
					locals: body.locals().to_vec(),
					opcodes: body.code().clone(),
					labels: labels,
				};
				let func_instance =
					FuncInstance::alloc_internal(Rc::clone(instance), func_type, func_body);
				instance.push_func(func_instance);
			}
		}

		for table_type in module.table_section().map(|ts| ts.entries()).unwrap_or(&[]) {
			let table = alloc_table(table_type)?;
			instance.push_table(table);
		}

		for memory_type in module.memory_section().map(|ms| ms.entries()).unwrap_or(
			&[],
		)
		{
			let memory = alloc_memory(memory_type)?;
			instance.push_memory(memory);
		}

		for global_entry in module.global_section().map(|gs| gs.entries()).unwrap_or(
			&[],
		)
		{
			let init_val = eval_init_expr(global_entry.init_expr(), &*instance);
			let global = alloc_global(global_entry.global_type().clone(), init_val);
			instance.push_global(global);
		}

		for export in module.export_section().map(|es| es.entries()).unwrap_or(
			&[],
		)
		{
			let field = export.field();
			let extern_val: ExternVal<St> = match *export.internal() {
				Internal::Function(idx) => {
					let func = instance.func_by_index(idx).expect(
						"Due to validation func should exists",
					);
					ExternVal::Func(func)
				}
				Internal::Global(idx) => {
					let global = instance.global_by_index(idx).expect(
						"Due to validation global should exists",
					);
					ExternVal::Global(global)
				}
				Internal::Memory(idx) => {
					let memory = instance.memory_by_index(idx).expect(
						"Due to validation memory should exists",
					);
					ExternVal::Memory(memory)
				}
				Internal::Table(idx) => {
					let table = instance.table_by_index(idx).expect(
						"Due to validation table should exists",
					);
					ExternVal::Table(table)
				}
			};
			instance.insert_export(field, extern_val);
		}

		Ok(())
	}

	fn instantiate_with_externvals(
		module: &Module,
		extern_vals: &[ExternVal<St>],
	) -> Result<Rc<ModuleInstance<St>>, Error> {
		let instance = Rc::new(ModuleInstance::default());

		ModuleInstance::alloc_module(module, extern_vals, &instance)?;

		for element_segment in module.elements_section().map(|es| es.entries()).unwrap_or(
			&[],
		)
		{
			let offset_val = match eval_init_expr(element_segment.offset(), &instance) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation elem segment offset should evaluate to i32"),
			};

			let table_inst = instance.table_by_index(DEFAULT_TABLE_INDEX).expect(
				"Due to validation default table should exists",
			);
			for (j, func_idx) in element_segment.members().into_iter().enumerate() {
				let func = instance.func_by_index(*func_idx).expect(
					"Due to validation funcs from element segments should exists",
				);

				table_inst.set(offset_val + j as u32, func)?;
			}
		}

		for data_segment in module.data_section().map(|ds| ds.entries()).unwrap_or(&[]) {
			let offset_val = match eval_init_expr(data_segment.offset(), &instance) {
				RuntimeValue::I32(v) => v as u32,
				_ => panic!("Due to validation data segment offset should evaluate to i32"),
			};

			let memory_inst = instance.memory_by_index(DEFAULT_MEMORY_INDEX).expect(
				"Due to validation default memory should exists",
			);
			memory_inst.set(offset_val, data_segment.value())?;
		}

		Ok(instance)
	}

	fn instantiate_with_imports(
		module: &Module,
		imports: &Imports<St>,
	) -> Result<Rc<ModuleInstance<St>>, Error> {
		let mut extern_vals = Vec::new();
		for import_entry in module.import_section().map(|s| s.entries()).unwrap_or(&[]) {
			let module_name = import_entry.module();
			let field_name = import_entry.field();
			let resolver = imports.resolver(module_name).ok_or_else(|| {
				Error::Instatiation(format!("Module {} not found", module_name))
			})?;
			let extern_val = match *import_entry.external() {
				External::Function(fn_ty_idx) => {
					// Module is not yet validated so we have to check type indexes.
					let types = module.type_section().map(|s| s.types()).unwrap_or(&[]);
					let &Type::Function(ref func_type) =
						types.get(fn_ty_idx as usize).ok_or_else(|| {
							Error::Validation(format!("Function type {} not found", fn_ty_idx))
						})?;

					let func = resolver.resolve_func(field_name, func_type)?;
					ExternVal::Func(func)
				}
				External::Table(ref table_type) => {
					let table = resolver.resolve_table(field_name, table_type)?;
					ExternVal::Table(table)
				}
				External::Memory(ref memory_type) => {
					let memory = resolver.resolve_memory(field_name, memory_type)?;
					ExternVal::Memory(memory)
				}
				External::Global(ref global_type) => {
					let global = resolver.resolve_global(field_name, global_type)?;
					ExternVal::Global(global)
				}
			};
			extern_vals.push(extern_val);
		}

		Self::instantiate_with_externvals(module, &extern_vals)
	}

	pub fn new<'a>(module: &'a Module) -> InstantiationBuilder<'a, St> {
		InstantiationBuilder::new(module)
	}

	pub fn invoke_index(
		&self,
		func_idx: u32,
		args: &[RuntimeValue],
		state: &St,
	) -> Result<Option<RuntimeValue>, Error> {
		let func_instance = self.func_by_index(func_idx).ok_or_else(|| {
			Error::Program(format!(
				"Module doesn't contain function at index {}",
				func_idx
			))
		})?;
		FuncInstance::invoke(func_instance, Cow::Borrowed(args), state)
	}

	pub fn invoke_export(
		&self,
		func_name: &str,
		args: &[RuntimeValue],
		state: &St,
	) -> Result<Option<RuntimeValue>, Error> {
		let extern_val = self.export_by_name(func_name).ok_or_else(|| {
			Error::Program(format!("Module doesn't have export {}", func_name))
		})?;

		let func_instance = match extern_val {
			ExternVal::Func(func_instance) => func_instance,
			unexpected => {
				return Err(Error::Program(format!(
					"Export {} is not a function, but {:?}",
					func_name,
					unexpected
				)));
			}
		};

		FuncInstance::invoke(Rc::clone(&func_instance), Cow::Borrowed(args), state)
	}
}

pub struct InstantiationBuilder<'a, St: 'a> {
	module: &'a Module,
	imports: Option<Imports<'a, St>>,
}

impl<'a, St: 'a> InstantiationBuilder<'a, St> {
	fn new(module: &'a Module) -> Self {
		InstantiationBuilder {
			module,
			imports: None,
		}
	}

	pub fn with_imports(mut self, imports: Imports<'a, St>) -> Self {
		self.imports = Some(imports);
		self
	}

	pub fn with_import<N: Into<String>>(
		mut self,
		name: N,
		import_resolver: &'a ImportResolver<St>,
	) -> Self {
		self.imports
			.get_or_insert_with(|| Imports::default())
			.push_resolver(name, import_resolver);
		self
	}

	pub fn run_start(mut self, state: &St) -> Result<Rc<ModuleInstance<St>>, Error> {
		let imports = self.imports.get_or_insert_with(|| Imports::default());
		let instance = ModuleInstance::instantiate_with_imports(self.module, imports)?;

		if let Some(start_fn_idx) = self.module.start_section() {
			let start_func = instance.func_by_index(start_fn_idx).expect(
				"Due to validation start function should exists",
			);
			FuncInstance::invoke(start_func, Cow::Borrowed(&[]), state)?;
		}
		Ok(instance)
	}

	pub fn assert_no_start(mut self) -> Result<Rc<ModuleInstance<St>>, Error> {
		assert!(self.module.start_section().is_none());
		let imports = self.imports.get_or_insert_with(|| Imports::default());
		let instance = ModuleInstance::instantiate_with_imports(self.module, imports)?;
		Ok(instance)
	}
}

impl<St> ImportResolver<St> for ModuleInstance<St> {
	fn resolve_func(
		&self,
		field_name: &str,
		_func_type: &FunctionType,
	) -> Result<Rc<FuncInstance<St>>, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Validation(format!("Export {} not found", field_name))
			})?
			.as_func()
			.ok_or_else(|| {
				Error::Validation(format!("Export {} is not a function", field_name))
			})?)
	}

	fn resolve_global(
		&self,
		field_name: &str,
		_global_type: &GlobalType,
	) -> Result<Rc<GlobalInstance>, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Validation(format!("Export {} not found", field_name))
			})?
			.as_global()
			.ok_or_else(|| {
				Error::Validation(format!("Export {} is not a global", field_name))
			})?)
	}

	fn resolve_memory(
		&self,
		field_name: &str,
		_memory_type: &MemoryType,
	) -> Result<Rc<MemoryInstance>, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Validation(format!("Export {} not found", field_name))
			})?
			.as_memory()
			.ok_or_else(|| {
				Error::Validation(format!("Export {} is not a memory", field_name))
			})?)
	}

	fn resolve_table(
		&self,
		field_name: &str,
		_table_type: &TableType,
	) -> Result<Rc<TableInstance<St>>, Error> {
		Ok(self.export_by_name(field_name)
			.ok_or_else(|| {
				Error::Validation(format!("Export {} not found", field_name))
			})?
			.as_table()
			.ok_or_else(|| {
				Error::Validation(format!("Export {} is not a table", field_name))
			})?)
	}
}

fn alloc_func_type(func_type: FunctionType) -> Rc<FunctionType> {
	Rc::new(func_type)
}

fn alloc_table<St>(table_type: &TableType) -> Result<Rc<TableInstance<St>>, Error> {
	let table = TableInstance::new(table_type)?;
	Ok(Rc::new(table))
}

fn alloc_memory(mem_type: &MemoryType) -> Result<Rc<MemoryInstance>, Error> {
	let memory = MemoryInstance::new(&mem_type)?;
	Ok(Rc::new(memory))
}

fn alloc_global(global_type: GlobalType, val: RuntimeValue) -> Rc<GlobalInstance> {
	let global = GlobalInstance::new(val, global_type.is_mutable());
	Rc::new(global)
}

fn eval_init_expr<T>(init_expr: &InitExpr, module: &ModuleInstance<T>) -> RuntimeValue {
	let code = init_expr.code();
	debug_assert!(
		code.len() == 2,
		"Due to validation `code`.len() should be 2"
	);
	match code[0] {
		Opcode::I32Const(v) => v.into(),
		Opcode::I64Const(v) => v.into(),
		Opcode::F32Const(v) => RuntimeValue::decode_f32(v),
		Opcode::F64Const(v) => RuntimeValue::decode_f64(v),
		Opcode::GetGlobal(idx) => {
			let global = module.global_by_index(idx).expect(
				"Due to validation global should exists in module",
			);
			global.get()
		}
		_ => panic!("Due to validation init should be a const expr"),
	}
}

fn match_limits(l1: &ResizableLimits, l2: &ResizableLimits) -> Result<(), Error> {
	if l1.initial() < l2.initial() {
		return Err(Error::Instatiation(format!(
			"trying to import with limits l1.initial={} and l2.initial={}",
			l1.initial(),
			l2.initial()
		)));
	}

	match (l1.maximum(), l2.maximum()) {
		(_, None) => (),
		(Some(m1), Some(m2)) if m1 <= m2 => (),
		_ => {
			return Err(Error::Instatiation(format!(
				"trying to import with limits l1.max={:?} and l2.max={:?}",
				l1.maximum(),
				l2.maximum()
			)))
		}
	}

	Ok(())
}

pub fn check_limits(limits: &ResizableLimits) -> Result<(), Error> {
	if let Some(maximum) = limits.maximum() {
		if maximum < limits.initial() {
			return Err(Error::Validation(format!(
				"maximum limit {} is lesser than minimum {}",
				maximum,
				limits.initial()
			)));
		}
	}

	Ok(())
}
