#![cfg(test)]

mod run;

#[test_generator::test_resources("testsuite/testsuite/*.wast")]
fn basic(path: &str) {
	run::check(path);
}
