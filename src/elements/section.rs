use std::io;
use super::{Deserialize, Unparsed, Error, VarUint7};

pub struct Section {
    id: u8,
    unparsed: Unparsed,
}

impl Deserialize for Section {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let id = VarUint7::deserialize(reader)?;
        let unparsed = Unparsed::deserialize(reader)?;

        Ok(Section {
            id: id.0,
            unparsed: unparsed,
        })
    }    
}