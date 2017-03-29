use std::io;

mod module;
mod section;

pub use self::module::Module;
pub use self::section::Section;

use byteorder::{LittleEndian, ByteOrder};

pub trait Deserialize : Sized {
    type Error;
    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error>;
}

#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    InconsistentLength { expected: usize, actual: usize },
    Other(&'static str),
    HeapOther(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::HeapOther(format!("I/O Error: {}", err))
    }
}

struct Unparsed(pub Vec<u8>);

impl Deserialize for Unparsed {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let len = VarUint32::deserialize(reader)?.into();
        let vec = vec![0u8; len];
        Ok(Unparsed(vec))
    }
}

struct VarUint32(u32);

impl From<VarUint32> for usize {
    fn from(var: VarUint32) -> usize {
        var.0 as usize
    }
}

impl Deserialize for VarUint32 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut res = 0;
        let mut shift = 0;
        let mut u8buf = [0u8; 1];
        loop {
            reader.read_exact(&mut u8buf)?;
            let b = u8buf[0] as u32;
            res |= (b & 0x7f) << shift;
            shift += 7;
            if (b >> 7) == 0 {
                break;
            }
        }
        Ok(VarUint32(res))
    }
}

struct VarUint7(u8);

impl Deserialize for VarUint7 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read_exact(&mut u8buf)?;
        Ok(VarUint7(u8buf[0]))
    }
}

struct Uint32(u32);

impl Deserialize for Uint32 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        ;
        Ok(Uint32(LittleEndian::read_u32(&buf)))
    }
}
