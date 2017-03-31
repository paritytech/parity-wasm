use std::io;
use super::{Deserialize, Error, VarUint7, VarInt7, VarUint1, CountedList};

pub enum Type {
    Function(FunctionType),
}

impl Deserialize for Type {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Type::Function(FunctionType::deserialize(reader)?))
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl Deserialize for ValueType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let val = VarInt7::deserialize(reader)?;

        match val.into() {
            -0x01 => Ok(ValueType::I32),
            -0x02 => Ok(ValueType::I64),
            -0x03 => Ok(ValueType::F32),
            -0x04 => Ok(ValueType::F64),
            _ => Err(Error::UnknownValueType(val.into())),
        }
    }    
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlockType {
    Value(ValueType),
    NoResult,
}

impl Deserialize for BlockType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let val = VarInt7::deserialize(reader)?;

        match val.into() {
            -0x01 => Ok(BlockType::Value(ValueType::I32)),
            -0x02 => Ok(BlockType::Value(ValueType::I64)),
            -0x03 => Ok(BlockType::Value(ValueType::F32)),
            -0x04 => Ok(BlockType::Value(ValueType::F64)),
            -0x40 => Ok(BlockType::NoResult),
            _ => Err(Error::UnknownValueType(val.into())),
        }
    }    
}


pub struct FunctionType {
    form: u8,
    params: Vec<ValueType>,
    return_type: Option<ValueType>,
}

impl FunctionType {
    pub fn form(&self) -> u8 { self.form }
    pub fn params(&self) -> &[ValueType] { &self.params }
    pub fn return_type(&self) -> Option<ValueType> { self.return_type }
}

impl Deserialize for FunctionType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let form: u8 = VarUint7::deserialize(reader)?.into();

        let params: Vec<ValueType> = CountedList::deserialize(reader)?.into_inner();

        let has_return_type = VarUint1::deserialize(reader)?;
        let return_type = if has_return_type.into() {
            Some(ValueType::deserialize(reader)?)
        } else {
            None
        };

        Ok(FunctionType {
            form: form,
            params: params,
            return_type: return_type,
        })
    }    
}
