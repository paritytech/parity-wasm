//! WebAssembly format library

#![warn(missing_docs)]

extern crate byteorder;

pub mod elements;
pub mod builder;

pub use elements::{
	Error as SerializationError,
	deserialize_buffer,
	deserialize_file,
	serialize,
	serialize_to_file,
	peek_size,
};