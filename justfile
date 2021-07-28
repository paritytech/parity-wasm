_default:
    just --list

# Run rustfmt and ensure the code meets the expectation of the checks in the CI
format:
    cargo +nightly fmt --all

# Run basic checks similar to what the CI does to ensure your code is fine
check:
    cargo +nightly fmt --all -- --check
    cargo +stable clippy --all-targets --all-features -- -D warnings

# Run the tests
test:
    cargo test --all-features

# So you are ready? This runs format, check and test
ready: format check test
