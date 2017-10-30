use std::io;
use byteorder::{LittleEndian, ByteOrder};

use super::{Deserialize, Serialize, Error, Uint32};
use super::section::{
    Section, CodeSection, TypeSection, ImportSection, ExportSection, FunctionSection,
    GlobalSection, TableSection, ElementSection, DataSection, MemorySection
};

const WASM_MAGIC_NUMBER: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

/// WebAssembly module
#[derive(Clone)]
pub struct Module {
    magic: u32,
    version: u32,
    sections: Vec<Section>,
}

impl Default for Module {
    fn default() -> Self {
        Module {
            magic: 0x6d736100,
            version: 1,
            sections: Vec::with_capacity(16),
        }
    }
}

impl Module {
    /// New module with sections
    pub fn new(sections: Vec<Section>) -> Self {
        Module {
            sections: sections, ..Default::default()
        }
    }

    /// Destructure the module, yielding sections
    pub fn into_sections(self) -> Vec<Section> {
        self.sections
    }

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

    /// Globals section, if any.
    pub fn global_section(&self) -> Option<&GlobalSection> {
        for section in self.sections() {
            if let &Section::Global(ref section) = section { return Some(section); }
        }
        None
    }

    /// Exports section, if any.
    pub fn export_section(&self) -> Option<&ExportSection> {
        for section in self.sections() {
            if let &Section::Export(ref export_section) = section { return Some(export_section); }
        }
        None
    }

    /// Table section, if any.
    pub fn table_section(&self) -> Option<&TableSection> {
        for section in self.sections() {
            if let &Section::Table(ref section) = section { return Some(section); }
        }
        None
    }

    /// Data section, if any.
    pub fn data_section(&self) -> Option<&DataSection> {
        for section in self.sections() {
            if let &Section::Data(ref section) = section { return Some(section); }
        }
        None
    }

    /// Element section, if any.
    pub fn elements_section(&self) -> Option<&ElementSection> {
        for section in self.sections() {
            if let &Section::Element(ref section) = section { return Some(section); }
        }
        None
    }

    /// Memory section, if any.
    pub fn memory_section(&self) -> Option<&MemorySection> {
        for section in self.sections() {
            if let &Section::Memory(ref section) = section { return Some(section); }
        }
        None
    }

    /// Functions signatures section, if any.
    pub fn function_section(&self) -> Option<&FunctionSection> {
        for section in self.sections() {
            if let &Section::Function(ref sect) = section { return Some(sect); }
        }
        None
    }

    /// Start section, if any.
    pub fn start_section(&self) -> Option<u32> {
        for section in self.sections() {
            if let &Section::Start(sect) = section { return Some(sect); }
        }
        None
    }
}

impl Deserialize for Module {
    type Error = super::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut sections = Vec::new();

        let mut magic = [0u8; 4];
        reader.read(&mut magic)?;
        if magic != WASM_MAGIC_NUMBER {
            return Err(Error::InvalidMagic);
        }

        let version: u32 = Uint32::deserialize(reader)?.into();

        if version != 1 {
            return Err(Error::UnsupportedVersion(version));
        }

        loop {
            match Section::deserialize(reader) {
                Err(Error::UnexpectedEof) => { break; },
                Err(e) => { return Err(e) },
                Ok(section) => { sections.push(section); }
            }
        }

        Ok(Module {
            magic: LittleEndian::read_u32(&magic),
            version: version,
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

struct PeekSection<'a> {
    cursor: usize,
    region: &'a [u8],
}

impl<'a> io::Read for PeekSection<'a> {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        let available = ::std::cmp::min(buf.len(), self.region.len() - self.cursor);
        if available < buf.len() {
            return Err(::std::io::Error::from(::std::io::ErrorKind::UnexpectedEof));
        }

        let range = self.cursor..self.cursor + buf.len();
        buf.copy_from_slice(&self.region[range]);

        self.cursor += available;
        Ok(available)
    }
}

/// Returns size of the module in the provided stream
pub fn peek_size(source: &[u8]) -> usize {
    if source.len() < 9 {
        return 0;
    }

    let mut cursor = 8;
    loop {
        let (new_cursor, section_id, section_len) = {
            let mut peek_section = PeekSection { cursor: 0, region: &source[cursor..] };
            let section_id: u8 = match super::VarUint7::deserialize(&mut peek_section) {
                Ok(res) => res.into(),
                Err(_) => { break; },
            };
            let section_len: u32 = match super::VarUint32::deserialize(&mut peek_section) {
                Ok(res) => res.into(),
                Err(_) => { break; },
            };

            (peek_section.cursor, section_id, section_len)
        };

        if section_id <= 11 && section_len > 0 {
            let next_cursor = cursor + new_cursor + section_len as usize;
            if next_cursor >= source.len() {
                break;
            }
            cursor = next_cursor;
        } else {
            break;
        }
    }

    cursor
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

    #[test]
    fn const_() {
        use super::super::Opcode::*;

        let module = deserialize_file("./res/cases/v1/const.wasm").expect("Should be deserialized");
        let func = &module.code_section().expect("Code section to exist").bodies()[0];
        assert_eq!(func.code().elements().len(), 20);

        assert_eq!(I64Const(9223372036854775807), func.code().elements()[0]);
        assert_eq!(I64Const(-9223372036854775808), func.code().elements()[1]);
        assert_eq!(I64Const(-1152894205662152753), func.code().elements()[2]);
        assert_eq!(I64Const(-8192), func.code().elements()[3]);
        assert_eq!(I32Const(1024), func.code().elements()[4]);
        assert_eq!(I32Const(2048), func.code().elements()[5]);
        assert_eq!(I32Const(4096), func.code().elements()[6]);
        assert_eq!(I32Const(8192), func.code().elements()[7]);
        assert_eq!(I32Const(16384), func.code().elements()[8]);
        assert_eq!(I32Const(32767), func.code().elements()[9]);
        assert_eq!(I32Const(-1024), func.code().elements()[10]);
        assert_eq!(I32Const(-2048), func.code().elements()[11]);
        assert_eq!(I32Const(-4096), func.code().elements()[12]);
        assert_eq!(I32Const(-8192), func.code().elements()[13]);
        assert_eq!(I32Const(-16384), func.code().elements()[14]);
        assert_eq!(I32Const(-32768), func.code().elements()[15]);
        assert_eq!(I32Const(-2147483648), func.code().elements()[16]);
        assert_eq!(I32Const(2147483647), func.code().elements()[17]);
    }

    #[test]
    fn store() {
        use super::super::Opcode::*;

        let module = deserialize_file("./res/cases/v1/offset.wasm").expect("Should be deserialized");
        let func = &module.code_section().expect("Code section to exist").bodies()[0];

        assert_eq!(func.code().elements().len(), 5);
        assert_eq!(I64Store(0, 32), func.code().elements()[2]);
    }

    #[test]
    fn peek() {
        use super::peek_size;

        let module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        let mut buf = serialize(module).expect("serialization to succeed");

        buf.extend_from_slice(&[1, 5, 12, 17]);

        assert_eq!(peek_size(&buf), buf.len() - 4);
    }


    #[test]
    fn peek_2() {
        use super::peek_size;

        let module = deserialize_file("./res/cases/v1/offset.wasm").expect("Should be deserialized");
        let mut buf = serialize(module).expect("serialization to succeed");

        buf.extend_from_slice(&[0, 0, 0, 0, 0, 1, 5, 12, 17]);

        assert_eq!(peek_size(&buf), buf.len() - 9);
    }
}
