use std::io;
use super::{Deserialize, Unparsed, Error, VarUint7, VarInt7, VarUint32, VarUint1};

pub enum Type {
    Function(FunctionType),
}

impl Deserialize for Type {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Type::Function(FunctionType::deserialize(reader)?))
    }
}

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

pub struct FunctionType {
    form: u8,
    params: Vec<ValueType>,
    return_type: Option<ValueType>,
}

impl Deserialize for FunctionType {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let form: u8 = VarUint7::deserialize(reader)?.into();
        println!("function form {}", form);
        let param_count: usize = VarUint32::deserialize(reader)?.into();

        println!("type param count {}", param_count);

        let mut params = Vec::new();
        for _ in 0..param_count {
            params.push(ValueType::deserialize(reader)?);
        }

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
