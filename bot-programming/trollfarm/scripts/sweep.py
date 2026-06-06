#!/usr/bin/env python3
"""Targeted hyperparameter sweep on a few fixed seeds.

Runs the ``--features tuning`` bot on one or more chosen seeds under different
``TF_*`` parameter configs and prints the score margin for each, so you can hunt
the settings that flip a specific game (or a small seed set). Complements the
other two tools:

  * ``eval.py``  — aggregate benchmark over many random seeds.
  * ``tune.py``  — coordinate descent optimising avg margin over many games.
  * ``sweep.py`` — this: what-if probing on hand-picked seeds.

The referee passes its environment to the bot child, so we set ``os.environ``
``TF_*`` per config before launching each game (same mechanism as ``tune.py``).

Usage:
  # 1-D sweep of one param on one seed
  python scripts/sweep.py --seed 5342343 grove_value 1 2 3 5 8

  # hold extra params fixed via --base while sweeping
  python scripts/sweep.py --seed 5342343 --base early_max_turns=16 \
      grove_value 1 2 3

  # average over several seeds
  python scripts/sweep.py --seeds 5342343,1,2,3 grove_value 1 2 3

  # full grid (cartesian product) instead of a 1-D sweep
  python scripts/sweep.py --seed 5342343 \
      --grid grove_value=1,2,3 early_max_turns=16,20
"""
import argparse
import itertools
import os
import sys
from pathlib import Path

import eval as ev  # reuse run_game + scoring
from tune import build_tuning_bot  # reuse the tuning-bot build/deploy

SCRIPT_DIR = Path(__file__).resolve().parent


def coerce(s: str):
    """Parse a CLI token as int, then float, else leave it a string."""
    for cast in (int, float):
        try:
            return cast(s)
        except ValueError:
            pass
    return s


def parse_kv(items: list[str]) -> dict:
    """['a=1', 'b=2.5'] -> {'a': 1, 'b': 2.5} with coerced values."""
    out: dict = {}
    for it in items:
        k, _, v = it.partition("=")
        out[k] = coerce(v)
    return out


def set_env(config: dict) -> None:
    for k, v in config.items():
        os.environ[f"TF_{k.upper()}"] = str(v)


def expand_configs(args) -> list[dict]:
    """Build the list of param configs from the CLI (each merged onto --base)."""
    base = parse_kv(args.base)
    if args.grid:
        axes = {}
        for spec in args.grid:
            k, _, vs = spec.partition("=")
            axes[k] = [coerce(v) for v in vs.split(",")]
        keys = list(axes)
        configs = [dict(zip(keys, combo)) for combo in itertools.product(*axes.values())]
    elif args.sweep:
        param, *values = args.sweep
        if not values:
            sys.exit(f"sweep param '{param}' needs at least one value")
        configs = [{param: coerce(v)} for v in values]
    else:
        configs = [{}]  # just the base / shipped defaults
    return [{**base, **c} for c in configs]


def run_config(config: dict, seeds: list[int], p1: str, p2: str) -> list[tuple[int, int]]:
    set_env(config)
    return [(r.p1, r.p2) for r in (ev.run_game(i, s, p1, p2) for i, s in enumerate(seeds))]


def label_of(config: dict) -> str:
    return "  ".join(f"{k}={v}" for k, v in config.items()) or "(defaults)"


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("sweep", nargs="*", metavar="PARAM VALUE...",
                    help="parameter name followed by the values to sweep")
    ap.add_argument("--seed", type=int, help="single game seed")
    ap.add_argument("--seeds", help="comma-separated game seeds (results averaged)")
    ap.add_argument("--base", action="append", default=[], metavar="K=V",
                    help="param held fixed across all configs (repeatable)")
    ap.add_argument("--grid", nargs="+", default=None, metavar="K=v1,v2",
                    help="cartesian grid instead of a 1-D positional sweep")
    ap.add_argument("--p1", default="./trollfarm-tuning", help="bot under test (tuning build)")
    ap.add_argument("--p2", default=ev.DEFAULT_P2, help="reference opponent")
    ap.add_argument("--no-build", action="store_true",
                    help="skip building the tuning bot (reuse the deployed binary)")
    args = ap.parse_args()

    if args.seeds:
        seeds = [int(s) for s in args.seeds.split(",")]
    elif args.seed is not None:
        seeds = [args.seed]
    else:
        ap.error("provide --seed or --seeds")

    if not args.no_build:
        build_tuning_bot()

    configs = expand_configs(args)
    print(f"\nSeeds: {seeds}   P1={args.p1}  P2={args.p2}")
    print(f"Configs: {len(configs)}\n" + "-" * 66)

    best = None
    for cfg in configs:
        rows = run_config(cfg, seeds, args.p1, args.p2)
        margins = [a - b for a, b in rows]
        avg = sum(margins) / len(margins)
        wins = sum(1 for m in margins if m > 0)
        if len(seeds) == 1:
            a, b = rows[0]
            tag = "WIN " if a > b else "LOSS" if a < b else "DRAW"
            detail = f"{a:3d}-{b:3d} ({a - b:+4d}) [{tag}]"
        else:
            detail = f"avg {avg:+6.1f}   {wins}/{len(seeds)} wins"
        print(f"  {detail}   {label_of(cfg)}")
        if best is None or avg > best[0]:
            best = (avg, cfg)

    print("-" * 66)
    print(f"BEST  avg margin {best[0]:+.1f}   {label_of(best[1])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
