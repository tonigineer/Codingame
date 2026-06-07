#!/usr/bin/env python3
"""Full-parameter coordinate-descent sweep across MULTIPLE opponents.

Sweeps *every* tunable `TF_*` parameter of OUR bot (P1 only) to maximise an
objective combined over several fixed opponents — so we tune for robustness
across playstyles instead of overfitting to one bot.

Why multiple opponents: denial/harassment behaviour flips sign by opponent type
(see BOTS.md). Optimising the *mean* (or *min*) margin over a strong-economy and
a strong all-round opponent keeps any single style from dominating the result.

Clean opponents only: the referee passes its env to BOTH bots, so a `TF_*` sweep
is valid only if each opponent ignores `TF_*`. The `trollfarm-spar-*` wrappers
use `env -i` to guarantee that (see BOTS.md). Do NOT point `--opponents` at a
raw `--features tuning` binary.

Search method — coordinate descent:
  start from the shipped defaults; for each parameter in turn, try its candidate
  values (others held at the current best), keep the best by objective, move on.
  `--passes 2+` re-sweeps to catch interactions. The current best config is
  written to `--out` after every parameter so a long run is reviewable / not lost.

Usage:
  # default: ~1.5-2h, 60 games/opponent/trial, 2 passes, eco + v1
  python3 scripts/sweep_all.py

  # quick smoke (few params via --only, tiny game count)
  python3 scripts/sweep_all.py --games 8 --passes 1 --only grove_value,harass_denial_weight

  # custom opponents / objective
  python3 scripts/sweep_all.py --opponents ./trollfarm-spar-eco,./trollfarm-spar-v1,./trollfarm-spar-harass --objective min

  python3 scripts/sweep_all.py --dry-run     # print plan + game-count estimate
"""

import argparse
import json
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import _common as C
import harness as ev  # reuse run_game + scoring

# Must mirror src/bot/params.rs::DEFAULT.
DEFAULTS = {
    "early_max_turns": 20,
    "gather_best": 10,
    "gather_good": 5,
    "gather_least": 2,
    "cost_pick_drop": 2,
    "min_movement_speed": 2,
    "min_carry_capacity": 2,
    "min_chop_power": 1,
    "relax_movement_speed": 1,
    "relax_carry_capacity": 1,
    "relax_chop_power": 1,
    "stuck_horizon": 20,
    "lemon_bonus": 0.01,
    "banana_bonus": 0.005,
    "denial_bonus": 8.0,
    "opp_denial_radius": 6,
    "return_weight_economy": 1.0,
    "return_weight_harasser": 0.3,
    "return_full_boost": 4.0,
    "grove_value": 4.0,
    "econ_pick_weight": 2.0,
    "econ_harvest_weight": 8.0,
    "econ_chop_weight": 5.0,
    "econ_pick_early_boost": 4.0,
    "econ_pick_boost_turns": 120.0,
    "plant_decay_turns": 10,
    "harass_seed_plant_score": 1000.0,
    "harass_seed_fetch_score": 0.0,
    "harass_camp_score": 0.0,
    "harass_denial_weight": 2.0,
    "harass_chop_scale_lemon": 1.75,
    "harass_chop_scale_plum": 1.50,
    "harass_chop_scale_apple": 1.25,
    "harass_chop_scale_banana": 1.00,
    "harass_return_weight": 1.0,
    "harass_turn_decay": 120.0,
    "harass_opp_cap": 150.0,
    "harass_bottleneck_weight": 0.0,
    "harass_train_min_stat": 2,
}

INF = 1_000_000_000.0  # the bot's "never decay / never surrender" sentinel (float: feeds f32 fields)

