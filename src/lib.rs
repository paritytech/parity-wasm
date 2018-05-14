//! WebAssembly format library

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#![warn(missing_docs)]

extern crate byteorder;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

pub mod elements;
pub mod builder;
mod io;

pub use elements::{
	Error as SerializationError,
	deserialize_buffer,
	serialize,
	peek_size,
};

#[cfg(feature = "std")]
pub use elements::{
	deserialize_file,
	serialize_to_file,
};



#[cfg(not(feature = "std"))]
mod std {
	pub use core::*;
	pub use alloc::vec;
	pub use alloc::string;
	pub use alloc::boxed;
	pub use alloc::borrow;
}
