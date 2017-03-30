use std::io;
use super::{Deserialize, Error, VarUint7, VarUint32};

pub enum Internal {
    Function(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

impl Deserialize for Internal {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let kind = VarUint7::deserialize(reader)?;
        match kind.into() {
            0x00 => Ok(Internal::Function(VarUint32::deserialize(reader)?.into())),
            0x01 => Ok(Internal::Table(VarUint32::deserialize(reader)?.into())),
            0x02 => Ok(Internal::Memory(VarUint32::deserialize(reader)?.into())),
            0x03 => Ok(Internal::Global(VarUint32::deserialize(reader)?.into())),
            _ => Err(Error::UnknownInternalKind(kind.into())),
        }
    }    
} 

pub struct ExportEntry {
    field_str: String,
    internal: Internal,
}

impl ExportEntry {
    pub fn field(&self) -> &str { &self.field_str }
    pub fn internal(&self) -> &Internal { &self.internal }
}

impl Deserialize for ExportEntry {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let field_str = String::deserialize(reader)?;
        let internal = Internal::deserialize(reader)?;

        Ok(ExportEntry {
            field_str: field_str,
            internal: internal,
        })
    }    
}