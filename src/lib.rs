//! WebAssembly format library

#![warn(missing_docs)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate unsafe_any;

pub mod elements;
pub mod builder;
pub mod interpreter;
mod validation;
mod common;

pub use elements::{
    Error as SerializationError,
    deserialize_buffer,
    deserialize_file,
    serialize,
    serialize_to_file,
    peek_size,
};

pub use validation::{validate_module, ValidatedModule, Error as ValidationError};

#[allow(deprecated)]
pub use interpreter::{
    ProgramInstance,
    RuntimeValue,
};
