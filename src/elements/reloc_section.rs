use std::io::{Read, Write};

use super::{CountedList, CountedListWriter, CountedWriter, Deserialize, Error, Serialize, VarInt32, VarUint32, VarUint7};

const R_WEBASSEMBLY_FUNCTION_INDEX_LEB: u8 = 0;
const R_WEBASSEMBLY_TABLE_INDEX_SLEB: u8 = 1;
const R_WEBASSEMBLY_TABLE_INDEX_I32: u8 = 2;
const R_WEBASSEMBLY_MEMORY_ADDR_LEB: u8 = 3;
const R_WEBASSEMBLY_MEMORY_ADDR_SLEB: u8 = 4;
const R_WEBASSEMBLY_MEMORY_ADDR_I32: u8 = 5;
const R_WEBASSEMBLY_TYPE_INDEX_LEB: u8 = 6;
const R_WEBASSEMBLY_GLOBAL_INDEX_LEB: u8 = 7;

/// Relocation information.
#[derive(Clone, Debug)]
pub struct RelocSection {
	// Name of this section.
	name: String,

	// ID of the section containing the relocations described in this section.
	section_id: u32,

	// Name of the section containing the relocations described in this section. Only set if section_id is 0.
	relocation_section_name: Option<String>,

	// Relocation entries.
	entries: Vec<RelocationEntry>,
}

impl RelocSection {
	pub fn deserialize<R: Read>(
		name: String,
		rdr: &mut R,
	) -> Result<Self, Error> {
		let section_id = VarUint32::deserialize(rdr)?.into();
		let relocation_section_name =
			if section_id == 0 {
				Some(String::deserialize(rdr)?)
			}
			else {
				None
			};

		let entries = CountedList::deserialize(rdr)?.into_inner();

		Ok(RelocSection {
			name,
			section_id,
			relocation_section_name,
			entries,
		})
	}
}

impl Serialize for RelocSection {
	type Error = Error;

	fn serialize<W: Write>(self, wtr: &mut W) -> Result<(), Error> {
		let mut counted_writer = CountedWriter::new(wtr);
		self.name.serialize(&mut counted_writer)?;

		VarUint32::from(self.section_id).serialize(&mut counted_writer)?;

		if let Some(relocation_section_name) = self.relocation_section_name {
			relocation_section_name.serialize(&mut counted_writer)?;
		}

		let counted_list = CountedListWriter(self.entries.len(), self.entries.into_iter());
		counted_list.serialize(&mut counted_writer)?;

		counted_writer.done()?;

		Ok(())
	}
}

/// Relocation entry.
#[derive(Clone, Debug)]
pub enum RelocationEntry {
	// Function index.
	FunctionIndexLeb {
		offset: u32,
		index: u32,
	},

	// Function table index.
	TableIndexSleb {
		offset: u32,
		index: u32,
	},

	// Function table index.
	TableIndexI32 {
		offset: u32,
		index: u32,
	},

	// Linear memory index.
	MemoryAddressLeb {
		offset: u32,
		index: u32,
		addend: i32,
	},

	// Linear memory index.
	MemoryAddressSleb {
		offset: u32,
		index: u32,
		addend: i32,
	},

	// Linear memory index.
	MemoryAddressI32 {
		offset: u32,
		index: u32,
		addend: i32,
	},

	// Type table index.
	TypeIndexLeb {
		offset: u32,
		index: u32,
	},

	// Global index.
	GlobalIndexLeb {
		offset: u32,
		index: u32,
	},
}

impl Deserialize for RelocationEntry {
	type Error = Error;

	fn deserialize<R: Read>(rdr: &mut R) -> Result<Self, Self::Error> {
		match VarUint7::deserialize(rdr)?.into() {
			R_WEBASSEMBLY_FUNCTION_INDEX_LEB => Ok(RelocationEntry::FunctionIndexLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			R_WEBASSEMBLY_TABLE_INDEX_SLEB => Ok(RelocationEntry::TableIndexSleb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			R_WEBASSEMBLY_TABLE_INDEX_I32 => Ok(RelocationEntry::TableIndexI32 {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			R_WEBASSEMBLY_MEMORY_ADDR_LEB => Ok(RelocationEntry::MemoryAddressLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
				addend: VarInt32::deserialize(rdr)?.into(),
			}),

			R_WEBASSEMBLY_MEMORY_ADDR_SLEB => Ok(RelocationEntry::MemoryAddressSleb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
				addend: VarInt32::deserialize(rdr)?.into(),
			}),

			R_WEBASSEMBLY_MEMORY_ADDR_I32 => Ok(RelocationEntry::MemoryAddressI32 {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
				addend: VarInt32::deserialize(rdr)?.into(),
			}),

			R_WEBASSEMBLY_TYPE_INDEX_LEB => Ok(RelocationEntry::TypeIndexLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			R_WEBASSEMBLY_GLOBAL_INDEX_LEB => Ok(RelocationEntry::GlobalIndexLeb {
				offset: VarUint32::deserialize(rdr)?.into(),
				index: VarUint32::deserialize(rdr)?.into(),
			}),

			entry_type => Err(Error::UnknownValueType(entry_type as i8)),
		}
	}
}

impl Serialize for RelocationEntry {
	type Error = Error;

	fn serialize<W: Write>(self, wtr: &mut W) -> Result<(), Error> {
		match self {
			RelocationEntry::FunctionIndexLeb { offset, index } => {
				VarUint7::from(R_WEBASSEMBLY_FUNCTION_INDEX_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::TableIndexSleb { offset, index } => {
				VarUint7::from(R_WEBASSEMBLY_TABLE_INDEX_SLEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::TableIndexI32 { offset, index } => {
				VarUint7::from(R_WEBASSEMBLY_TABLE_INDEX_I32).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::MemoryAddressLeb { offset, index, addend } => {
				VarUint7::from(R_WEBASSEMBLY_MEMORY_ADDR_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
				VarInt32::from(addend).serialize(wtr)?;
			},

			RelocationEntry::MemoryAddressSleb { offset, index, addend } => {
				VarUint7::from(R_WEBASSEMBLY_MEMORY_ADDR_SLEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
				VarInt32::from(addend).serialize(wtr)?;
			},

			RelocationEntry::MemoryAddressI32 { offset, index, addend } => {
				VarUint7::from(R_WEBASSEMBLY_MEMORY_ADDR_I32).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
				VarInt32::from(addend).serialize(wtr)?;
			},

			RelocationEntry::TypeIndexLeb { offset, index } => {
				VarUint7::from(R_WEBASSEMBLY_TYPE_INDEX_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},

			RelocationEntry::GlobalIndexLeb { offset, index } => {
				VarUint7::from(R_WEBASSEMBLY_GLOBAL_INDEX_LEB).serialize(wtr)?;
				VarUint32::from(offset).serialize(wtr)?;
				VarUint32::from(index).serialize(wtr)?;
			},
		}

		Ok(())
	}
}
