# fincal — rule-based financial holiday calendar framework. `just gate` is the done-gate.

# show available recipes
default:
    @just --list

# the done-gate: formatting, lints and tests for both feature sets, docs (warnings denied), dependency audit
gate:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo clippy --all-targets --no-default-features -- -D warnings
    cargo test
    cargo test --no-default-features
    RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
    cargo audit

# days until each built-in calendar's valid range ends (exit status 2 if one ends within DAYS)
horizon DAYS="90":
    cargo run --quiet -- horizon --within {{DAYS}}
