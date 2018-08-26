use io;
use std::string::String;
use super::{
	Deserialize, Serialize, Error, VarUint7, VarInt7, VarUint32, VarUint1, Uint8,
	ValueType, TableElementType
};

/// Global definition struct
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlobalType {
	content_type: ValueType,
	is_mutable: bool,
}

impl GlobalType {
	/// New global type
	pub fn new(content_type: ValueType, is_mutable: bool) -> Self {
		GlobalType {
			content_type: content_type,
			is_mutable: is_mutable,
		}
	}

	/// Type of the global entry
	pub fn content_type(&self) -> ValueType { self.content_type }

	/// Is global entry is declared as mutable
	pub fn is_mutable(&self) -> bool { self.is_mutable }
}

impl Deserialize for GlobalType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let content_type = ValueType::deserialize(reader)?;
		let is_mutable = VarUint1::deserialize(reader)?;
		Ok(GlobalType {
			content_type: content_type,
			is_mutable: is_mutable.into(),
		})
	}
}

impl Serialize for GlobalType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.content_type.serialize(writer)?;
		VarUint1::from(self.is_mutable).serialize(writer)?;
		Ok(())
	}
}

/// Table entry
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TableType {
	elem_type: TableElementType,
	limits: ResizableLimits,
}

impl TableType {
	/// New table definition
	pub fn new(min: u32, max: Option<u32>) -> Self {
		TableType {
			elem_type: TableElementType::AnyFunc,
			limits: ResizableLimits::new(min, max),
		}
	}

	/// Table memory specification
	pub fn limits(&self) -> &ResizableLimits { &self.limits }

	/// Table element type
	pub fn elem_type(&self) -> TableElementType { self.elem_type }
}

impl Deserialize for TableType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let elem_type = TableElementType::deserialize(reader)?;
		let limits = ResizableLimits::deserialize(reader)?;
		Ok(TableType {
			elem_type: elem_type,
			limits: limits,
		})
	}
}

impl Serialize for TableType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.elem_type.serialize(writer)?;
		self.limits.serialize(writer)
	}
}

/// Memory limits
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ResizableLimits {
	initial: u32,
	maximum: Option<u32>,
	flags: u8,
}

impl ResizableLimits {
	/// New memory limits definition
	pub fn new(min: u32, max: Option<u32>) -> Self {
		ResizableLimits {
			initial: min,
			maximum: max,
			flags: 0,
		}
	}
	/// Initial size
	pub fn initial(&self) -> u32 { self.initial }
	/// Maximum size
	pub fn maximum(&self) -> Option<u32> { self.maximum }
	/// Whether or not this is a shared array buffer
	pub fn shared(&self) -> bool { self.flags & 0x2 != 0 }
}

impl Deserialize for ResizableLimits {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let flags: u8 = Uint8::deserialize(reader)?.into();
		let initial = VarUint32::deserialize(reader)?;
		let maximum = if flags & 0x1 != 0 {
			Some(VarUint32::deserialize(reader)?.into())
		} else {
			None
		};

		Ok(ResizableLimits {
			initial: initial.into(),
			maximum: maximum,
			flags,
		})
	}
}

impl Serialize for ResizableLimits {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		let max = self.maximum;
		let flags = self.flags & !0x1 | (if max.is_some() { 0x1 } else { 0x0 });
		Uint8::from(flags).serialize(writer)?;
		VarUint32::from(self.initial).serialize(writer)?;
		if let Some(val) = max {
			VarUint32::from(val).serialize(writer)?;
		}
		Ok(())
	}
}

/// Memory entry.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MemoryType(ResizableLimits);

impl MemoryType {
	/// New memory definition
	pub fn new(min: u32, max: Option<u32>) -> Self {
		MemoryType(ResizableLimits::new(min, max))
	}
	/// Limits of the memory entry.
	pub fn limits(&self) -> &ResizableLimits {
		&self.0
	}
}

impl Deserialize for MemoryType {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		Ok(MemoryType(ResizableLimits::deserialize(reader)?))
	}
}

impl Serialize for MemoryType {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.0.serialize(writer)
	}
}

/// External to local binding.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum External {
	/// Binds to function with index.
	Function(u32),
	/// Describes local table definition to be imported as.
	Table(TableType),
	/// Describes local memory definition to be imported as.
	Memory(MemoryType),
	/// Describes local global entry to be imported as.
	Global(GlobalType),
}

impl Deserialize for External {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let kind = VarUint7::deserialize(reader)?;
		match kind.into() {
			0x00 => Ok(External::Function(VarUint32::deserialize(reader)?.into())),
			0x01 => Ok(External::Table(TableType::deserialize(reader)?)),
			0x02 => Ok(External::Memory(MemoryType::deserialize(reader)?)),
			0x03 => Ok(External::Global(GlobalType::deserialize(reader)?)),
			_ => Err(Error::UnknownExternalKind(kind.into())),
		}
	}
}

impl Serialize for External {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use self::External::*;

		match self {
			Function(index) => {
				VarUint7::from(0x00).serialize(writer)?;
				VarUint32::from(index).serialize(writer)?;
			},
			Table(tt) => {
				VarInt7::from(0x01).serialize(writer)?;
				tt.serialize(writer)?;
			},
			Memory(mt) => {
				VarInt7::from(0x02).serialize(writer)?;
				mt.serialize(writer)?;
			},
			Global(gt) => {
				VarInt7::from(0x03).serialize(writer)?;
				gt.serialize(writer)?;
			},
		}

		Ok(())
	}
}

/// Import entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportEntry {
	module_str: String,
	field_str: String,
	external: External,
}

impl ImportEntry {
	/// New import entry.
	pub fn new(module_str: String, field_str: String, external: External) -> Self {
		ImportEntry {
			module_str: module_str,
			field_str: field_str,
			external: external,
		}
	}

	/// Module reference of the import entry.
	pub fn module(&self) -> &str { &self.module_str }

	/// Module reference of the import entry (mutable).
	pub fn module_mut(&mut self) -> &mut String {
		&mut self.module_str
	}

	/// Field reference of the import entry.
	pub fn field(&self) -> &str { &self.field_str }

	/// Field reference of the import entry (mutable)
	pub fn field_mut(&mut self) -> &mut String {
		&mut self.field_str
	}

	/// Local binidng of the import entry.
	pub fn external(&self) -> &External { &self.external }

	/// Local binidng of the import entry (mutable)
	pub fn external_mut(&mut self) -> &mut External { &mut self.external }
}

impl Deserialize for ImportEntry {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let module_str = String::deserialize(reader)?;
		let field_str = String::deserialize(reader)?;
		let external = External::deserialize(reader)?;

		Ok(ImportEntry {
			module_str: module_str,
			field_str: field_str,
			external: external,
		})
	}
}

impl Serialize for ImportEntry {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		self.module_str.serialize(writer)?;
		self.field_str.serialize(writer)?;
		self.external.serialize(writer)
	}
}
