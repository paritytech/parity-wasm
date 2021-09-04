#![cfg(test)]

mod run;

#[test_generator::test_resources("testsuite/spec/*.wast")]
fn basic(path: &str) {
	run::check(path);
}
