# Codingame-AI

A general-purpose repo for solving coding puzzles and building competitive bots
in `Rust`, organized as one cargo workspace.

## Workspace

| Crate | What it is |
|-------|------------|
| [`common/`](common/README.md) | Shared library: `Game`/`Player` traits, `Competition` runner, search strategies (minimax + baselines) — see its README for how to add a game |
| [`games/tic-tac-toe/`](games/tic-tac-toe/) | Tic-tac-toe on the `common` traits — playable in the terminal vs minimax |
| [`games/connect-four/`](games/connect-four/) | Connect Four (bitboards) on the `common` traits — playable in the terminal vs minimax |
| [`games/ultimate-ttt/`](games/ultimate-ttt/) | Ultimate Tic-Tac-Toe on the `common` traits — playable in the terminal vs minimax |
| [`bots/trollfarm/`](bots/trollfarm/) | [Troll Farm](https://www.codingame.com/multiplayer/bot-programming/spring-challenge-2026-troll-farm) arena bot — **Legend** league |
| [`bots/snakebyte/`](bots/snakebyte/) | Snakebyte arena bot |
| [`bots/ultimate-tic-tac-toe/`](bots/ultimate-tic-tac-toe/) | [Ultimate Tic-Tac-Toe](https://www.codingame.com/multiplayer/bot-programming/tic-tac-toe) arena bot — Bronze league |
| [`puzzles/one-billion-rows/`](puzzles/one-billion-rows/README.md) | The 1BRC challenge |

Supporting directories (not crates):

- [`tools/`](tools/README.md) — dev tooling: `flatten.py` (single-file CG
  submissions) plus the trollfarm benchmark / arena-replay / browser-IDE
  automation scripts.
- [`bots/README.md`](bots/README.md) — bot overview and submission notes.
- [`docs/`](docs/) — guides: [MCTS for tic-tac-toe](docs/mcts-tic-tac-toe.md)
  (also as [PDF](docs/mcts-tic-tac-toe.pdf)).

## Usage

Each project is driven by [`just`](https://github.com/casey/just). The root
`Justfile` has a the workspace-wide tasks; every project dir also has its
own dedicated `Justfile` with the full recipe set (`just --list` inside the dir).

```sh
just trollfarm           # test + build + flatten + compile-check the CG submission
just snakebyte           # test + build + flatten
just uttt                # test + build + flatten (ultimate-tic-tac-toe)
just tic-tac-toe         # play in the terminal vs minimax
just connect-four        # play in the terminal vs minimax
just ultimate-ttt        # play in the terminal vs minimax
just brc                 # generate input (if missing) + solve one-billion-rows
```

Workspace-wide (optionally scoped: `just test trollfarm`):

```sh
just ci                  # fmt-check + lint + build + test, whole workspace
just fmt | lint | build | test
just flatten <bot>       # single-file CG submission for any bot crate
```
