# One Billion Rows (1BRC)

An implementation of the “One Billion Rows” challenge.

> Original problem: [gunnarmorling/1brc](https://github.com/gunnarmorling/1brc)

## Quick Start

From the workspace root:

```bash
# 1) Create example data
cargo run --release -p one-billion-rows --bin create-input

# 2) Solve the problem
cargo run --release -p one-billion-rows --bin solve
