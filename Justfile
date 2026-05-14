default:
    @just --list

fmt:
    cargo fmt --check -- `find . -name "*.rs"`

lint:
    cargo clippy --all-targets --all-features -- --deny warnings

build:
    cargo build --workspace --release

test:
    cargo test --workspace --release

ci: fmt lint build test

trollfarm:
    cargo build --release -p trollfarm
    python ./bot-programming/flatten.py trollfarm

snakebyte:
    cargo fmt --check -- `find . -name "*.rs"`
    # cargo clippy -p snakebyte -- --deny warnings
    cargo test -p snakebyte
    cargo build --release -p snakebyte
    python ./bot-programming/flatten.py snakebyte
    # ./bot-programming/snakebyte/sim.sh

brc-input:
    cargo run --release -p one-billion-rows --bin create-input 1_000_000

brc:
    cargo run --release -p one-billion-rows --bin solve

play-ttt:
    cargo run --release --bin play -- --game tic-tac-toe

play-c4:
    cargo run --release --bin play -- --game connect-four

play-uttt:
    cargo run --release --bin play -- --game ult-tic-tac-toe
