# parity-wasm

[![Build Status](https://travis-ci.org/NikVolf/parity-wasm.svg?branch=master)](https://travis-ci.org/NikVolf/parity-wasm)

[Documentation](https://nikvolf.github.io/parity-wasm/parity_wasm/)

## Rust WebAssembly format serializing/deserializing

```rust

extern crate parity_wasm;

let module = parity-wasm::deserialize_file("./res/cases/v1/hello.wasm");
assert_eq!(module.code_section().is_some());

let code_section = module.code_section().unwrap(); // Part of the module with functions code

println!("Function count in wasm file: {}", code_section.bodies().len());
```

# License

`parity-wasm` is primarily distributed under the terms of both the MIT
license and the Apache License (Version 2.0), at your choice.

See LICENSE-APACHE, and LICENSE-MIT for details.