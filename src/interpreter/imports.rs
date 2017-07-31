use std::sync::{Arc, Weak};
use std::collections::HashMap;
use elements::{ImportSection, ImportEntry, External, Internal};
use interpreter::Error;
use interpreter::memory::MemoryInstance;
use interpreter::module::{ModuleInstanceInterface, ItemIndex, ExportEntryType, FunctionSignature};
use interpreter::program::ProgramInstanceEssence;
use interpreter::table::TableInstance;
use interpreter::variable::{VariableInstance, VariableType};

/// Module imports.
pub struct ModuleImports {
	/// Program instance.
	program: Weak<ProgramInstanceEssence>,
	/// External functions.
	functions: Vec<usize>,
	/// External tables.
	tables: Vec<usize>,
	/// External memory regions.
	memory: Vec<usize>,
	/// External globals.
	globals: Vec<usize>,
}

impl ModuleImports {
	/// Create new imports for given import section.
	pub fn new(program: Weak<ProgramInstanceEssence>, import_section: Option<&ImportSection>) -> Self {
		let mut functions = Vec::new();
		let mut tables = Vec::new();
		let mut memory = Vec::new();
		let mut globals = Vec::new();
		if let Some(import_section) = import_section {
			for (import_index, import_entry) in import_section.entries().iter().enumerate() {
				match import_entry.external() {
					&External::Function(_) => functions.push(import_index),
					&External::Table(_) => tables.push(import_index),
					&External::Memory(_) => memory.push(import_index),
					&External::Global(_) => globals.push(import_index),
				}
			}
		}

		ModuleImports {
			program: program,
			functions: functions,
			tables: tables,
			memory: memory,
			globals: globals,
		}
	}

	/// Number of imported tables.
	pub fn tables_len(&self) -> usize {
		self.tables.len()
	}

	/// Number of imported memory regions.
	pub fn memory_regions_len(&self) -> usize {
		self.memory.len()
	}

	/// Parse function index.
	pub fn parse_function_index(&self, index: ItemIndex) -> ItemIndex {
		match index {
			ItemIndex::IndexSpace(index) => match index.checked_sub(self.functions.len() as u32) {
				Some(index) => ItemIndex::Internal(index),
				None => ItemIndex::External(self.functions[index as usize] as u32),
			},
			index @ _ => index,
		}
	}

	/// Parse table index.
	pub fn parse_table_index(&self, index: ItemIndex) -> ItemIndex {
		match index {
			ItemIndex::IndexSpace(index) => match index.checked_sub(self.tables.len() as u32) {
				Some(index) => ItemIndex::Internal(index),
				None => ItemIndex::External(self.tables[index as usize] as u32),
			},
			index @ _ => index,
		}
	}

	/// Parse memory index.
	pub fn parse_memory_index(&self, index: ItemIndex) -> ItemIndex {
		match index {
			ItemIndex::IndexSpace(index) => match index.checked_sub(self.memory.len() as u32) {
				Some(index) => ItemIndex::Internal(index),
				None => ItemIndex::External(self.memory[index as usize] as u32),
			},
			index @ _ => index,
		}
	}

	/// Parse global index.
	pub fn parse_global_index(&self, index: ItemIndex) -> ItemIndex {
		match index {
			ItemIndex::IndexSpace(index) => match index.checked_sub(self.globals.len() as u32) {
				Some(index) => ItemIndex::Internal(index),
				None => ItemIndex::External(self.globals[index as usize] as u32),
			},
			index @ _ => index,
		}
	}

	/// Get module reference.
	pub fn module<'a>(&self, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>, name: &str) -> Result<Arc<ModuleInstanceInterface + 'a>, Error> {
		if let Some(externals) = externals {
			if let Some(module) = externals.get(name).cloned() {
				return Ok(module);
			}
		}

		self.program
			.upgrade()
			.ok_or(Error::Program("program unloaded".into()))
			.and_then(|p| p.module(name).ok_or(Error::Program(format!("module {} is not loaded", name))))
	}

	/// Get function index.
	pub fn function<'a>(&self, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>, import: &ImportEntry, required_type: Option<FunctionSignature>) -> Result<u32, Error> {
		let (_, export) = self.external_export(externals, import, &required_type.map(|ft| ExportEntryType::Function(ft)).unwrap_or(ExportEntryType::Any))?;
		if let Internal::Function(external_index) = export {
			return Ok(external_index);
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting function)", import.field(), import.module())))
	}

	/// Get table reference.
	pub fn table<'a>(&self, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>, import: &ImportEntry) -> Result<Arc<TableInstance>, Error> {
		let (module, export) = self.external_export(externals, import, &ExportEntryType::Any)?;
		if let Internal::Table(external_index) = export {
			return module.table(ItemIndex::Internal(external_index));
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting table)", import.field(), import.module())))
	}

	/// Get memory reference.
	pub fn memory<'a>(&self, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>, import: &ImportEntry) -> Result<Arc<MemoryInstance>, Error> {
		let (module, export) = self.external_export(externals, import, &ExportEntryType::Any)?;
		if let Internal::Memory(external_index) = export {
			return module.memory(ItemIndex::Internal(external_index));
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting memory)", import.field(), import.module())))
	}

	/// Get global reference.
	pub fn global<'a>(&self, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>, import: &ImportEntry, required_type: Option<VariableType>) -> Result<Arc<VariableInstance>, Error> {
		let (module, export) = self.external_export(externals, import, &required_type.clone().map(|rt| ExportEntryType::Global(rt)).unwrap_or(ExportEntryType::Any))?;
		if let Internal::Global(external_index) = export {
			return module.global(ItemIndex::Internal(external_index), required_type, externals);
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting global)", import.field(), import.module())))
	}

	fn external_export<'a>(&self, externals: Option<&'a HashMap<String, Arc<ModuleInstanceInterface + 'a>>>, import: &ImportEntry, required_type: &ExportEntryType) -> Result<(Arc<ModuleInstanceInterface + 'a>, Internal), Error> {
		self.module(externals, import.module())
			.and_then(|m|
				m.export_entry(import.field(), required_type)
					.map(|e| (m, e)))
	}
}
