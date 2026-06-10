# Troll-Farm tooling scripts

Helper scripts for developing the bot, grouped into three categories. Run them
from the `trollfarm/` directory.

```text
scripts/
├── _common.py            # shared path anchors + helpers (imported by all)
├── local/                # 1. LOCAL HARNESS — benchmark & tune against the referee jar
│   ├── harness.py        #    aggregate benchmark + shared game runner (run_game)
│   ├── bench_arena.py    #    THE standard regression bench: 10 pinned arena seeds
│   │                     #    (5 lost + 5 won, eval/arena_bench_seeds.json), candidate
│   │                     #    vs baseline binary, margins + failed-action counts
│   ├── tune.py           #    coordinate-descent tuner (small curated param set)
│   ├── sweep.py          #    what-if probing on a few hand-picked seeds
│   ├── sweep_all.py      #    full 39-param coordinate descent vs several opponents
│   └── sensitivity.py    #    one-at-a-time response curves for the economy knobs
├── replays/              # 2. ARENA DATA — pull & inspect real games from CodinGame
│   ├── get_player_games.py   download all of a player's arena replays
│   └── parse_replay.py       fetch + summarise a single replay by URL / id
└── ide/                  # 3. LIVE IDE — drive the in-browser editor
    └── play_my_code.py   submit the current bot, play, report the result
```

## Environments

Set up a venv once: `python -m venv .venv && .venv/bin/pip install selenium request numpy matplotlib`.

## Shared — `_common.py`

Single source of truth so no script hard-codes directory depths. Derives every
path from its own location (`TROLLFARM_DIR`, `GAME_DIR`, `EVAL_DIR`,
`REPLAYS_DIR`, `WORKSPACE_DIR`) and provides the bits every tuner needs:
`make_seeds(n, base)` (the reproducible int64 game-seed draw), `set_tf_env(cfg)`
(apply a `{param: value}` config as `TF_*` overrides), and `build_tuning_bot()`
(`cargo build --features tuning` → deploy as `trollfarm-tuning`). Scripts reach
it via a one-line bootstrap that puts `scripts/` on `sys.path`.

---

## 1. `local/` — local match harness

Runs games against the referee jar in `codingame/` (`-l` to dump the replay,
never `-s`). Games are **deterministic per seed**; a base seed reproduces the
same set of maps across all of these tools.

- **`harness.py`** — the core benchmark _and_ the shared game runner the others
  import (`run_game`). Runs N games in parallel vs a reference bot, then prints
  win-rate / margin / economy stats and writes `eval/<label>/` + plots. (Named
  `harness`, not `eval`, so it doesn't clash with the `eval/` output dir or the
  `eval()` builtin.)

    ```bash
    python scripts/local/harness.py 1000 --seed 1 --jobs 16 --label baseline
    python scripts/local/harness.py --p2 ./trollfarm-ref-gold-70
    python scripts/local/harness.py --replay-dir replays/tonigineer     # analyse downloaded games
    ```

    Also via `just eval`, `just eval-quick`, `just eval-replay <dir>`.

- **`tune.py`** — coordinate descent over a small curated parameter set
  (optimises avg margin vs one opponent).

    ```bash
    python scripts/local/tune.py --games 100 --passes 2 --p2 ./trollfarm-ref-gold-X
    ```

- **`sweep.py`** — what-if probing on one or a few **hand-picked seeds** (1-D
  sweep or full grid); for chasing what flips a specific game.

    ```bash
    python scripts/local/sweep.py --seed 5342343 grove_value 1 2 3 5
    python scripts/local/sweep.py --seeds 1,2,3 --grid grove_value=2,4 harass_denial_weight=2,4
    ```

- **`sweep_all.py`** — full coordinate descent over **all 39 `TF_*` params**,
  scoring the mean (or `--objective min`) margin across **several opponents** to
  avoid overfitting to one playstyle. Writes the running best to
  `eval/sweep_all_best.json` and prints a ready-to-paste `params.rs` diff.

    ```bash
    python scripts/local/sweep_all.py --dry-run                 # plan + game/time estimate
    python scripts/local/sweep_all.py --games 100 --passes 3    # the long run (~hours)
    ```

- **`sensitivity.py`** — one-at-a-time (OAT) response curves for the economy
  knobs, each swept from the shipped defaults — to bracket useful ranges.

    ```bash
    python scripts/local/sensitivity.py --games 100 --only econ_chop_weight
    ```

> **`TF_*` sweeps only work if the opponent ignores `TF_*`** — the referee passes
> its env to _both_ bots. Use a non-tuning ref or an `env -i` sparring wrapper.
> See [`../BOTS.md`](../BOTS.md) for the opponent roster and this gotcha in full.

---

## 2. `replays/` — arena data

Pulls real games from CodinGame's public services into `replays/`.

- **`get_player_games.py`** — download all (or `--limit N`) of a player's arena
  replays to `replays/<player>/`; `--list-top N` lists the leaderboard.

    ```bash
    python scripts/replays/get_player_games.py tonigineer          # or by user id, e.g. 4083906
    ```

    Also `just get-games <player>`.

- **`parse_replay.py`** — fetch one replay by URL or id, print players / map /
  scores, and save the raw JSON to `replays/replay_<id>.json`.

    ```bash
    python scripts/replays/parse_replay.py 891040557
    ```

    Note: the anonymous API serves **public arena** replays only. Private _sandbox_
    ("Play my code") replays are fetched by `ide/play_my_code.py` from inside the
    logged-in browser instead.

---

## 3. `ide/` — live IDE automation

- **`play_my_code.py`** — one command: `just submit` (build + flatten +
  clipboard) → paste into the Monaco editor → click **Play my code** → read the
  new `/share-replay/<id>` and report scores/winner. Drives a persistent,
  logged-in Brave profile (`.cg-browser-profile`); attaches to the open window
  on the debug port across runs.

    ```bash
    python scripts/ide/play_my_code.py                # full pipeline, random map
    python scripts/ide/play_my_code.py --seed 12345   # replay a fixed map (tuning)
    python scripts/ide/play_my_code.py --no-submit    # play what's in the editor
    ```

    `--seed` pins the map via OPTIONS→Manual (honoured by _Play my code_, not
    "Replay in same conditions"). Sandbox replays need your user id (`--user-id`,
    default tonigineer) to authorise the result fetch.
