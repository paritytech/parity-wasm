use std::io;
use super::{Deserialize, Error, Uint32};
use super::section::Section;

pub struct Module {
    _magic: u32,
    version: u32,
    sections: Vec<Section>,
}

impl Module {
    pub fn version(&self) -> u32 { self.version }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }
}

impl Deserialize for Module {
    type Error = super::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut sections = Vec::new();

        let magic = Uint32::deserialize(reader)?;

        let version = Uint32::deserialize(reader)?;

        loop {
            match Section::deserialize(reader) {
                Err(Error::UnexpectedEof) => { break; },
                Err(e) => { return Err(e) },
                Ok(section) => { sections.push(section); }
            }
        }

        Ok(Module { 
            _magic: magic.into(),
            version: version.into(),
            sections: sections,
        })
    }    
}

#[cfg(test)]
mod integration_tests {

    use super::super::deserialize_file;

    #[test]
    fn hello() {
        let module = deserialize_file("./res/cases/v1/hello.wasm").expect("Should be deserialized");

        assert_eq!(module.version(), 1);
        assert_eq!(module.sections().len(), 8);
    }
}