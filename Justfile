
# ---------------------------------------------------------------------------
# Project-wide tasks
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

# Check formatting across the whole workspace
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

# Full pipeline for the entire project
ci: fmt-check lint build test

# ---------------------------------------------------------------------------
# CodinGame bots
#
# `flatten` inlines a modular crate into one submittable file.
# `submit`  builds + flattens any bot crate: `just submit trollfarm`.
# `play`    play against ref for given seed (or random)
# `eval`    evaluate 100 games against ref
# ---------------------------------------------------------------------------

# Flatten a bot crate into a single-file CG submission
flatten pkg:
    python ./bot-programming/flatten.py {{ pkg }}

# Build a bot and flatten it for submission
submit pkg: (build pkg) (flatten pkg)

play pkg='trollfarm' seed='1': (build pkg)
    #!/usr/bin/env bash
    set -euo pipefail
    game_dir="./bot-programming/trollfarm/assets/Troll-Farm"
    cp "./target/release/{{ pkg }}" "$game_dir/"
    cd "$game_dir"

    nohup java -jar ./troll-farm-1.0-SNAPSHOT.jar \
        -p1 "./{{ pkg }}" -p2 "./{{ pkg }}-ref" -s -seed "{{ seed }}" > server.log 2>&1 &
    disown
    # echo "started; tail -f $runner/server.log to watch"

# Named bot pipelines (test first, then build + flatten)
trollfarm: (test "trollfarm") (submit "trollfarm")

snakebyte: (test "snakebyte") (submit "snakebyte")
    # ./bot-programming/snakebyte/sim.sh

# # ---------------------------------------------------------------------------
# # Local game runner
# # ---------------------------------------------------------------------------

# # Play a game locally: `just play tic-tac-toe`
# play game:
#     cargo run --release --bin play -- --game {{ game }}

# # Shortcuts
# play-ttt: (play "tic-tac-toe")
# play-c4: (play "connect-four")
# play-uttt: (play "ult-tic-tac-toe")

# ---------------------------------------------------------------------------
# One Billion Row Challenge
# ---------------------------------------------------------------------------

# Generate an input file with N rows
brc-input rows='1_000_000':
    cargo run --release -p one-billion-rows --bin create-input {{ rows }}

# Run the solver
brc:
    cargo run --release -p one-billion-rows --bin solve
