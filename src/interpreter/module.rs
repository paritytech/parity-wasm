use std::sync::{Arc, Weak};
use elements::{Module, InitExpr, Opcode, Type, FunctionType, FuncBody};
use interpreter::Error;
use interpreter::imports::ModuleImports;
use interpreter::memory::MemoryInstance;
use interpreter::program::ProgramInstanceEssence;
use interpreter::runner::{Interpreter, FunctionContext};
use interpreter::stack::StackWithLimit;
use interpreter::table::TableInstance;
use interpreter::value::{RuntimeValue, TryInto};
use interpreter::variable::{VariableInstance, VariableType};

/// Item index in items index space.
#[derive(Debug, Clone, Copy)]
pub enum ItemIndex {
	/// Index in index space.
	IndexSpace(u32),
	/// Internal item index (i.e. index of item in items section).
	Internal(u32),
	/// External item index (i.e. index of item in export section).
	External(u32),
}

/// Module instance.
pub struct ModuleInstance {
	/// Module.
	module: Module,
	/// Module imports.
	imports: ModuleImports,
	/// Tables.
	tables: Vec<Arc<TableInstance>>,
	/// Linear memory regions.
	memory: Vec<Arc<MemoryInstance>>,
	/// Globals.
	globals: Vec<Arc<VariableInstance>>,
}

/// Caller context.
pub struct CallerContext<'a> {
	pub value_stack_limit: usize,
	pub frame_stack_limit: usize,
	pub value_stack: &'a mut StackWithLimit<RuntimeValue>,
}

impl ModuleInstance {
	/// Instantiate given module within program context.
	pub fn new(program: Weak<ProgramInstanceEssence>, module: Module) -> Result<Self, Error> {
		// TODO: missing validation step
	
		// load entries from import section
		let imports = ModuleImports::new(program, module.import_section());

		// instantiate linear memory regions, if any
		let mut memory = match module.memory_section() {
			Some(memory_section) => memory_section.entries()
										.iter()
										.map(MemoryInstance::new)
										.collect::<Result<Vec<_>, _>>()?,
			None => Vec::new(),
		};

		// instantiate tables, if any
		let mut tables = match module.table_section() {
			Some(table_section) => table_section.entries()
										.iter()
										.map(|tt| TableInstance::new(VariableType::AnyFunc, tt)) // TODO: actual table type
										.collect::<Result<Vec<_>, _>>()?,
			None => Vec::new(),
		};

		// instantiate globals, if any
		let globals = match module.global_section() {
			Some(global_section) => global_section.entries()
										.iter()
										.map(|g| {
											get_initializer(g.init_expr(), &module, &imports)
												.map_err(|e| Error::Initialization(e.into()))
												.and_then(|v| VariableInstance::new_global(g.global_type(), v).map(Arc::new))
										})
										.collect::<Result<Vec<_>, _>>()?,
			None => Vec::new(),
		};

		// use data section to initialize linear memory regions
		if let Some(data_section) = module.data_section() {
			for (data_segement_index, data_segment) in data_section.entries().iter().enumerate() {
				let offset: u32 = get_initializer(data_segment.offset(), &module, &imports)?.try_into()?;
				memory
					.get_mut(data_segment.index() as usize)
					.ok_or(Error::Initialization(format!("DataSegment {} initializes non-existant MemoryInstance {}", data_segement_index, data_segment.index())))
					.and_then(|m| m.set(offset, data_segment.value()).map_err(|e| Error::Initialization(e.into())))?;
			}
		}

		// use element section to fill tables
		if let Some(element_section) = module.elements_section() {
			for (element_segment_index, element_segment) in element_section.entries().iter().enumerate() {
				let offset: u32 = get_initializer(element_segment.offset(), &module, &imports)?.try_into()?;
				tables
					.get_mut(element_segment.index() as usize)
					.ok_or(Error::Initialization(format!("ElementSegment {} initializes non-existant Table {}", element_segment_index, element_segment.index())))
					.and_then(|m| m.set_raw(offset, element_segment.members()).map_err(|e| Error::Initialization(e.into())))?;
			}
		}

		Ok(ModuleInstance {
			module: module,
			imports: imports,
			memory: memory,
			tables: tables,
			globals: globals,
		})
	}

	/// Execute start function of the module.
	pub fn execute_main(&self, args: Vec<RuntimeValue>) -> Result<Option<RuntimeValue>, Error> {
		let index = self.module.start_section().ok_or(Error::Program("module has no start section".into()))?;
		self.execute(index, args)
	}

	/// Execute function with the given index.
	pub fn execute(&self, index: u32, args: Vec<RuntimeValue>) -> Result<Option<RuntimeValue>, Error> {
		let args_len = args.len();
		let mut args = StackWithLimit::with_data(args, args_len);
		let caller_context = CallerContext::topmost(&mut args);
		self.call_function(caller_context, ItemIndex::IndexSpace(index))
	}

	/// Get module description reference.
	pub fn module(&self) -> &Module {
		&self.module
	}

