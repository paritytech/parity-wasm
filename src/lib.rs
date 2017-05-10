//! WebAssembly format library

#![warn(missing_docs)]

extern crate byteorder;
extern crate parking_lot;

pub mod elements;
pub mod builder;
pub mod interpreter;

pub use elements::{
    Error as SerializationError,
    deserialize_buffer, 
    deserialize_file,
    serialize,
    serialize_to_file,
};

pub use interpreter::{
    ProgramInstance,
    ModuleInstance,
    ModuleInstanceInterface,
    RuntimeValue,
};
