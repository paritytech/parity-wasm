_default:
    just --list

# Run rustfmt and ensure the code meets the expectation of the checks in the CI
format:
    cargo +nightly fmt --all

# Run basic checks similar to what the CI does to ensure your code is fine
check:
    cargo +stable clippy --all-targets --all-features -- -D warnings
