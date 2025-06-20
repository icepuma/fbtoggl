# Run all checks
check:
    cargo fmt
    cargo clippy --all-targets --all-features
    cargo nextest run --all-targets