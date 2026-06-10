# ---------------------------------------------------------------------------
# Workspace-wide tasks
#
# These take an optional package name. With no argument they run across the
# whole workspace; with one they scope to a single crate:
#   just test            # whole workspace, incl. doc tests
#   just test trollfarm  # only the `trollfarm` crate, incl. doc tests
# ---------------------------------------------------------------------------

# List available recipes
default:
    @just --list

# Check formatting across the whole workspace
fmt:
    cargo fmt --all

# Verify formatting without modifying files
fmt-check:
    cargo fmt --all --check

# Clippy with warnings as errors; optionally scope to one package
lint pkg='':
    cargo clippy {{ if pkg == '' { '--workspace' } else { '-p ' + pkg } }} --all-targets --all-features -- --deny warnings

# Release build; optionally scope to one package
build pkg='':
    cargo build --release {{ if pkg == '' { '--workspace' } else { '-p ' + pkg } }}

# Run ALL tests including doc tests; optionally scope to one package.
# NOTE: no `--all-targets` here on purpose — it silently skips doc tests.
test pkg='':
    cargo test --release {{ if pkg == '' { '--workspace' } else { '-p ' + pkg } }}

# Full pipeline for the entire workspace
ci: fmt-check lint build test

# Flatten a bot crate into a single-file CG submission (tools/README.md)
flatten pkg:
    uv run --project tools ./tools/flatten.py {{ pkg }}

# ---------------------------------------------------------------------------
# One-click solutions, one per project.
#
# Each project also has its own dedicated Justfile with the full recipe set —
# run `just --list` inside the project dir (or `just -d <dir> -f <dir>/Justfile`).
# ---------------------------------------------------------------------------

# Shared library: run its test suite
common: (test 'common')

# Play tic-tac-toe in the terminal (human vs minimax)
tic-tac-toe:
    @just -d games/tic-tac-toe -f games/tic-tac-toe/Justfile play

# Play connect-four in the terminal (human vs minimax)
connect-four:
    @just -d games/connect-four -f games/connect-four/Justfile play

# Troll Farm bot: test, then build + flatten + compile-check for the CG editor
trollfarm: (test 'trollfarm')
    @just -d bots/trollfarm -f bots/trollfarm/Justfile submit

# Snakebyte bot: test, then build + flatten for the CG editor
snakebyte: (test 'snakebyte')
    @just -d bots/snakebyte -f bots/snakebyte/Justfile submit

# Ultimate Tic-Tac-Toe bot: test, then build + flatten for the CG editor
ultimate-tic-tac-toe: (test 'ultimate-tic-tac-toe')
    @just -d bots/ultimate-tic-tac-toe -f bots/ultimate-tic-tac-toe/Justfile submit

alias uttt := ultimate-tic-tac-toe

# One Billion Rows: generate the input if missing, then solve
one-billion-rows:
    @just -d puzzles/one-billion-rows -f puzzles/one-billion-rows/Justfile run

alias brc := one-billion-rows