	/// Get table reference.
	pub fn table(&self, index: ItemIndex) -> Result<Arc<TableInstance>, Error> {
		match self.imports.parse_table_index(index) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_table_index resolves IndexSpace option"),
			ItemIndex::Internal(index) => self.tables.get(index as usize).cloned()
				.ok_or(Error::Table(format!("trying to access table with local index {} when there are only {} local tables", index, self.tables.len()))),
			ItemIndex::External(index) => self.module.import_section()
				.ok_or(Error::Table(format!("trying to access external table with index {} in module without import section", index)))
				.and_then(|s| s.entries().get(index as usize)
					.ok_or(Error::Table(format!("trying to access external table with index {} in module with {}-entries import section", index, s.entries().len()))))
				.and_then(|e| self.imports.table(e)),
		}
	}

	/// Get memory reference.
	pub fn memory(&self, index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		match self.imports.parse_memory_index(index) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_memory_index resolves IndexSpace option"),
			ItemIndex::Internal(index) => self.memory.get(index as usize).cloned()
				.ok_or(Error::Memory(format!("trying to access memory with local index {} when there are only {} memory regions", index, self.memory.len()))),
			ItemIndex::External(index) => self.module.import_section()
				.ok_or(Error::Memory(format!("trying to access external memory with index {} in module without import section", index)))
				.and_then(|s| s.entries().get(index as usize)
					.ok_or(Error::Memory(format!("trying to access external memory with index {} in module with {}-entries import section", index, s.entries().len()))))
				.and_then(|e| self.imports.memory(e)),
		}
	}

	/// Get global reference.
	pub fn global(&self, index: ItemIndex) -> Result<Arc<VariableInstance>, Error> {
		match self.imports.parse_global_index(index) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_global_index resolves IndexSpace option"),
			ItemIndex::Internal(index) => self.globals.get(index as usize).cloned()
				.ok_or(Error::Global(format!("trying to access global with local index {} when there are only {} globals", index, self.globals.len()))),
			ItemIndex::External(index) => self.module.import_section()
				.ok_or(Error::Global(format!("trying to access external global with index {} in module without import section", index)))
				.and_then(|s| s.entries().get(index as usize)
					.ok_or(Error::Global(format!("trying to access external global with index {} in module with {}-entries import section", index, s.entries().len()))))
				.and_then(|e| self.imports.global(e)),
		}
	}

	/// Call function with given index in functions index space.
	pub fn call_function(&self, outer: CallerContext, index: ItemIndex) -> Result<Option<RuntimeValue>, Error> {
		match self.imports.parse_function_index(index) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_function_index resolves IndexSpace option"),
			ItemIndex::Internal(index) => self.call_internal_function(outer, index, None),
			ItemIndex::External(index) =>
				self.module.import_section()
				.ok_or(Error::Function(format!("trying to access external function with index {} in module without import section", index)))
				.and_then(|s| s.entries().get(index as usize)
					.ok_or(Error::Function(format!("trying to access external function with index {} in module with {}-entries import section", index, s.entries().len()))))
				.and_then(|e| self.imports.module(e.module()))
				.and_then(|m| m.call_internal_function(outer, index, None)),
		}
	}

	/// Call function with given index in the given table.
	pub fn call_function_indirect(&self, outer: CallerContext, table_index: ItemIndex, type_index: u32, func_index: u32) -> Result<Option<RuntimeValue>, Error> {
		let function_type = match self.module.type_section()
			.ok_or(Error::Function(format!("trying to indirect call function {} with non-existent function section", func_index)))
			.and_then(|s| s.types().get(type_index as usize)
				.ok_or(Error::Function(format!("trying to indirect call function {} with non-existent type index {}", func_index, type_index))))? {
			&Type::Function(ref function_type) => function_type,
		};

		match self.imports.parse_table_index(table_index) {
			ItemIndex::IndexSpace(_) => unreachable!("parse_function_index resolves IndexSpace option"),
			ItemIndex::Internal(table_index) => {
				let table = self.table(ItemIndex::Internal(table_index))?;
				let index = match table.get(func_index)? {
					RuntimeValue::AnyFunc(index) => index,
					_ => return Err(Error::Function(format!("trying to indirect call function {} via non-anyfunc table {}", func_index, table_index))),
				};
				self.call_internal_function(outer, index, Some(function_type))
			},
			ItemIndex::External(table_index) => {
				let table = self.table(ItemIndex::External(table_index))?;
				let index = match table.get(func_index)? {
					RuntimeValue::AnyFunc(index) => index,
					_ => return Err(Error::Function(format!("trying to indirect call function {} via non-anyfunc table {}", func_index, table_index))),
				};
				let module = self.module.import_section()
					.ok_or(Error::Function(format!("trying to access external table with index {} in module without import section", table_index)))
					.and_then(|s| s.entries().get(table_index as usize)
						.ok_or(Error::Function(format!("trying to access external table with index {} in module with {}-entries import section", table_index, s.entries().len()))))
					.and_then(|e| self.imports.module(e.module()))?;
				module.call_internal_function(outer, index, Some(function_type))
			}
		}
	}

	/// Call function with internal index.
	fn call_internal_function(&self, outer: CallerContext, index: u32, function_type: Option<&FunctionType>) -> Result<Option<RuntimeValue>, Error> {
		// TODO: cache
		// internal index = index of function in functions section && index of code in code section
		// get function type index
		let function_type_index = self.module
			.functions_section()
			.ok_or(Error::Function(format!("trying to call function with index {} in module without function section", index)))
			.and_then(|s| s.entries()
				.get(index as usize)
				.ok_or(Error::Function(format!("trying to call function with index {} in module with {} functions", index, s.entries().len()))))?
			.type_ref();
		// function type index = index of function type in types index
		// get function type
		let item_type = self.module
			.type_section()
			.ok_or(Error::Function(format!("trying to call function with index {} in module without types section", index)))
			.and_then(|s| s.types()
				.get(function_type_index as usize)
				.ok_or(Error::Function(format!("trying to call function with type index {} in module with {} types", index, s.types().len()))))?;
		let actual_function_type = match item_type {
			&Type::Function(ref function_type) => function_type,
		};
		if let Some(ref function_type) = function_type {
			if function_type != &actual_function_type {
				return Err(Error::Function(format!("expected function with signature ({:?}) -> {:?} when got with ({:?}) -> {:?}",
					function_type.params(), function_type.return_type(), actual_function_type.params(), actual_function_type.return_type())));
			}
		}
		// get function body
		let function_body = self.module
			.code_section()
			.ok_or(Error::Function(format!("trying to call function with index {} in module without code section", index)))
			.and_then(|s| s.bodies()
				.get(index as usize)
				.ok_or(Error::Function(format!("trying to call function with index {} in module with {} functions codes", index, s.bodies().len()))))?;

		// each functions has its own value stack
		// but there's global stack limit
		// args, locals
		let function_code = function_body.code().elements();
		let value_stack_limit = outer.value_stack_limit;
		let frame_stack_limit = outer.frame_stack_limit;
		let locals = prepare_function_locals(actual_function_type, function_body, outer)?;
		let mut innner = FunctionContext::new(self, value_stack_limit, frame_stack_limit, actual_function_type, function_code, locals)?;
		Interpreter::run_function(&mut innner, function_code)
	}
}

