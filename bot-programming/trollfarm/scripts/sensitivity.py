#!/usr/bin/env python3
"""One-at-a-time (OAT) sensitivity analysis for the economy tuning knobs.

For each parameter, sweep it across a candidate range *from the shipped
defaults* (every other param held at default) over the same reproducible seed
set as the benchmark (base seed -> eval's int64 seed draw), and print a compact
table of avg margin (P1 - P2) and win% per value. The point is to see each
knob's response curve and bracket a useful range before fine optimisation.

Builds the `--features tuning` bot once (reusing tune.build_tuning_bot), then
sets `TF_*` env vars per config; the referee passes its environment to the bot
child. Avg margin is the dense signal; win% is the actual goal (beat gold-X).

Usage:
  python3 scripts/sensitivity.py                  # 60 games/value, all knobs
  python3 scripts/sensitivity.py --games 100 --jobs 16
  python3 scripts/sensitivity.py --only econ_chop_weight
"""
import argparse
import os
import random
import sys
import time
from concurrent.futures import ThreadPoolExecutor

import eval as ev
from tune import build_tuning_bot

INT64_MIN, INT64_MAX = -(2**63), 2**63 - 1

# Parameter -> candidate values to sweep (defaults marked in comments).
SWEEPS = {
    "econ_pick_weight":    [0.0, 0.5, 1.0, 2.0, 4.0, 8.0],      # default 2.0
    "econ_harvest_weight": [0.0, 0.5, 1.0, 2.0, 4.0, 8.0],      # default 2.0
    "econ_chop_weight":    [1.0, 2.0, 5.0, 10.0, 20.0, 40.0],   # default 5.0
    "grove_value":         [0.5, 1.0, 2.0, 4.0, 8.0, 16.0],     # default 2.0
}
MANAGED = [f"TF_{k.upper()}" for k in SWEEPS]


def clear_env() -> None:
    for k in MANAGED:
        os.environ.pop(k, None)


def safe_game(index, seed, p1, p2):
    """run_game with one sequential retry; None if it stays flaky (skip it)."""
    for _ in range(2):
        try:
            return ev.run_game(index, seed, p1, p2)
        except Exception:
            continue
    return None


def run_batch(seeds, p1, p2, jobs) -> tuple[float, float, float]:
    """Run all seeds for the current env config; return (avg_margin, win%, draw%)."""
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        rows = list(ex.map(lambda t: safe_game(t[0], t[1], p1, p2), enumerate(seeds)))
    margins = [r.p1 - r.p2 for r in rows if r is not None]
    n = len(margins)
    avg = sum(margins) / n
    wins = 100.0 * sum(1 for m in margins if m > 0) / n
    draws = 100.0 * sum(1 for m in margins if m == 0) / n
    return avg, wins, draws


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--games", type=int, default=60, help="games per value")
    ap.add_argument("--seed", type=int, default=1, help="base seed (benchmark set)")
    ap.add_argument("--jobs", type=int, default=16)
    ap.add_argument("--p1", default="./trollfarm-tuning")
    ap.add_argument("--p2", default=ev.DEFAULT_P2)
    ap.add_argument("--only", action="append", default=None,
                    help="restrict to these params (repeatable)")
    ap.add_argument("--no-build", action="store_true")
    args = ap.parse_args()

    rng = random.Random(args.seed)
    seeds = [rng.randint(INT64_MIN, INT64_MAX) for _ in range(args.games)]

    if not args.no_build:
        build_tuning_bot()

    params = args.only if args.only else list(SWEEPS)

    # Baseline: all defaults.
    clear_env()
    t0 = time.time()
    avg, win, draw = run_batch(seeds, args.p1, args.p2, args.jobs)
    print(f"\n{args.games} games/value  base seed {args.seed}  vs {args.p2}")
    print("=" * 60)
    print(f"BASELINE (all defaults):  avg {avg:+6.1f}   win {win:4.1f}%   draw {draw:4.1f}%")

    for param in params:
        print("-" * 60)
        print(f"{param}   (default in DEFAULT; others held at default)")
        for v in SWEEPS[param]:
            clear_env()
            os.environ[f"TF_{param.upper()}"] = str(v)
            avg, win, draw = run_batch(seeds, args.p1, args.p2, args.jobs)
            print(f"  {param}={v:<6}  avg {avg:+6.1f}   win {win:4.1f}%   draw {draw:4.1f}%")

    print("=" * 60)
    print(f"done in {time.time() - t0:.0f}s")
    return 0


if __name__ == "__main__":
    sys.exit(main())
