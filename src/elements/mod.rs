//! Elemets of the WebAssembly binary format.

use std::io;

mod module;
mod section;
mod primitives;
mod types;
mod import_entry;
mod export_entry;
mod global_entry;
mod ops;
mod func;
mod segment;

pub use self::module::Module;
pub use self::section::{
    Section, FunctionsSection, CodeSection, MemorySection, DataSection,
    ImportSection, ExportSection, GlobalSection,
};
pub use self::import_entry::{ImportEntry, MemoryType, TableType, GlobalType, External};
pub use self::export_entry::{ExportEntry, Internal};
pub use self::global_entry::GlobalEntry;
pub use self::primitives::{
    VarUint32, VarUint7, VarUint1, VarInt7, Uint32, 
    Uint64, VarUint64, CountedList, CountedWriter, CountedListWriter,
};
pub use self::types::{ValueType, BlockType, FunctionType};
pub use self::ops::{Opcode, Opcodes, InitExpr};
pub use self::func::{Func, FuncBody, Local};
pub use self::segment::{ElementSegment, DataSegment};

/// Deserialization from serial i/o
pub trait Deserialize : Sized {
    /// Serialization error produced by deserialization routine.
    type Error;
    /// Deserialize type from serial i/o
    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error>;
}

/// Serialization to serial i/o
pub trait Serialize {
    /// Serialization error produced by serialization routine.
    type Error;
    /// Serialize type to serial i/o
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error>;
}

/// Deserialization/serialization error
#[derive(Debug)]
pub enum Error {
    /// Unexpected end of input
    UnexpectedEof,
    /// Inconsistence between declared and actual length
    InconsistentLength { 
        /// Expected length of the definition
        expected: usize, 
        /// Actual length of the definition
        actual: usize 
    },
    /// Other static error
    Other(&'static str),
    /// Other allocated error
    HeapOther(String),
    /// Invalid/unknown value type declaration
    UnknownValueType(i8),
    /// Non-utf8 string
    NonUtf8String,
    /// Unknown external kind code
    UnknownExternalKind(u8),
    /// Unknown internal kind code
    UnknownInternalKind(u8),
    /// Unknown opcode encountered
    UnknownOpcode(u8),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::HeapOther(format!("I/O Error: {}", err))
    }
}

/// Unparsed part of the module/section
pub struct Unparsed(pub Vec<u8>);

impl Deserialize for Unparsed {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let len = VarUint32::deserialize(reader)?.into();
        let mut vec = vec![0u8; len];
        reader.read_exact(&mut vec[..])?;
        Ok(Unparsed(vec))
    }
}

impl From<Unparsed> for Vec<u8> {
    fn from(u: Unparsed) -> Vec<u8> {
        u.0
    }
}

/// Deserialize module from file.
pub fn deserialize_file<P: AsRef<::std::path::Path>>(p: P) -> Result<Module, Error> {
    use std::io::Read;

    let mut contents = Vec::new();
    ::std::fs::File::open(p)?.read_to_end(&mut contents)?;

    deserialize_buffer(contents)
}

/// Deserialize deserializable type from buffer.
pub fn deserialize_buffer<T: Deserialize>(contents: Vec<u8>) -> Result<T, T::Error> {
    let mut reader = io::Cursor::new(contents);
    T::deserialize(&mut reader)
}

/// Create buffer with serialized value.
pub fn serialize<T: Serialize>(val: T) -> Result<Vec<u8>, T::Error> {
    let mut buf = Vec::new();
    val.serialize(&mut buf)?;
    Ok(buf)
}