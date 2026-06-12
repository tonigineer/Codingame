# Tools

Dev tooling for the CodinGame bots, in two categories: `local/` (offline games
against a downloaded referee jar) and `ide/` (everything that talks to
codingame.com). All of it is generic: every CLI requires `--game <crate>`
(the bot crate in `bots/`; `flatten.py` takes it as a positional). Each
game's puzzle URL slug (`codingame.com/multiplayer/bot-programming/<slug>`)
is defined in code, in `ide/_browser.PUZZLE_SLUGS` (e.g. `trollfarm` →
`spring-challenge-2026-troll-farm`) — crate names do not encode the slug;
adding a game means adding its entry there.

```text
tools/
├── flatten.py            # inline a bot crate into one submittable file
├── extract_minimax.py    # single-file minimax harness from common/ — just define the game
├── local/                # 1. LOCAL — offline games vs the referee jar
│   └── validate.py       #    N reproducible games vs a roster of local opponents
└── ide/                  # 2. CODINGAME — the IDE & replay toolbox
    ├── _browser.py       #    shared library: path anchors + browser primitives (no CLI)
    ├── play.py           #    play one game in the IDE (just ide)
    ├── bench.py          #    the standard bench: pinned arena seeds vs real opponents
    ├── fetch_replay.py   #    one replay → summary + full JSON (--ide for sandbox games)
    └── fetch_player_games.py  all arena replays of a player (just get-games)
```

The browser-based tools (`play`, `bench`, `fetch_replay --ide`) drive a
persistent logged-in Brave profile at `tools/.cg-browser-profile` (gitignored;
log in once in the window that opens). Per-bot outputs land in the bot's crate
dir: `bots/<game>/eval/` and `bots/<game>/replays/`.

## Environment

Dependencies are managed with [uv](https://docs.astral.sh/uv/):
`pyproject.toml` is the manifest, `uv.lock` pins exact versions (committed),
and the env lives in `tools/.venv` (gitignored). No activation needed — run
any tool through uv and the env is created/synced on demand:

```bash
uv run --project tools tools/local/validate.py --game trollfarm 100
uv run --project tools ruff check tools/        # ruff is a locked dev dep
```

The `just` recipes already do this. After editing `pyproject.toml`, the next
`uv run` re-syncs automatically (or run `uv sync` inside `tools/`).

## `flatten.py` — single-file CG submission

Inlines a bot crate's `mod foo;` tree into `src/main.rs.flattened` (strips
comments, stamps the build time, copies to the clipboard via `wl-copy`).
Crates are resolved as `bots/<game>` relative to the repo root.

```bash
python tools/flatten.py trollfarm        # or: just flatten trollfarm
```

## `extract_minimax.py` — minimax starter for a new puzzle

Extracts the search harness from `common/src` (`GameError`, the
`Player`/`Game`/`Strategy` traits, the negamax `Minimax` with ahash swapped
for std `HashMap`) into one dependency-free Rust file, followed by a skeleton
game: fill in the `todo!()`s (see `common/README.md` for the method
contracts), parse the referee input in `main()`, and the file is a complete
CodinGame submission. This is the route for bots that would otherwise depend
on `common` — which flatten.py can't inline yet. Stdlib-only, no uv needed.

```bash
python tools/extract_minimax.py    # rustc-typecheck, then copy to clipboard
```

## 1. `local/` — offline games vs the referee jar

- **`validate.py`** — N reproducible games (parallel) against each opponent
  in a roster of local binaries, reporting W-L-D / win% / avg+median margin
  per opponent. Builds + deploys the bot first (`--no-build` to skip).
  Everything lives in `bots/<game>/codingame/`: the referee jar
  (auto-discovered) and the opponent binaries; the default roster per bot is
  defined in the script (`DEFAULT_OPPONENTS`). Games are **deterministic per
  seed** and
  `--seed` is a base that draws the per-game seeds, so the same base replays
  the same maps for every opponent — and a base the change was never tuned
  on gives you held-out validation.

    ```bash
    uv run --project tools tools/local/validate.py --game trollfarm    # default roster
    uv run --project tools tools/local/validate.py --game trollfarm 200 --seed 7 --jobs 16
    uv run --project tools tools/local/validate.py --game trollfarm --opponents ./trollfarm-ref-gold-X,./trollfarm-spar-v1
    ```

    Also `just eval [games] [seed]` / `just eval-quick` from the bot dir.
    (The old TF_* sweep tooling — `harness.py`, `sweep.py`, `sweep_all.py` —
    was removed 2026-06-10.)

## 2. `ide/` — the CodinGame toolbox

Generic across games: each CLI requires `--game <crate>`; the puzzle slug
comes from `_browser.PUZZLE_SLUGS`. The browser tools
attach to a persistent logged-in Brave profile (`tools/.cg-browser-profile`)
over the Chrome debug port, reusing the open window across runs.

- **`_browser.py`** — the shared **library** (no CLI): path anchors, the
  puzzle-slug registry, and the selenium primitives — driver attach/launch,
  login wait, clipboard-free editor injection with tail verification (a bare
  OS Ctrl+V paste can silently leave the editor empty — score −2, "main not
  found"), seed pinning, play, result fetch.

- **`play.py`** — play ONE game: injects `bots/<game>/src/main.rs.flattened`
  into the editor, verifies it, clicks PLAY MY CODE, reports the result.
  `--seed` pins the map (OPTIONS → Manual); `--submit` runs `just submit` in
  the bot dir first. This is what `just ide [seed]` runs.

- **`bench.py`** — **THE standard bench**: replays the pinned arena seeds
  (`--seeds-file`, default `bots/<game>/eval/arena_bench_seeds.json`: 5 lost +
  5 won) against the REAL arena opponents — sets the opponent in the PLAYERS
  tab, pins the seed, plays, records the result. `--skip N` resumes a broken
  run.

- **`fetch_replay.py`** — one replay → summary (players, scores, referee
  seed, frames) + full JSON to `bots/<game>/replays/replay_<id>.json`. Two
  mutually exclusive modes: a **game id/URL** fetches that finished game
  (anonymous public API; your private sandbox games transparently fall back
  to the logged-in browser), while **`--ide`** operates the IDE — fetches
  the game in the viewer; if there is none it first plays one properly via
  the play.py flow (inject current bot, verify editor, PLAY MY CODE).

- **`fetch_player_games.py`** — download all (or `--limit N`) of a player's
  arena replays to `bots/<game>/replays/<player>/`; `--list-top N` lists the
  leaderboard. Also `just get-games <player>`.

    ```bash
    uv run --project tools tools/ide/fetch_player_games.py --game trollfarm tonigineer
    ```
