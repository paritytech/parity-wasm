//! Elements of the WebAssembly binary format.

use std::error;
use std::fmt;
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

pub use self::module::{Module, peek_size};
pub use self::section::{
    Section, FunctionSection, CodeSection, MemorySection, DataSection,
    ImportSection, ExportSection, GlobalSection, TypeSection, ElementSection,
    TableSection, CustomSection,
};
pub use self::import_entry::{ImportEntry, ResizableLimits, MemoryType, TableType, GlobalType, External};
pub use self::export_entry::{ExportEntry, Internal};
pub use self::global_entry::GlobalEntry;
pub use self::primitives::{
    VarUint32, VarUint7, VarUint1, VarInt7, Uint32, VarInt32, VarInt64,
    Uint64, VarUint64, CountedList, CountedWriter, CountedListWriter,
};
pub use self::types::{Type, ValueType, BlockType, FunctionType, TableElementType};
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
#[derive(Debug, Clone)]
pub enum Error {
    /// Unexpected end of input
    UnexpectedEof,
    /// Invalid magic
    InvalidMagic,
    /// Unsupported version
    UnsupportedVersion(u32),
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
    /// Invalid/unknown table element type declaration
    UnknownTableElementType(i8),
    /// Non-utf8 string
    NonUtf8String,
    /// Unknown external kind code
    UnknownExternalKind(u8),
    /// Unknown internal kind code
    UnknownInternalKind(u8),
    /// Unknown opcode encountered
    UnknownOpcode(u8),
    /// Invalid VarUint1 value
    InvalidVarUint1(u8),
    /// Invalid VarInt32 value
    InvalidVarInt32,
    /// Invalid VarInt64 value
    InvalidVarInt64,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnexpectedEof => write!(f, "Unexpected end of input"),
            Error::InvalidMagic => write!(f, "Invalid magic number at start of file"),
            Error::UnsupportedVersion(v) => write!(f, "Unsupported wasm version {}", v),
            Error::InconsistentLength { expected, actual } => {
                write!(f, "Expected length {}, found {}", expected, actual)
            }
            Error::Other(msg) => write!(f, "{}", msg),
            Error::HeapOther(ref msg) => write!(f, "{}", msg),
            Error::UnknownValueType(ty) => write!(f, "Invalid or unknown value type {}", ty),
            Error::UnknownTableElementType(ty) => write!(f, "Unknown table element type {}", ty),
            Error::NonUtf8String => write!(f, "Non-UTF-8 string"),
            Error::UnknownExternalKind(kind) => write!(f, "Unknown external kind {}", kind),
            Error::UnknownInternalKind(kind) => write!(f, "Unknown internal kind {}", kind),
            Error::UnknownOpcode(opcode) => write!(f, "Unknown opcode {}", opcode),
            Error::InvalidVarUint1(val) => write!(f, "Not an unsigned 1-bit integer: {}", val),
            Error::InvalidVarInt32 => write!(f, "Not a signed 32-bit integer"),
            Error::InvalidVarInt64 => write!(f, "Not a signed 64-bit integer"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnexpectedEof => "Unexpected end of input",
            Error::InvalidMagic => "Invalid magic number at start of file",
            Error::UnsupportedVersion(_) => "Unsupported wasm version",
            Error::InconsistentLength { .. } => "Inconsistent length",
            Error::Other(msg) => msg,
            Error::HeapOther(ref msg) => &msg[..],
            Error::UnknownValueType(_) => "Invalid or unknown value type",
            Error::UnknownTableElementType(_) => "Unknown table element type",
            Error::NonUtf8String => "Non-UTF-8 string",
            Error::UnknownExternalKind(_) => "Unknown external kind",
            Error::UnknownInternalKind(_) => "Unknown internal kind",
            Error::UnknownOpcode(_) => "Unknown opcode",
            Error::InvalidVarUint1(_) => "Not an unsigned 1-bit integer",
            Error::InvalidVarInt32 => "Not a signed 32-bit integer",
            Error::InvalidVarInt64 => "Not a signed 64-bit integer",
        }
    }
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

    deserialize_buffer(&contents)
}

/// Deserialize deserializable type from buffer.
pub fn deserialize_buffer<T: Deserialize>(contents: &[u8]) -> Result<T, T::Error> {
    let mut reader = io::Cursor::new(contents);
    T::deserialize(&mut reader)
}

/// Create buffer with serialized value.
pub fn serialize<T: Serialize>(val: T) -> Result<Vec<u8>, T::Error> {
    let mut buf = Vec::new();
    val.serialize(&mut buf)?;
    Ok(buf)
}

/// Serialize module to the file
pub fn serialize_to_file<P: AsRef<::std::path::Path>>(p: P, module: Module) -> Result<(), Error>
{
    let mut io = ::std::fs::File::create(p)?;
    module.serialize(&mut io)
}
