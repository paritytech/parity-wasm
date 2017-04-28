//! Various builders to generate/alter wasm components

mod invoke;
mod module;
mod code;
mod misc;
mod import;
mod memory;

pub use self::module::{module, from_module, ModuleBuilder};
pub use self::code::{signatures, signature, function};
pub use self::import::import;