use std::io;
use super::{Deserialize, Serialize, Error, Uint32};
use super::section::{Section, CodeSection, TypeSection, ImportSection};

/// WebAssembly module
pub struct Module {
    magic: u32,
    version: u32,
    sections: Vec<Section>,
}

impl Module {
    /// Version of module.
    pub fn version(&self) -> u32 { self.version }

    /// Sections list.
    /// Each known section is optional and may appear at most once.
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Sections list (mutable)
    /// Each known section is optional and may appear at most once.
    pub fn sections_mut(&mut self) -> &mut Vec<Section> {
        &mut self.sections
    }

    /// Code section, if any.
    pub fn code_section(&self) -> Option<&CodeSection> {
        for section in self.sections() {
            if let &Section::Code(ref code_section) = section { return Some(code_section); }
        }
        None
    }

    /// Types section, if any.
    pub fn type_section(&self) -> Option<&TypeSection> {
        for section in self.sections() {
            if let &Section::Type(ref type_section) = section { return Some(type_section); }
        }
        None
    }

    /// Imports section, if any.
    pub fn import_section(&self) -> Option<&ImportSection> {
        for section in self.sections() {
            if let &Section::Import(ref import_section) = section { return Some(import_section); }
        }
        None
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
            magic: magic.into(),
            version: version.into(),
            sections: sections,
        })
    }    
}

impl Serialize for Module {
    type Error = Error;

    fn serialize<W: io::Write>(self, w: &mut W) -> Result<(), Self::Error> {
        Uint32::from(self.magic).serialize(w)?;
        Uint32::from(self.version).serialize(w)?;
        for section in self.sections.into_iter() {
            section.serialize(w)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {

    use super::super::{deserialize_file, serialize, deserialize_buffer, Section};
    use super::Module;

    #[test]
    fn hello() {
        let module = deserialize_file("./res/cases/v1/hello.wasm").expect("Should be deserialized");

        assert_eq!(module.version(), 1);
        assert_eq!(module.sections().len(), 8);
    }

    #[test]
    fn serde() {
        let module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        let buf = serialize(module).expect("serialization to succeed");

        let module_new: Module = deserialize_buffer(buf).expect("deserialization to succeed");
        let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");

        assert_eq!(module_old.sections().len(), module_new.sections().len());
    }

    #[test]
    fn serde_type() {
        let mut module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        module.sections_mut().retain(|x| {
            if let &Section::Type(_) = x { true } else { false }
        });

        let buf = serialize(module).expect("serialization to succeed");

        let module_new: Module = deserialize_buffer(buf).expect("deserialization to succeed");
        let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        assert_eq!(
            module_old.type_section().expect("type section exists").types().len(),
            module_new.type_section().expect("type section exists").types().len(),
            "There should be equal amount of types before and after serialization"
        );
    }

    #[test]
    fn serde_import() {
        let mut module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        module.sections_mut().retain(|x| {
            if let &Section::Import(_) = x { true } else { false }
        });

        let buf = serialize(module).expect("serialization to succeed");

        let module_new: Module = deserialize_buffer(buf).expect("deserialization to succeed");
        let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        assert_eq!(
            module_old.import_section().expect("import section exists").entries().len(),
            module_new.import_section().expect("import section exists").entries().len(),
            "There should be equal amount of import entries before and after serialization"
        );
    }    

    #[test]
    fn serde_code() {
        let mut module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        module.sections_mut().retain(|x| {
            if let &Section::Code(_) = x { true } else { false }
        });

        let buf = serialize(module).expect("serialization to succeed");

        let module_new: Module = deserialize_buffer(buf).expect("deserialization to succeed");
        let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        assert_eq!(
            module_old.code_section().expect("code section exists").bodies().len(),
            module_new.code_section().expect("code section exists").bodies().len(),
            "There should be equal amount of function bodies before and after serialization"
        );
    }
}