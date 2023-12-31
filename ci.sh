#!/bin/sh

export RUSTFLAGS="-Dwarnings"

echo "== Build:"
cargo clean
cargo build --all-targets --all-features
echo "== Test:"
cargo test --all-targets --all-features
echo "== Lint:"
cargo clippy --all-targets --all-features
echo "== Format:"
cargo fmt --all -- --check

echo "Completed!"
