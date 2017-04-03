use std::io;
use super::{Deserialize, Error, ValueType, VarUint32, CountedList, Opcodes};


pub struct Local {
    count: u32,
    value_type: ValueType,
}

impl Local {
    pub fn count(&self) -> u32 { self.count }
    pub fn value_type(&self) -> ValueType { self.value_type }
}

impl Deserialize for Local {
     type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let count = VarUint32::deserialize(reader)?;
        let value_type = ValueType::deserialize(reader)?;
        Ok(Local { count: count.into(), value_type: value_type })
    }   
}

pub struct FuncBody {
    locals: Vec<Local>,
    opcodes: Opcodes,
}

impl FuncBody {
    pub fn locals(&self) -> &[Local] { &self.locals }
    pub fn code(&self) -> &Opcodes { &self.opcodes }
}

impl Deserialize for FuncBody {
     type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _body_size = VarUint32::deserialize(reader)?;
        let locals: Vec<Local> = CountedList::deserialize(reader)?.into_inner();
        let opcodes = Opcodes::deserialize(reader)?;
        Ok(FuncBody { locals: locals, opcodes: opcodes })
    }   
}