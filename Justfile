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

play-ttt:
    cargo run --release -p games --bin play -- --game tictactoe

play-c4:
    cargo run --release -p games --bin play -- --game connect-four
