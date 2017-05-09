//! Various builders to generate/alter wasm components

mod invoke;
mod module;
mod code;
mod misc;
mod import;
mod export;
mod global;

pub use self::module::{module, from_module, ModuleBuilder};
pub use self::code::{signatures, signature, function};
pub use self::import::import;
pub use self::export::export;
pub use self::global::global;