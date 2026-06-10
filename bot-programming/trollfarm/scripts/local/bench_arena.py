#!/usr/bin/env python3
"""The standard regression bench: 10 pinned ARENA seeds (5 lost + 5 won).

Runs the candidate bot (and optionally a baseline binary) on the maps of 5
recently LOST and 5 recently WON arena games (seeds pinned in
eval/arena_bench_seeds.json), against one or more local reference opponents.
Reports per-seed margins (candidate vs baseline, same seed = paired) plus the
candidate's failed-action count per game.

Reading the table: WON seeds must stay comfortable wins (regression guard);
LOST seeds show whether a change helps where it matters. NB: 10 paired games
are noisy (single-map margins swing +-30) — corroborate risky changes with a
big harness.py run before believing a small diff.

Usage:
  python scripts/local/bench_arena.py                          # ./trollfarm vs baseline ./trollfarm-prev
  python scripts/local/bench_arena.py --no-baseline            # candidate only
  python scripts/local/bench_arena.py --baseline ./trollfarm-base --opps trollfarm-spar-v1
"""
import argparse
import json
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import _common as C

SEEDS_FILE = C.EVAL_DIR / "arena_bench_seeds.json"
JAR = "troll-farm-1.0-SNAPSHOT.jar"


def run_game(bot: str, opp: str, seed: str) -> dict:
    """One local referee game; returns scores + p1 failed-action count."""
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
        out = tf.name
    subprocess.run(
        ["java", "-jar", JAR, "-p1", bot, "-p2", f"./{opp}", "-l", out, "-seed", seed],
        cwd=C.GAME_DIR,
        capture_output=True,
        timeout=300,
    )
    d = json.loads(Path(out).read_text())
    Path(out).unlink()
    fails = sum(
        1
        for s in d.get("summaries") or []
        for line in (s or "").splitlines()
        if re.match(r"\$0: \[failed\]", line.strip())
    )
    return {"p1": d["scores"].get("0"), "p2": d["scores"].get("1"), "fails": fails}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--p1", default="./trollfarm", help="candidate binary (in codingame/)")
    ap.add_argument("--baseline", default="./trollfarm-prev", help="baseline binary")
    ap.add_argument("--no-baseline", action="store_true")
    ap.add_argument(
        "--opps",
        default="trollfarm-spar-v1,trollfarm-ref-gold-70",
        help="comma-separated reference opponents (in codingame/)",
    )
    ap.add_argument("--jobs", type=int, default=8)
    args = ap.parse_args()

    cfg = json.loads(SEEDS_FILE.read_text())
    seeds = [("LOST", e) for e in cfg["lost"]] + [("WON ", e) for e in cfg["won"]]
    opps = args.opps.split(",")
    bots = [args.p1] + ([] if args.no_baseline else [args.baseline])

    jobs = [(bot, opp, e["seed"]) for bot in bots for opp in opps for _, e in seeds]
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        results = dict(zip(jobs, ex.map(lambda j: run_game(*j), jobs)))

    for opp in opps:
        print(f"\n=== vs {opp} ===")
        hdr = f"{'arena':<24} {'cand':>9} {'fails':>5}"
        if not args.no_baseline:
            hdr += f" | {'base':>9} {'Δmargin':>8}"
        print(hdr)
        tot_d, cand_w, base_w = 0, 0, 0
        for tag, e in seeds:
            c = results[(args.p1, opp, e["seed"])]
            row = f"{tag} {e['opp'][:9]:<9} {e['result']:<9} {c['p1']:>4}-{c['p2']:<4} {c['fails']:>5}"
            cand_w += c["p1"] > c["p2"]
            if not args.no_baseline:
                b = results[(args.baseline, opp, e["seed"])]
                d = (c["p1"] - c["p2"]) - (b["p1"] - b["p2"])
                tot_d += d
                base_w += b["p1"] > b["p2"]
                row += f" | {b['p1']:>4}-{b['p2']:<4} {d:>+8}"
            print(row)
        foot = f"candidate wins {cand_w}/10"
        if not args.no_baseline:
            foot += f"  (baseline {base_w}/10)   total Δmargin {tot_d:+d}"
        print(foot)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
