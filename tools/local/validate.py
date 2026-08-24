#!/usr/bin/env python3
"""Benchmark the current bot locally against a roster of opponent binaries.

Runs N games per opponent through the game's referee jar and reports W-L-D,
win rate, and score margins. Everything lives in bots/<game>/codingame/: the
referee jar (auto-discovered) and the opponent binaries. CodinGame SDK
referees share the same CLI (`-p1 -p2 -seed -l`); run with `-l` (log file),
never `-s` (starts a web server that never exits). Games are deterministic
per seed; `--seed` is a base that draws the per-game seeds, so the same base
replays the same maps for every opponent. The bot is rebuilt and deployed
first unless `--no-build` is given.

Usage:
  uv run --project tools tools/local/validate.py --game trollfarm 200 --seed 7 --jobs 16
  uv run --project tools tools/local/validate.py --game trollfarm --opponents ./trollfarm-ref-gold-X,./trollfarm-spar-v1
"""

import argparse
import io
import json
import random
import statistics
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

WORKSPACE_DIR = Path(__file__).resolve().parents[2]  # cargo workspace root
BOTS_DIR = WORKSPACE_DIR / "bots"

# Default sparring roster per game (binaries in bots/<game>/codingame/),
# used when --opponents is not given.
DEFAULT_OPPONENTS = {
    "trollfarm": ["./trollfarm-ref-gold-X"],
}

INT64_MIN, INT64_MAX = -(2**63), 2**63 - 1


def make_seeds(n: int, base: int) -> list[int]:
    """Draw `n` int64 game seeds from a base; the same base yields the same set."""
    rng = random.Random(base)
    return [rng.randint(INT64_MIN, INT64_MAX) for _ in range(n)]


def find_jar(game_dir: Path) -> str:
    jars = sorted(game_dir.glob("*.jar"))
    if not jars:
        raise SystemExit(f"no referee jar in {game_dir} — download the game's jar")
    if len(jars) > 1:
        print(f"  (multiple jars in {game_dir}, using {jars[0].name})")
    return jars[0].name


def build_and_deploy(game: str, game_dir: Path) -> None:
    print(f"Building {game} (release) ...")
    subprocess.run(
        ["cargo", "build", "--release", "-p", game], cwd=WORKSPACE_DIR, check=True
    )
    src = WORKSPACE_DIR / "target" / "release" / game
    dst = game_dir / game
    dst.write_bytes(src.read_bytes())
    dst.chmod(0o755)
    print(f"Deployed {dst}")


def run_game(game_dir: Path, jar: str, seed: int, p1: str, p2: str) -> tuple[int, int]:
    """Run one referee game; return (p1_score, p2_score)."""
    with tempfile.NamedTemporaryFile(prefix="cg_", suffix=".json") as tmp:
        cmd = ["java", "-jar", f"./{jar}", "-p1", p1, "-p2", p2]
        cmd += ["-seed", str(seed), "-l", tmp.name]
        proc = subprocess.run(
            cmd,
            cwd=game_dir,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=120,
        )
        if proc.returncode != 0:
            tail = "\n".join(proc.stdout.splitlines()[-15:])
            raise RuntimeError(f"seed {seed} failed (exit {proc.returncode}):\n{tail}")
        log = json.loads(Path(tmp.name).read_text())
    return int(log["scores"]["0"]), int(log["scores"]["1"])


def bench_opponent(
    game_dir: Path, jar: str, p1: str, p2: str, seeds: list[int], jobs: int
) -> dict:
    """Run all games against one opponent; return a stats dict (failed games
    are reported and skipped)."""
    margins: list[int] = []
    wins = losses = draws = failed = 0
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        futures = {ex.submit(run_game, game_dir, jar, s, p1, p2): s for s in seeds}
        for fut in as_completed(futures):
            try:
                mine, theirs = fut.result()
            except Exception as e:  # noqa: BLE001
                failed += 1
                print(f"  !! {str(e).splitlines()[0][:100]}")
                continue
            margins.append(mine - theirs)
            wins += mine > theirs
            losses += mine < theirs
            draws += mine == theirs
    n = len(margins)
    return {
        "opponent": p2,
        "games": n,
        "failed": failed,
        "wins": wins,
        "losses": losses,
        "draws": draws,
        "win_pct": 100.0 * wins / n if n else 0.0,
        "avg_margin": statistics.fmean(margins) if margins else 0.0,
        "med_margin": statistics.median(margins) if margins else 0.0,
    }


def main() -> int:
    if isinstance(sys.stdout, io.TextIOWrapper):
        sys.stdout.reconfigure(line_buffering=True)  # stream progress when piped
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("games", nargs="?", type=int, default=100, help="games/opponent")
    ap.add_argument("--game", required=True, help="game to bench (bot crate in bots/)")
    ap.add_argument(
        "--opponents",
        default=None,
        help="comma-separated opponent binaries in codingame/ "
        "(default: DEFAULT_OPPONENTS[game])",
    )
    ap.add_argument("--p1", default=None, help="our binary (default ./<game>)")
    ap.add_argument("--seed", type=int, default=1, help="base seed for the map draw")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--no-build", action="store_true", help="skip build + deploy")
    args = ap.parse_args()

    game_dir = BOTS_DIR / args.game / "codingame"
    jar = find_jar(game_dir)
    p1 = args.p1 or f"./{args.game}"
    if args.opponents:
        opponents = [o for o in args.opponents.split(",") if o]
    else:
        opponents = DEFAULT_OPPONENTS.get(args.game) or []
        if not opponents:
            ap.error(f"no default roster for {args.game!r} — pass --opponents")

    if not args.no_build:
        build_and_deploy(args.game, game_dir)

    seeds = make_seeds(args.games, args.seed)
    print(
        f"{p1} vs {len(opponents)} opponent(s), {args.games} games each "
        f"(base seed {args.seed}, jobs {args.jobs})\n"
    )

    t0 = time.time()
    rows = [
        bench_opponent(game_dir, jar, p1, opp, seeds, args.jobs) for opp in opponents
    ]

    width = max(len(r["opponent"]) for r in rows)
    print(f"\n{'opponent':<{width}}  games  W-L-D        win%   avg     med")
    print("─" * (width + 44))
    for r in rows:
        wld = f"{r['wins']}-{r['losses']}-{r['draws']}"
        fail = f"  ({r['failed']} failed)" if r["failed"] else ""
        print(
            f"{r['opponent']:<{width}}  {r['games']:>5}  {wld:<11} "
            f"{r['win_pct']:>5.1f}%  {r['avg_margin']:>+6.1f}  {r['med_margin']:>+5.1f}"
            f"{fail}"
        )
    total_w = sum(r["wins"] for r in rows)
    total_n = sum(r["games"] for r in rows)
    pct = 100.0 * total_w / total_n if total_n else 0.0
    print(f"\ntotal: {total_w}/{total_n} wins ({pct:.1f}%)  in {time.time() - t0:.0f}s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
