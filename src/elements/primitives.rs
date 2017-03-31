use std::io;
use byteorder::{LittleEndian, ByteOrder};
use super::{Error, Deserialize};

#[derive(Copy, Clone)]
pub struct VarUint32(u32);

impl From<VarUint32> for usize {
    fn from(var: VarUint32) -> usize {
        var.0 as usize
    }
}

impl From<VarUint32> for u32 {
    fn from(var: VarUint32) -> u32 {
        var.0
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


#[derive(Copy, Clone)]
pub struct VarUint64(u64);

impl From<VarUint64> for u64 {
    fn from(var: VarUint64) -> u64 {
        var.0
    }
}

impl Deserialize for VarUint64 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut res = 0;
        let mut shift = 0;
        let mut u8buf = [0u8; 1];
        loop {
            reader.read_exact(&mut u8buf)?;
            let b = u8buf[0] as u64;
            res |= (b & 0x7f) << shift;
            shift += 7;
            if (b >> 7) == 0 {
                break;
            }
        }
        Ok(VarUint64(res))
    }
}

#[derive(Copy, Clone)]
pub struct VarUint7(u8);

impl From<VarUint7> for u8 {
    fn from(v: VarUint7) -> u8 {
        v.0
    }
}

impl Deserialize for VarUint7 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read_exact(&mut u8buf)?;
        Ok(VarUint7(u8buf[0]))
    }
}

#[derive(Copy, Clone)]
pub struct VarInt7(i8);

impl From<VarInt7> for i8 {
    fn from(v: VarInt7) -> i8 {
        v.0
    }
}

impl Deserialize for VarInt7 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read_exact(&mut u8buf)?;
        // expand sign
        if u8buf[0] & 0b0100_0000 == 0b0100_0000 { u8buf[0] |= 0b1000_0000 }
        // todo check range
        Ok(VarInt7(unsafe { ::std::mem::transmute (u8buf[0]) }))
    }
}

#[derive(Copy, Clone)]
pub struct Uint32(u32);

impl Deserialize for Uint32 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        // todo check range
        Ok(Uint32(LittleEndian::read_u32(&buf)))
    }
}

impl From<Uint32> for u32 {
    fn from(var: Uint32) -> u32 {
        var.0
    }
}


#[derive(Copy, Clone)]
pub struct Uint64(u64);

impl Deserialize for Uint64 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        // todo check range
        Ok(Uint64(LittleEndian::read_u64(&buf)))
    }
}

impl From<Uint64> for u64 {
    fn from(var: Uint64) -> u64 {
        var.0
    }
}

#[derive(Copy, Clone)]
pub struct VarUint1(bool);

impl From<VarUint1> for bool {
    fn from(v: VarUint1) -> bool {
        v.0
    }
}

impl Deserialize for VarUint1 {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut u8buf = [0u8; 1];
        reader.read_exact(&mut u8buf)?;
        // todo check range
        Ok(VarUint1(u8buf[0] == 1))
    }
}

impl Deserialize for String {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let length = VarUint32::deserialize(reader)?.into();
        if length > 0 {
            let mut buf = vec![0u8; length];
            reader.read_exact(&mut buf)?;
            String::from_utf8(buf).map_err(|_| Error::NonUtf8String)
        }
        else {
            Ok(String::new())
        }
    }
}

pub struct CountedList<T: Deserialize>(Vec<T>);

impl<T: Deserialize> CountedList<T> {
    pub fn into_inner(self) -> Vec<T> { self.0 }
}

impl<T: Deserialize> Deserialize for CountedList<T> where T::Error : From<Error> {
    type Error = T::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let count: usize = VarUint32::deserialize(reader)?.into();
        let mut result = Vec::new();
        for _ in 0..count { result.push(T::deserialize(reader)?); }
        Ok(CountedList(result))
    }
}

#[cfg(test)]
mod tests {

    use super::super::deserialize_buffer;
    use super::{CountedList, VarInt7};

    #[test]
    fn counted_list() {
        let payload = vec![
            133u8, //(128+5), length is 5
                0x80, 0x80, 0x80, 0x0, // padding
            0x01, 
            0x7d,
            0x05,
            0x07,
            0x09,
        ];

        let list: CountedList<VarInt7> = 
            deserialize_buffer(payload).expect("type_section be deserialized");

        let vars = list.into_inner();
        assert_eq!(5, vars.len());
        let v3: i8 = (*vars.get(1).unwrap()).into();
        assert_eq!(-0x03i8, v3);
    }
}
