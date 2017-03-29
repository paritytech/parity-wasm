use std::io;
use super::{Deserialize, Unparsed, Error, VarUint7, VarUint32, CountedList};
use super::types::Type;

pub enum Section {
    Unparsed {
        id: u8,
        payload: Vec<u8>,
    },
    Custom(Vec<u8>),
    Type(TypeSection),
}

impl Deserialize for Section {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let id = match VarUint7::deserialize(reader) {
            // todo: be more selective detecting no more section
            Err(_) => { return Err(Error::UnexpectedEof); },
            Ok(id) => id,
        };

        Ok(
            match id.into() {
                0 => {
                    Section::Custom(Unparsed::deserialize(reader)?.into())
                },
                1 => {
                    Section::Type(TypeSection::deserialize(reader)?)
                },
                _ => {
                    Section::Unparsed { id: id.into(), payload: Unparsed::deserialize(reader)?.into() }
                }
            }
        )
    }    
}

pub struct TypeSection {
    types: Vec<Type>,
}

impl TypeSection {
    fn types(&self) -> &[Type] {
        &self.types
    }
}

impl Deserialize for TypeSection {
     type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let types: Vec<Type> = CountedList::deserialize(reader)?.into_inner();
        Ok(TypeSection { types: types })
    }   
}

#[cfg(test)]
mod tests {

    use super::super::{deserialize_buffer};
    use super::{TypeSection, Type};

    #[test]
    fn type_section() {
        let payload = vec![
            129u8, 0x80, 0x80, 0x80, 0x0,
            // func 1
            // form=1
            0x01, 
            // param_count=1
            129u8, 0x80, 0x80, 0x80, 0x0,
                // first param
                0x7e, // i64
            // no return params
            0u8
        ];

        let type_section: TypeSection = 
            deserialize_buffer(payload).expect("type_section be deserialized");

        assert_eq!(type_section.types().len(), 1);
        match type_section.types()[0] {
            Type::Function(_) => {},
            _ => panic!("Type should be a function")
        }
    }

}