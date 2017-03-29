use std::io;
use super::{Deserialize, Unparsed, Error, VarUint7};

pub struct Section {
    id: u8,
    unparsed: Unparsed,
}

impl Deserialize for Section {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let id = match VarUint7::deserialize(reader) {
            // todo: be more selective detecting no more section
            Err(_) => { return Err(Error::UnexpectedEof); },
            Ok(id) => id,
        };
        let unparsed = Unparsed::deserialize(reader)?;
        Ok(Section {
            id: id.0,
            unparsed: unparsed,
        })
    }    
}