#!/usr/bin/env python3
"""Coordinate-descent tuner for the Troll-Farm bot hyperparameters.

Builds a `--features tuning` bot once, then probes parameter settings by
running the *same* games as the benchmark (it imports ``eval.run_game``) with
``TF_*`` environment overrides. Optimises **average margin** (P1 - P2), which
is a far denser signal than the ~8% win rate at 100 games.

Search method — coordinate descent:
  start from the shipped defaults; for each parameter in turn, sweep it across
  its small candidate range (all other params held at the current best), keep
  the value with the best avg margin, then move to the next parameter. One pass
  by default; `--passes 2` re-sweeps to catch interactions.

How overrides reach the bot:
  `eval.run_game` spawns the referee, which passes its environment to the bot
  child. We set ``os.environ["TF_*"]`` before each trial, so every game in that
  trial uses those params. Each game is a fresh process, so there's no stale
  caching between trials.

Usage:
  python3 tune.py                     # 100 games/trial, base seed 1, vs gold-X
  python3 tune.py --games 100 --jobs 16 --passes 1
  python3 tune.py --dry-run           # just print the plan + trial count
"""

import argparse
import os
import random
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import eval as ev  # reuse run_game + seeding + scoring

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_DIR = SCRIPT_DIR.parent.parent.parent    # workspace root (has Cargo.toml)
GAME_DIR = SCRIPT_DIR.parent / "codingame"
TUNING_BIN = "trollfarm-tuning"               # deployed name for the tuning build

# Defaults must mirror params.rs::DEFAULT for the params we tune.
DEFAULTS = {
    "early_max_turns": 20,
    "gather_best": 10,
    "min_carry_capacity": 2,
    "min_chop_power": 1,
    "lemon_bonus": 0.01,
    "denial_bonus": 8.0,
}

# Small ranges swept per parameter (coordinate descent visits them in order).
SEARCH_SPACE = {
    "early_max_turns": [12, 16, 20, 24],
    "gather_best": [6, 8, 10, 12],
    "min_carry_capacity": [2, 3],
    "min_chop_power": [1, 2],
    "lemon_bonus": [0.0, 0.01, 0.05],
    "denial_bonus": [1.0, 2.0, 3.0],
}

INT64_MIN, INT64_MAX = -(2 ** 63), 2 ** 63 - 1


def build_tuning_bot() -> str:
    """Build the `tuning`-featured bot and deploy it under a distinct name.

    Returns the P1 command (relative to GAME_DIR) eval.run_game should launch.
    """
    print("Building --features tuning bot ...")
    subprocess.run(
        ["cargo", "build", "--release", "-p", "trollfarm", "--features", "tuning"],
        cwd=REPO_DIR, check=True,
    )
    src = REPO_DIR / "target/release/trollfarm"
    dst = GAME_DIR / TUNING_BIN
    dst.write_bytes(src.read_bytes())
    dst.chmod(0o755)
    print(f"Deployed {dst}")
    return f"./{TUNING_BIN}"


def set_env(config: dict) -> None:
    """Apply a parameter config as TF_* environment overrides."""
    for name, value in config.items():
        os.environ[f"TF_{name.upper()}"] = str(value)


def run_trial(config: dict, seeds: list[int], p1: str, p2: str, jobs: int) -> dict:
    """Run one parameter config over all seeds; return aggregate metrics."""
    set_env(config)
    margins, wins = [], 0
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = [
            pool.submit(ev.run_game, i, s, p1, p2) for i, s in enumerate(seeds)
        ]
        for fut in as_completed(futures):
            try:
                r = fut.result()
            except Exception:  # noqa: BLE001 - tolerate the occasional flaky game
                continue
            margins.append(r.margin)
            wins += r.outcome == "WIN"
    n = len(margins) or 1
    return {
        "avg_margin": sum(margins) / n,
        "win_rate": wins / n,
        "games": len(margins),
    }


def fmt_config(config: dict) -> str:
    return "  ".join(f"{k}={v}" for k, v in config.items())


def coordinate_descent(seeds, p1, p2, jobs, passes) -> dict:
    best = dict(DEFAULTS)
    cache: dict[tuple, dict] = {}

    def evaluate(config) -> dict:
        key = tuple(sorted(config.items()))
        if key not in cache:
            t0 = time.time()
            cache[key] = run_trial(config, seeds, p1, p2, jobs)
            cache[key]["secs"] = time.time() - t0
        return cache[key]

    base = evaluate(best)
    print(f"\nbaseline (defaults): margin {base['avg_margin']:+.2f}  "
          f"win {base['win_rate'] * 100:.1f}%  ({base['secs']:.1f}s)\n")

    for p in range(1, passes + 1):
        print(f"═══ pass {p}/{passes} " + "═" * 40)
        for name, values in SEARCH_SPACE.items():
            print(f"\n▶ sweeping {name}  (current best {best[name]})")
            local_best, local_metric = best[name], evaluate(best)["avg_margin"]
            for v in values:
                if v == best[name]:
                    mark, m = "  (current)", local_metric
                else:
                    trial = {**best, name: v}
                    res = evaluate(trial)
                    m = res["avg_margin"]
                    mark = f"  win {res['win_rate'] * 100:.1f}%  ({res['secs']:.1f}s)"
                star = " *" if m > local_metric else ""
                print(f"    {name}={v:<6}  margin {m:+.2f}{mark}{star}")
                if m > local_metric:
                    local_metric, local_best = m, v
            if local_best != best[name]:
                print(f"  → {name}: {best[name]} → {local_best}  "
                      f"(margin {evaluate(best)['avg_margin']:+.2f} → {local_metric:+.2f})")
                best[name] = local_best
            else:
                print(f"  → {name}: kept {best[name]}")

    return best


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--games", type=int, default=100, help="games per trial (default 100)")
    ap.add_argument("--seed", type=int, default=1, help="base seed for the game-seed RNG")
    ap.add_argument("--jobs", type=int, default=16, help="parallel games (default 16)")
    ap.add_argument("--passes", type=int, default=1, help="coordinate-descent passes")
    ap.add_argument("--p2", default=ev.DEFAULT_P2, help="reference opponent")
    ap.add_argument("--dry-run", action="store_true", help="print plan + trial count, don't run")
    args = ap.parse_args()

    rng = random.Random(args.seed)
    seeds = [rng.randint(INT64_MIN, INT64_MAX) for _ in range(args.games)]

    # Worst-case distinct trials = baseline + (len-1) per swept param, per pass.
    trials = 1 + args.passes * sum(len(v) - 1 for v in SEARCH_SPACE.values())
    print(f"Coordinate descent | {args.games} games/trial | base seed {args.seed} "
          f"| {args.jobs} parallel")
    print(f"Params: {list(SEARCH_SPACE)}")
    print(f"Up to {trials} distinct trials (~{trials * args.games} games)")
    if args.dry_run:
        return 0

    p1 = build_tuning_bot()
    t0 = time.time()
    best = coordinate_descent(seeds, p1, args.p2, args.jobs, args.passes)
    print("\n" + "═" * 56)
    print(f"BEST CONFIG (after {time.time() - t0:.0f}s):")
    print("  " + fmt_config(best))
    changed = {k: v for k, v in best.items() if v != DEFAULTS[k]}
    print("Changed from defaults:", fmt_config(changed) if changed else "(none)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
