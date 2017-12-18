#![allow(deprecated)]

#[cfg_attr(test, macro_use)] #[cfg(test)] extern crate serde_derive;
extern crate parity_wasm;
extern crate serde;
extern crate serde_json;

mod run;
mod test;
mod fixtures;
