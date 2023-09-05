$env:RUSTFLAGS="-Dwarnings"

Write-Host "== Build:"
& cargo clean
& cargo build --all-targets --all-features
Write-Host "== Test:"
& cargo test --all-targets --all-features
Write-Host "== Lint:"
& cargo clippy --all-targets --all-features
Write-Host "== Format:"
& cargo fmt --all -- --check

Write-Host "Completed!"

