use std::io;
use super::{Deserialize, VarUint32, Error, Uint32};
use super::section::Section;

pub struct Module {
    magic: u32,
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
            magic: magic.0,
            version: version.0,
            sections: sections,
        })
    }    
}

#[cfg(test)]
mod integration_tests {

    use std::io::{self, Read};
    use std::fs::File;

    use super::super::Deserialize;
    use super::Module;
    
    #[test]
    fn hello() {
        let mut contents = Vec::new();
        File::open("./res/cases/v1/hello.wasm")
            .expect("readable file")
            .read_to_end(&mut contents)
            .expect("read succeeds");
        
        let mut reader = io::Cursor::new(contents);
        let module = Module::deserialize(&mut reader).expect("Should be deserialized");

        assert_eq!(module.version(), 1);
        assert_eq!(module.sections().len(), 8);
    }
}