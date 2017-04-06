//! Various builders to generate/alter wasm components

mod invoke;
mod module;
mod code;

pub use self::module::{module, ModuleBuilder};
pub use self::code::function;