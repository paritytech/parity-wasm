#![cfg(test)]

use std::ffi::OsStr;

mod run;

const BASIC_BLACKLIST: [&str; 8] = [
	// those use unsupported i32_trunc_sat_* instructions
	"binary-leb128.wast",
	"conversions.wast",
	// those use multi-value for blocks which is unsupported
	"block.wast",
	"func.wast",
	"if.wast",
	"loop.wast",
	"fac.wast",
	"br.wast",
];

#[test_generator::test_resources("testsuite/spec/*.wast")]
fn basic(path: &str) {
	let blacklisted = std::path::Path::new(path)
		.file_name()
		.map(|file| BASIC_BLACKLIST.iter().any(|black| OsStr::new(black) == file))
		.unwrap_or(false);

	if !blacklisted {
		run::check(path);
	}
}