# Candidate values per parameter (coordinate descent visits them in order).
# Each list includes the default so a param never regresses below baseline.
SEARCH_SPACE = {
    "early_max_turns": [12, 16, 20, 24],
    "gather_best": [6, 8, 10, 12, 14],
    "gather_good": [3, 5, 7],
    "gather_least": [1, 2, 3],
    "cost_pick_drop": [1, 2, 3],
    "min_movement_speed": [1, 2, 3],
    "min_carry_capacity": [2, 3],
    "min_chop_power": [1, 2],
    "relax_movement_speed": [1, 2],
    "relax_carry_capacity": [1, 2],
    "relax_chop_power": [1, 2],
    "stuck_horizon": [10, 20, 30],
    "lemon_bonus": [0.0, 0.01, 0.05],
    "banana_bonus": [0.0, 0.005, 0.05],
    "denial_bonus": [4.0, 8.0, 12.0],
    "opp_denial_radius": [4, 6, 8],
    "return_weight_economy": [0.5, 1.0, 2.0],
    "return_weight_harasser": [0.1, 0.3, 0.6],
    "return_full_boost": [2.0, 4.0, 8.0],
    "grove_value": [2.0, 4.0, 6.0, 8.0],
    "econ_pick_weight": [1.0, 2.0, 4.0],
    "econ_harvest_weight": [4.0, 8.0, 12.0],
    "econ_chop_weight": [3.0, 5.0, 8.0],
    "econ_pick_early_boost": [0.0, 2.0, 4.0, 8.0],
    "econ_pick_boost_turns": [60.0, 120.0, 180.0],
    "plant_decay_turns": [5, 10, 20],
    "harass_seed_plant_score": [500.0, 1000.0],
    "harass_seed_fetch_score": [0.0, 1.0],
    "harass_camp_score": [0.0, 5.0, 20.0],
    "harass_denial_weight": [1.0, 2.0, 4.0, 6.0],
    "harass_chop_scale_lemon": [1.25, 1.5, 1.75, 2.0],
    "harass_chop_scale_plum": [1.0, 1.25, 1.5],
    "harass_chop_scale_apple": [1.0, 1.25, 1.5],
    "harass_chop_scale_banana": [0.5, 1.0, 1.5],
    "harass_return_weight": [0.3, 1.0, 2.0],
    "harass_turn_decay": [60.0, 120.0, 300.0, INF],
    "harass_opp_cap": [100.0, 150.0, INF],
    "harass_bottleneck_weight": [0.0, 1.0, 2.0, 4.0],
    "harass_train_min_stat": [2, 3],
}

DEFAULT_OPPONENTS = ["./trollfarm-spar-eco", "./trollfarm-spar-v1"]


def run_vs(seeds, p1, p2, jobs) -> dict:
    """Average margin / win-rate for the current env config vs one opponent."""
    margins, wins = [], 0
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        futs = [pool.submit(ev.run_game, i, s, p1, p2) for i, s in enumerate(seeds)]
        for fut in as_completed(futs):
            try:
                r = fut.result()
            except Exception:  # noqa: BLE001 - tolerate a flaky game
                continue
            margins.append(r.margin)
            wins += r.outcome == "WIN"
    n = len(margins) or 1
    return {"margin": sum(margins) / n, "win": wins / n, "games": len(margins)}


def run_trial(config, seeds, p1, opponents, jobs, objective) -> dict:
    """Run a config vs every opponent; return per-opponent + combined score."""
    C.set_tf_env(config)
    per = {p2: run_vs(seeds, p1, p2, jobs) for p2 in opponents}
    margins = [per[p2]["margin"] for p2 in opponents]
    score = (sum(margins) / len(margins)) if objective == "mean" else min(margins)
    return {"score": score, "per": per}


def fmt_per(per) -> str:
    return "  ".join(
        f"{Path(p2).name.replace('trollfarm-spar-', '')}={d['margin']:+.0f}/{d['win']*100:.0f}%"
        for p2, d in per.items()
    )


def coordinate_descent(seeds, p1, opponents, jobs, passes, objective, params, out_path) -> dict:
    best = dict(DEFAULTS)
    cache: dict[tuple, dict] = {}

    def evaluate(config) -> dict:
        key = tuple(sorted(config.items()))
        if key not in cache:
            t0 = time.time()
            cache[key] = run_trial(config, seeds, p1, opponents, jobs, objective)
            cache[key]["secs"] = time.time() - t0
        return cache[key]

    base = evaluate(best)
    print(f"\nbaseline: score {base['score']:+.2f}  [{fmt_per(base['per'])}]  ({base['secs']:.0f}s)\n")
    save(out_path, best, base, "baseline")

    for p in range(1, passes + 1):
        print(f"═══ pass {p}/{passes} " + "═" * 44)
        for name in params:
            values = SEARCH_SPACE[name]
            cur = evaluate(best)["score"]
            local_best, local_score = best[name], cur
            print(f"\n▶ {name} (current {best[name]}, score {cur:+.1f})")
            for v in values:
                if v == best[name]:
                    print(f"    {name}={v:<12} score {cur:+.2f}   (current)")
                    continue
                res = evaluate({**best, name: v})
                star = " *" if res["score"] > local_score else ""
                print(
                    f"    {name}={v:<12} score {res['score']:+.2f}  "
                    f"[{fmt_per(res['per'])}] ({res['secs']:.0f}s){star}"
                )
                if res["score"] > local_score:
                    local_score, local_best = res["score"], v
            if local_best != best[name]:
                print(f"  → {name}: {best[name]} → {local_best}  (score {cur:+.2f} → {local_score:+.2f})")
                best[name] = local_best
            else:
                print(f"  → {name}: kept {best[name]}")
            save(out_path, best, evaluate(best), f"pass{p}:{name}")

    return best


