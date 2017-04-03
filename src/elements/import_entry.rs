use std::io;
use super::{
    Deserialize, Serialize, Error, VarUint7, VarInt7, VarUint32, VarUint1, 
    ValueType
};

pub struct GlobalType {
    content_type: ValueType,
    is_mutable: bool,
}

impl GlobalType {
    pub fn content_type(&self) -> ValueType { self.content_type }
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

pub struct TableType {
    elem_type: i8,
    limits: ResizableLimits,
}

impl TableType {
    pub fn limits(&self) -> &ResizableLimits { &self.limits }
}

impl Deserialize for TableType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let elem_type = VarInt7::deserialize(reader)?;
        let limits = ResizableLimits::deserialize(reader)?;
        Ok(TableType {
            elem_type: elem_type.into(),
            limits: limits,
        })
    }    
} 

impl Serialize for TableType {
    type Error = Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        VarInt7::from(self.elem_type).serialize(writer)?;
        self.limits.serialize(writer)
    }
}

pub struct ResizableLimits {
    initial: u32,
    maximum: Option<u32>,
}

impl ResizableLimits {
    pub fn initial(&self) -> u32 { self.initial }
    pub fn maximum(&self) -> Option<u32> { self.maximum }
}

impl Deserialize for ResizableLimits {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let has_max = VarUint1::deserialize(reader)?;
        let initial = VarUint32::deserialize(reader)?;
        let maximum = if has_max.into() {
            Some(VarUint32::deserialize(reader)?.into())
        } else {
            None
        };

        Ok(ResizableLimits {
            initial: initial.into(),
            maximum: maximum,
        })
    }    
} 

impl Serialize for ResizableLimits {
    type Error = Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let max = self.maximum;
        VarUint1::from(max.is_some());
        VarUint32::from(self.initial).serialize(writer)?;
        if let Some(val) = max { 
            VarUint32::from(val).serialize(writer)?; 
        }
        Ok(())
    }
}

pub struct MemoryType(ResizableLimits);

impl MemoryType {
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

pub enum External {
    Function(u32),
    Table(TableType),
    Memory(MemoryType),
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

pub struct ImportEntry {
    module_str: String,
    field_str: String,
    external: External,
}

impl ImportEntry {
    pub fn module(&self) -> &str { &self.module_str }
    pub fn field(&self) -> &str { &self.field_str }
    pub fn external(&self) -> &External { &self.external }
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