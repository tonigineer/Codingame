# Bot binaries (eval roster)

All binaries live in `codingame/` (the game dir, gitignored). `eval.py` / `tune.py`
/ `sweep.py` launch them with paths relative to that dir, e.g. `--p2 ./trollfarm-ref-gold-70`.

**Key gotcha — `TF_*` leaks to both players.** The referee passes *its* environment to
**both** bot children. So when you sweep a `TF_*` parameter (exported in your shell),
any opponent that reads `TF_*` will get the *same* value — the test is confounded.
**Only use opponents that ignore `TF_*` as `--p2` in a sweep.** Two ways to be safe:
a binary built *without* `--features tuning` (see "reads `TF_`?" column), or any binary
wrapped in **`env -i`** (empty environment), which is how the `trollfarm-spar-*`
sparring partners below pin their own config and block leakage.

## Roster

| binary | role | economy (opp fruit/game) | reads `TF_`? | our WR vs it | notes |
|---|---|---|---|---|---|
| `trollfarm` | our current release | n/a | no | — | what `just _deploy` ships; non-tuning, ignores `TF_*` |
| `trollfarm-tuning` | our current tuning build | n/a | **yes** | — | P1 for all sweeps; `--features tuning`, reads `TF_*` |
| `trollfarm-ref-gold-X` | fixed ref (default `--p2`) | **3.6 — none** | no | ~92% | pure wood, no grove. Best **no-economy** control |
| `trollfarm-ref-gold-3` | fixed ref | 8.2 — light | no | ~83% | light economy |
| `trollfarm-ref-gold-70` | fixed ref | **13.2 — moderate** | no | ~83% | best **clean with-economy** opponent for denial tests |
| `trollfarm-ref-v1` | fixed-ish ref | 13.6 — strong wood | **yes** ⚠ | ~17% (it beats us) | strongest opponent, but **confounded** as `--p2` in `TF_` sweeps |
| `trollfarm-prevdecay` | wrapper → `trollfarm-tuning` | ~33 — full grove | **yes** ⚠ | ~67% | our pre-"decay harass" config (`env`, not `env -i`, so **confounded** as `--p2`). Superseded by `trollfarm-spar-eco` |

### Sparring partners (clean — `env -i` wrappers, safe as `--p2` in sweeps)

| binary | role | opp fruit / wood | our WR | wraps |
|---|---|---|---|---|
| `trollfarm-spar-eco` | **strong economy** | **52 / 31** | ~85% | `trollfarm-tuning`, pinned to the pre-decay tuned-econ config (`TF_HARASS_TURN_DECAY=∞ TF_HARASS_OPP_CAP=∞ TF_ECON_PICK_EARLY_BOOST=0`) |
| `trollfarm-spar-v1` | **strongest all-round** | 25 / 61 | ~50% (even) | `trollfarm-ref-v1` — the real bot that beats us; `env -i` stops its `TF_` reading being perturbed |
| `trollfarm-spar-harass` | pure denier (stress test) | 44 / 4 | ~100% | `trollfarm-tuning`, relentless-denial config. *Weak* (harassing is net-negative) — little gradient, optional |

These regenerate trivially — they're shell wrappers in `codingame/`; they call
`trollfarm-tuning`/`trollfarm-ref-v1`, so rebuild those first (`just _deploy` builds
`trollfarm`; build `trollfarm-tuning` with `--features tuning`).

(Economy figures are mean opponent leftover-fruit/game from 12–80 game probes; WR figures are
indicative. "reads `TF_`?" verified by running each ref as P1 with extreme `TF_` values and
checking whether its margin moved.)

## Recommended eval set (anti-overfit)

Denial/harassment behaviour flips sign by opponent type, so always evaluate across the
economy spectrum, not one bot:

- **No economy (denial control):** `trollfarm-ref-gold-X` — nothing to deny; denial knobs should *not* hurt here.
- **With economy (denial target):** `trollfarm-ref-gold-70` — has trees/fruit to deny (clean, non-tuning).
- **Light economy (third point):** `trollfarm-ref-gold-3`.
- **Strongest / regression check:** `trollfarm-ref-v1` — only as P1 or with fixed env; it reads `TF_`.

## How to run

```bash
# Deploy current bot (non-tuning) as ./trollfarm
just _deploy trollfarm                       # or: cargo build --release -p trollfarm && cp ... codingame/

# Aggregate benchmark vs a chosen opponent
python scripts/eval.py 80 --seed 1000 --p2 ./trollfarm-ref-gold-70 --label myrun

# TF_ sweep: P1 must be the tuning build, P2 must ignore TF_ (non-tuning ref or env -i wrapper)
cargo build --release --features tuning -p trollfarm && cp target/release/trollfarm codingame/trollfarm-tuning
TF_HARASS_BOTTLENECK_WEIGHT=4 python scripts/eval.py 80 --p1 ./trollfarm-tuning --p2 ./trollfarm-ref-gold-70

# Full multi-opponent parameter sweep (~80 min, builds the tuning bot itself):
python scripts/sweep_all.py                 # 39 params, 2 passes, vs spar-eco + spar-v1
python scripts/sweep_all.py --dry-run       # plan + game-count estimate first
# running best config is written to eval/sweep_all_best.json throughout
```
