//! Various builders to generate/alter wasm components

mod invoke;
mod module;
mod code;
mod misc;
mod import;

pub use self::module::{module, ModuleBuilder};
pub use self::code::{signatures, function};
pub use self::import::import;