impl<'a> CallerContext<'a> {
	pub fn topmost(args: &'a mut StackWithLimit<RuntimeValue>) -> Self {
		CallerContext {
			value_stack_limit: 1024,
			frame_stack_limit: 1024,
			value_stack: args,
		}
	}

	pub fn nested(outer: &'a mut FunctionContext) -> Self {
		CallerContext {
			value_stack_limit: outer.value_stack().limit() - outer.value_stack().len(),
			frame_stack_limit: outer.frame_stack().limit() - outer.frame_stack().len(),
			value_stack: outer.value_stack_mut(),
		}
	}
}

fn prepare_function_locals(function_type: &FunctionType, function_body: &FuncBody, outer: CallerContext) -> Result<Vec<VariableInstance>, Error> {
	// locals = function arguments + defined locals
	function_type.params().iter().rev()
		.map(|param_type| {
			let param_value = outer.value_stack.pop()?;
			let actual_type = param_value.variable_type();
			let expected_type = (*param_type).into();
			if actual_type != Some(expected_type) {
				return Err(Error::Function(format!("invalid parameter type {:?} when expected {:?}", actual_type, expected_type)));
			}

			VariableInstance::new(true, expected_type, param_value)
		})
		.collect::<Vec<_>>().into_iter().rev()
		// TODO: default values (zero), not null
		.chain(function_body.locals().iter().map(|l| VariableInstance::new(true, l.value_type().into(), RuntimeValue::default(l.value_type().into()))))
		.collect::<Result<Vec<_>, _>>()
}

fn get_initializer(expr: &InitExpr, module: &Module, imports: &ModuleImports) -> Result<RuntimeValue, Error> {
	let first_opcode = expr.code().get(0).ok_or(Error::Initialization(format!("empty instantiation-time initializer")))?;
	match first_opcode {
		&Opcode::GetGlobal(index) => module.import_section()
			.ok_or(Error::Global(format!("trying to initialize with external global with index {} in module without import section", index)))
			.and_then(|s| s.entries().get(index as usize)
				.ok_or(Error::Global(format!("trying to initialize with external global with index {} in module with {}-entries import section", index, s.entries().len()))))
			.and_then(|e| imports.global(e))
			.map(|g| g.get()),
		&Opcode::I32Const(val) => Ok(RuntimeValue::I32(val)),
		&Opcode::I64Const(val) => Ok(RuntimeValue::I64(val)),
		&Opcode::F32Const(val) => Ok(RuntimeValue::F32(val as f32)), // TODO
		&Opcode::F64Const(val) => Ok(RuntimeValue::F64(val as f64)), // TODO
		_ => Err(Error::Initialization(format!("not-supported {:?} instruction in instantiation-time initializer", first_opcode))),
	}
}
