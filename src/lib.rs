extern crate byteorder;

mod elements;

pub use elements::{Section, Module, Error as DeserializeError, deserialize_buffer, deserialize_file};
