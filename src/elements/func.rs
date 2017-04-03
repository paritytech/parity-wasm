use std::io;
use super::{
    Deserialize, Error, ValueType, VarUint32, CountedList, Opcodes, 
    Serialize, CountedWriter, CountedListWriter, 
};

/// Function signature (type reference)
pub struct Func(u32);

impl Func {
    pub fn new(type_ref: u32) -> Self { Func(type_ref) }

    pub fn type_ref(&self) -> u32 {
        self.0
    }
}

pub struct Local {
    count: u32,
    value_type: ValueType,
}

impl Local {
    pub fn new(count: u32, value_type: ValueType) -> Self {
        Local { count: count, value_type: value_type }
    }

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

impl Serialize for Local {
    type Error = Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        VarUint32::from(self.count).serialize(writer)?;
        self.value_type.serialize(writer)?;
        Ok(())
    }
}

pub struct FuncBody {
    locals: Vec<Local>,
    opcodes: Opcodes,
}

impl FuncBody {
    pub fn new(locals: Vec<Local>, opcodes: Opcodes) -> Self {
        FuncBody { locals: locals, opcodes: opcodes }
    }

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

impl Serialize for FuncBody {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let mut counted_writer = CountedWriter::new(writer);

        let data = self.locals;
        let counted_list = CountedListWriter::<Local, _>(
            data.len(),
            data.into_iter().map(Into::into),
        );
        counted_list.serialize(&mut counted_writer)?;

        let code = self.opcodes;
        code.serialize(&mut counted_writer)?;

        counted_writer.done()?;

        Ok(())
    }
}