//! WebAssembly format library

#![warn(missing_docs)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate parking_lot;

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

pub use interpreter::{
    ProgramInstance,
    ModuleInstance,
    ModuleInstanceInterface,
    RuntimeValue,
};
