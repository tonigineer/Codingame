default:
    @just --list

build:
    cargo build --workspace

test:
    cargo test --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

ci: build test clippy

brc-input:
    cargo run --release -p one-billion-rows --bin create-input 1_000_000

brc:
    cargo run --release -p one-billion-rows --bin solve

ttt:
    cargo run --release -p tic-tac-toe --bin play

ttt-test:
    cargo test -p tic-tac-toe
