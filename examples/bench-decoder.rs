extern crate parity_wasm;
extern crate time;

use std::fs;
use time::Instant;

fn rate(file_name: &'static str, iterations: u64) {
	let file_size = fs::metadata(file_name).unwrap_or_else(|_| panic!("{} to exist", file_name)).len();
	let mut total_ms = 0;

	for _ in 0..iterations {
		let start = Instant::now();
		let _module = parity_wasm::deserialize_file(file_name);
		let end = Instant::now();

		total_ms += (end - start).whole_milliseconds();
	}

	println!("Rate for {}: {} MB/s", file_name,
		(file_size as f64 * iterations as f64 / (1024*1024) as f64) /  // total work megabytes
		(total_ms as f64 / 1000f64)									   // total seconds
	);
}

fn main() {
	rate("./res/cases/v1/clang.wasm", 10);
	rate("./res/cases/v1/hello.wasm", 100);
	rate("./res/cases/v1/with_names.wasm", 100);
}