def save(out_path, best, metrics, stage) -> None:
    changed = {k: v for k, v in best.items() if v != DEFAULTS[k]}
    out_path.write_text(json.dumps({
        "stage": stage,
        "score": metrics["score"],
        "per_opponent": {Path(k).name: v for k, v in metrics["per"].items()},
        "best": best,
        "changed_from_default": changed,
    }, indent=2))


def rust_lines(best) -> str:
    out = []
    for k, v in best.items():
        if v == DEFAULTS[k]:
            continue
        rv = f"{v:_}" if isinstance(v, int) and abs(v) >= 1000 else (f"{v}" if isinstance(v, int) else f"{v}")
        out.append(f"    {k}: {rv},")
    return "\n".join(out) if out else "    (no change from defaults)"


def main() -> int:
    # Line-buffer stdout so progress shows live even when redirected to a file
    # (block buffering otherwise hides output until ~8 KB accumulates).
    sys.stdout.reconfigure(line_buffering=True)

    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--games", type=int, default=60, help="games per opponent per trial (default 60)")
    ap.add_argument("--seed", type=int, default=1, help="base seed for the game-seed RNG")
    ap.add_argument("--jobs", type=int, default=16, help="parallel games (default 16)")
    ap.add_argument("--passes", type=int, default=2, help="coordinate-descent passes (default 2)")
    ap.add_argument("--opponents", default=",".join(DEFAULT_OPPONENTS),
                    help="comma list of NON-TUNING opponent binaries (env -i wrappers)")
    ap.add_argument("--objective", choices=["mean", "min"], default="mean",
                    help="combine per-opponent margins: mean (default) or min (maximin/robust)")
    ap.add_argument("--only", default="", help="comma list: sweep only these params")
    ap.add_argument("--out", default="sweep_all_best.json",
                    help="running-best output (relative paths land in scripts/../eval/)")
    ap.add_argument("--dry-run", action="store_true", help="print plan + game estimate, don't run")
    args = ap.parse_args()

    opponents = [o.strip() for o in args.opponents.split(",") if o.strip()]
    params = [p.strip() for p in args.only.split(",") if p.strip()] or list(SEARCH_SPACE)
    bad = [p for p in params if p not in SEARCH_SPACE]
    if bad:
        print(f"unknown params: {bad}", file=sys.stderr)
        return 2

    seeds = C.make_seeds(args.games, args.seed)
    trials = 1 + args.passes * sum(len(SEARCH_SPACE[p]) - 1 for p in params)
    games_total = trials * args.games * len(opponents)
    print(f"Full sweep | {len(params)} params | {args.games} games × {len(opponents)} opp / trial "
          f"| {args.passes} passes | objective={args.objective}")
    print(f"Opponents: {opponents}")
    print(f"≤ {trials} distinct trials  ≈ {games_total} games "
          f"(at ~4 games/s ≈ {games_total/4/60:.0f} min)")
    if args.dry_run:
        return 0

    out_path = Path(args.out)
    if not out_path.is_absolute():
        out_path = C.EVAL_DIR / out_path
    out_path.parent.mkdir(parents=True, exist_ok=True)
    p1 = C.build_tuning_bot()
    t0 = time.time()
    best = coordinate_descent(seeds, p1, opponents, args.jobs, args.passes, args.objective, params, out_path)
    print("\n" + "═" * 56)
    print(f"DONE in {(time.time()-t0)/60:.1f} min. Best written to {out_path}")
    print("\nparams.rs DEFAULT changes:\n" + rust_lines(best))
    return 0


if __name__ == "__main__":
    sys.exit(main())
