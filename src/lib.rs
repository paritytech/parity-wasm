extern crate byteorder;

pub mod elements;

pub use elements::{
    Error as SerializationError,
    deserialize_buffer, 
    deserialize_file
};
