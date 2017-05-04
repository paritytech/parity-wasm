use std::sync::{Arc, Weak};
use elements::{ImportSection, ImportEntry, External, Internal};
use interpreter::Error;
use interpreter::memory::MemoryInstance;
use interpreter::module::{ModuleInstanceInterface, ItemIndex};
use interpreter::program::ProgramInstanceEssence;
use interpreter::table::TableInstance;
use interpreter::variable::VariableInstance;

// TODO: cache Internal-s to fasten access
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
	pub fn module(&self, name: &str) -> Result<Arc<ModuleInstanceInterface>, Error> {
		self.program
			.upgrade()
			.ok_or(Error::Program("program unloaded".into()))
			.and_then(|p| p.module(name).ok_or(Error::Program(format!("module {} is not loaded", name))))
	}

	/// Get function index.
	pub fn function(&self, import: &ImportEntry) -> Result<u32, Error> {
		let (_, export) = self.external_export(import)?;
		if let Internal::Function(external_index) = export {
			return Ok(external_index);
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting function)", import.field(), import.module())))
	}

	/// Get table reference.
	pub fn table(&self, import: &ImportEntry) -> Result<Arc<TableInstance>, Error> {
		let (module, export) = self.external_export(import)?;
		if let Internal::Table(external_index) = export {
			return module.table(ItemIndex::Internal(external_index));
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting table)", import.field(), import.module())))
	}

	/// Get memory reference.
	pub fn memory(&self, import: &ImportEntry) -> Result<Arc<MemoryInstance>, Error> {
		let (module, export) = self.external_export(import)?;
		if let Internal::Memory(external_index) = export {
			return module.memory(ItemIndex::Internal(external_index));
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting memory)", import.field(), import.module())))
	}

	/// Get global reference.
	pub fn global(&self, import: &ImportEntry) -> Result<Arc<VariableInstance>, Error> {
		let (module, export) = self.external_export(import)?;
		if let Internal::Global(external_index) = export {
			return module.global(ItemIndex::Internal(external_index));
		}

		Err(Error::Program(format!("wrong import {} from module {} (expecting global)", import.field(), import.module())))
	}

	fn external_export(&self, import: &ImportEntry) -> Result<(Arc<ModuleInstanceInterface>, Internal), Error> {
		self.module(import.module())
			.and_then(|m| m.module().export_section()
				.ok_or(Error::Program(format!("trying to import from module {} without export section", import.module())))
				.and_then(|s| s.entries().iter()
					.find(|e| e.field() == import.field())
					.map(|e| e.internal())
					.ok_or(Error::Program(format!("unresolved import {} from module {}", import.field(), import.module())))
				.map(|export| (m.clone(), *export))))
	}
}
