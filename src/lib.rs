//! WebAssembly format library

#![warn(missing_docs)]

#[macro_use]
extern crate log;
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
    CustomProgramInstance,
    ModuleInstance,
    CustomModuleInstance,
    ModuleInstanceInterface,
    CustomModuleInstanceInterface,
    RuntimeValue,
};
