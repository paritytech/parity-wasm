//! WebAssembly format library

#![warn(missing_docs)]

#[macro_use]
extern crate log;
extern crate byteorder;

#[cfg(test)]
extern crate wabt;

pub mod elements;
pub mod builder;
pub mod interpreter;
pub mod validation;
mod common;

pub use elements::{
    Error as SerializationError,
    deserialize_buffer,
    deserialize_file,
    serialize,
    serialize_to_file,
    peek_size,
};
