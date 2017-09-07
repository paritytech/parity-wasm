extern crate cmake;
use cmake::Config;

fn main() {
    let _dst = Config::new("binaryen")
        .build();
}
