#!/usr/bin/env python3
"""Held-out validation of a tuned candidate config vs the shipped defaults.

`sweep_all.py` optimises greedily, so it can ratchet in evaluation noise
(dropped/crashed games make even a *mechanically inert* parameter look like an
improvement). This re-checks the winning config on a FRESH seed set — a base
seed the sweep never saw — to tell real gains from noise, and decomposes it
one-at-a-time (OAT): baseline + each single change in isolation, so you can see
which individual changes actually survive on held-out maps.

It reads the candidate from `sweep_all_best.json` (the `changed_from_default`
block), so it always validates whatever the latest sweep produced.

Determinism note: with all games completing, a config's score is exactly
reproducible. Any OAT row that changes nothing behaviourally (e.g. an inert
param) MUST show a 0.00 delta here; a nonzero delta on such a row means games
are still crashing and dropping — check the `n=` completion counts.

Usage:
  python3 scripts/local/validate.py                      # 150 games, seed 2, eco+v1+gold-X
  python3 scripts/local/validate.py --games 200 --jobs 8
  python3 scripts/local/validate.py --no-oat             # baseline vs full candidate only
"""
import argparse
import json
import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import _common as C
import harness as ev

DEFAULT_OPPONENTS = ["./trollfarm-spar-eco", "./trollfarm-spar-v1", "./trollfarm-ref-gold-X"]
P1 = "./trollfarm-tuning"


def clear_tf() -> None:
    """Drop every TF_* override so each config starts from the compiled defaults."""
    for k in [k for k in os.environ if k.startswith("TF_")]:
        os.environ.pop(k, None)


def safe_game(i: int, s: int, p1: str, p2: str):
    """run_game with one retry (the referee SIGSEGVs sporadically under load)."""
    for _ in range(2):
        try:
            return ev.run_game(i, s, p1, p2)
        except Exception:  # noqa: BLE001
            continue
    return None


def run_config(config: dict, seeds: list[int], opponents: list[str], jobs: int) -> dict:
    """Run one config vs each opponent; return {opp: {margin, win, n}} + objective."""
    clear_tf()
    C.set_tf_env(config)
    per_opp = {}
    for opp in opponents:
        margins, wins = [], 0
        with ThreadPoolExecutor(max_workers=jobs) as pool:
            futs = [pool.submit(safe_game, i, s, P1, opp) for i, s in enumerate(seeds)]
            for f in as_completed(futs):
                r = f.result()
                if r is None:
                    continue
                margins.append(r.margin)
                wins += r.outcome == "WIN"
        n = len(margins)
        per_opp[opp] = {
            "margin": sum(margins) / max(n, 1),
            "win": wins / max(n, 1),
            "n": n,
        }
    obj = sum(v["margin"] for v in per_opp.values()) / len(per_opp)
    return {"per_opp": per_opp, "obj": obj}


def short(opp: str) -> str:
    return opp.removeprefix("./").removeprefix("trollfarm-")


def fmt_row(label: str, res: dict, opponents: list[str], base_obj: float | None) -> str:
    cells = []
    for opp in opponents:
        v = res["per_opp"][opp]
        flag = "" if v["n"] == len(LAST_SEEDS) else f"!{v['n']}"
        cells.append(f"{v['margin']:+6.1f}/{v['win'] * 100:4.0f}%{flag}")
    delta = "" if base_obj is None else f"  (Δ {res['obj'] - base_obj:+.2f})"
    return f"  {label:<26} obj {res['obj']:+7.2f}{delta}   " + "  ".join(cells)


LAST_SEEDS: list[int] = []


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--games", type=int, default=150, help="games per opponent (default 150)")
    ap.add_argument("--seed", type=int, default=2, help="HELD-OUT base seed (sweep used 1)")
    ap.add_argument("--jobs", type=int, default=8, help="parallel games (lower = fewer crashes)")
    ap.add_argument("--opponents", default=",".join(DEFAULT_OPPONENTS))
    ap.add_argument(
        "--candidate",
        default=str(C.EVAL_DIR / "sweep_all_best.json"),
        help="sweep output json (uses its changed_from_default block)",
    )
    ap.add_argument("--no-oat", action="store_true", help="skip the per-change decomposition")
    ap.add_argument("--no-build", action="store_true")
    ap.add_argument("--out", default=str(C.EVAL_DIR / "validate.json"))
    args = ap.parse_args()

    global LAST_SEEDS
    opponents = args.opponents.split(",")
    seeds = LAST_SEEDS = C.make_seeds(args.games, args.seed)

    candidate = json.loads(Path(args.candidate).read_text())["changed_from_default"]

    print(f"Held-out validation | {args.games} games/opp | base seed {args.seed} (sweep used 1)")
    print(f"Opponents: {[short(o) for o in opponents]}")
    print(f"Candidate ({len(candidate)} changes): {candidate}")
    n_configs = 2 + (0 if args.no_oat else len(candidate))
    print(f"{n_configs} configs × {len(opponents)} opp × {args.games} games "
          f"= {n_configs * len(opponents) * args.games} games\n")

    if not args.no_build:
        C.build_tuning_bot()

    t0 = time.time()
    results = {}

    print("─" * 90)
    base = results["baseline"] = run_config({}, seeds, opponents, args.jobs)
    print(fmt_row("baseline (defaults)", base, opponents, None))

    full = results["candidate"] = run_config(candidate, seeds, opponents, args.jobs)
    print(fmt_row("FULL candidate", full, opponents, base["obj"]))
    print("─" * 90)

    oat = []
    if not args.no_oat:
        print("ONE-AT-A-TIME (baseline + a single change):")
        for k, v in candidate.items():
            res = run_config({k: v}, seeds, opponents, args.jobs)
            results[f"oat:{k}"] = res
            oat.append((k, v, res))
        # Sort by contribution so survivors float to the top.
        oat.sort(key=lambda t: t[2]["obj"] - base["obj"], reverse=True)
        for k, v, res in oat:
            print(fmt_row(f"+{k}={v}", res, opponents, base["obj"]))
        print("─" * 90)

    print(f"\nSummary (objective = mean margin across opponents):")
    print(f"  baseline        {base['obj']:+.2f}")
    print(f"  full candidate  {full['obj']:+.2f}   (Δ {full['obj'] - base['obj']:+.2f})")
    if oat:
        survivors = [(k, v, r) for k, v, r in oat if r["obj"] - base["obj"] > 0.5]
        inert = [(k, v, r) for k, v, r in oat if abs(r["obj"] - base["obj"]) <= 0.5]
        hurt = [(k, v, r) for k, v, r in oat if r["obj"] - base["obj"] < -0.5]
        print(f"  OAT survivors (Δobj > +0.5): "
              + (", ".join(f"{k}({r['obj'] - base['obj']:+.1f})" for k, v, r in survivors) or "none"))
        print(f"  OAT inert (|Δ| ≤ 0.5): "
              + (", ".join(k for k, v, r in inert) or "none"))
        print(f"  OAT harmful (Δobj < -0.5): "
              + (", ".join(f"{k}({r['obj'] - base['obj']:+.1f})" for k, v, r in hurt) or "none"))

    Path(args.out).write_text(json.dumps(
        {"games": args.games, "seed": args.seed, "opponents": opponents,
         "candidate": candidate,
         "results": {k: r for k, r in results.items()}}, indent=2))
    print(f"\nDone in {(time.time() - t0) / 60:.1f} min. Written to {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
