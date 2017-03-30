use std::io;
use super::{Deserialize, Unparsed, Error, VarUint7, VarUint32, CountedList, ImportEntry};
use super::types::Type;

pub enum Section {
    Unparsed {
        id: u8,
        payload: Vec<u8>,
    },
    Custom(Vec<u8>),
    Type(TypeSection),
    Import(ImportSection),
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
                2 => {
                    Section::Import(ImportSection::deserialize(reader)?)
                },
                _ => {
                    Section::Unparsed { id: id.into(), payload: Unparsed::deserialize(reader)?.into() }
                }
            }
        )
    }    
}

pub struct TypeSection(Vec<Type>);

impl TypeSection {
    pub fn types(&self) -> &[Type] {
        &self.0
    }
}

impl Deserialize for TypeSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let types: Vec<Type> = CountedList::deserialize(reader)?.into_inner();
        Ok(TypeSection(types))
    }   
}

pub struct ImportSection(Vec<ImportEntry>);

impl ImportSection {
    pub fn entries(&self) -> &[ImportEntry] {
        &self.0
    }
}

impl Deserialize for ImportSection {
     type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<ImportEntry> = CountedList::deserialize(reader)?.into_inner();
        Ok(ImportSection(entries))
    }   
}

#[cfg(test)]
mod tests {

    use super::super::{deserialize_buffer, deserialize_file};
    use super::{Section, TypeSection, Type};

    #[test]
    fn import_section() {
        let module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        let mut found = false;
        for section in module.sections() {
            match section {
                &Section::Import(ref import_section) => { 
                    assert_eq!(25, import_section.entries().len());
                    found = true
                },
                _ => { }
            }
        }
        assert!(found, "There should be import section in test5.wasm");
    }

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