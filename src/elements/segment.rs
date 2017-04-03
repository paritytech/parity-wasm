use std::io;
use super::{Deserialize, Serialize, Error, VarUint32, CountedList, InitExpr};

pub struct ElementSegment {
    index: u32,
    offset: InitExpr,
    members: Vec<u32>,
}

impl ElementSegment {
    pub fn members(&self) -> &[u32] { &self.members }
    pub fn index(&self) -> u32 { self.index }
    pub fn offset(&self) -> &InitExpr { &self.offset }
}

impl Deserialize for ElementSegment {
     type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = VarUint32::deserialize(reader)?;
        let offset = InitExpr::deserialize(reader)?;
        let funcs: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
            .into_inner()
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(ElementSegment { 
            index: index.into(), 
            offset: offset,  
            members: funcs,
        })
    }   
}

pub struct DataSegment {
    index: u32,
    offset: InitExpr,
    value: Vec<u8>,
}

impl DataSegment {
    pub fn index(&self) -> u32 { self.index }
    pub fn offset(&self) -> &InitExpr { &self.offset }   
    pub fn value(&self) -> &[u8] { &self.value }
}

impl Deserialize for DataSegment {
     type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = VarUint32::deserialize(reader)?;
        let offset = InitExpr::deserialize(reader)?;
        let value_len = VarUint32::deserialize(reader)?;

        let mut value_buf = vec![0u8; value_len.into()];
        reader.read_exact(&mut value_buf[..])?;

        Ok(DataSegment { 
            index: index.into(), 
            offset: offset,  
            value: value_buf,
        })
    }   
}

impl Serialize for DataSegment {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        VarUint32::from(self.index).serialize(writer)?;
        self.offset.serialize(writer)?;
        writer.write_all(&self.value[..])?;
        Ok(())
    }
}