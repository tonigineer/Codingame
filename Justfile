default:
    @just --list

format:
    cargo fmt --check -- `find . -name "*.rs"`

clippy:
    cargo clippy --all-targets --all-features -- --deny warnings

build:
    cargo build --workspace

test:
    cargo test --workspace

ci: format clippy build test

brc-input:
    cargo run --release -p one-billion-rows --bin create-input 1_000_000

brc:
    cargo run --release -p one-billion-rows --bin solve

play-ttt:
    cargo run --release -p games --bin play -- --game tictactoe

play-c4:
    cargo run --release -p games --bin play -- --game connect-four
