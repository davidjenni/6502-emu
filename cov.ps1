& cargo clean
# install nightly with:
# > rustup toolchain install nightly
& rustup default nightly

$env:CARGO_INCREMENTAL="0"
# local coverage report uses profile; the -Cinstrument-coverage currently fails on Win11 with access violations
$env:RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off"

& cargo build
& cargo test --no-fail-fast --all-targets

$COVERAGE_REPORT_DIR="target/debug"

& grcov . --output-type html --output-path "$COVERAGE_REPORT_DIR" --branch --ignore build.rs --ignore '/*' --ignore '[a-zA-Z]:/*' --ignore '**/tests/' --excl-start '#(\[cfg\(test\)\]|\[test\])' --excl-br-line '^\s*((debug_)?assert(_eq|_ne)?!|#\[derive\()'

Remove-Item Env:\RUSTFLAGS
Remove-Item Env:\CARGO_INCREMENTAL
& rustup default stable

Write-Host "report: $COVERAGE_REPORT_DIR/html/index.html"
Start-Process "$COVERAGE_REPORT_DIR/html/index.html"
