#!/usr/bin/env python3
"""Run N Troll-Farm games sequentially and summarize the results."""

import argparse
import json
import random
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
GAME_DIR = SCRIPT_DIR / "assets/Troll-Farm"
PKG = "trollfarm"
JAR = "troll-farm-1.0-SNAPSHOT.jar"


@dataclass
class GameResult:
    index: int
    seed: int
    p1: int
    p2: int

    @property
    def outcome(self) -> str:
        if self.p1 > self.p2:
            return "WIN"
        if self.p1 < self.p2:
            return "LOSS"
        return "DRAW"


def run_game(index: int, seed: int) -> GameResult:
    with tempfile.TemporaryDirectory(prefix=f"troll_{index}_") as tmp:
        cmd = [
            "java", "-jar", f"./{JAR}",
            "-p1", f"./{PKG}",
            "-p2", f"./{PKG}-ref",
            "-s", "-seed", str(seed)
        ]

        print(" ".join(cmd))

        proc = subprocess.run(
            cmd, cwd=GAME_DIR,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )

        if proc.returncode != 0:
            tail = "\n".join(proc.stdout.splitlines()[-30:])
            raise RuntimeError(
                f"Game {index} (seed {seed}) failed (exit {proc.returncode}):\n{tail}"
            )

        game_json = Path("/tmp/codingame/game.json")
        if not game_json.exists():
            raise FileNotFoundError(f"game.json not found for game {index}")

        data = json.loads(game_json.read_text())
        scores = data["scores"]
        return GameResult(index, seed, int(scores["0"]), int(scores["1"]))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Troll-Farm games sequentially.")
    parser.add_argument("num_games", type=int, nargs="?", default=1)
    parser.add_argument("--seed", type=int, default=None,
                        help="base seed; if set, games use seed, seed+1, ... "
                             "otherwise each game gets a random seed")
    args = parser.parse_args()

    if not GAME_DIR.exists():
        print(f"Game dir not found: {GAME_DIR}", file=sys.stderr)
        return 1

    if args.seed is not None:
        seeds = [args.seed + i for i in range(args.num_games)]
    else:
        INT64_MAX = 2**63 - 1
        INT64_MIN = -2**63
        seeds = [random.randint(INT64_MIN, INT64_MAX) for _ in range(args.num_games)]

    results: list[GameResult] = []

    for i in range(args.num_games):
        res = run_game(i, seeds[i])
        results.append(res)
        print(f"Game {res.index + 1:3d}/{args.num_games:<4d} (seed {res.seed:11d}): "
              f"{res.p1:3d} vs {res.p2:3d}  [{res.outcome}]")

    n = len(results)
    wins = sum(1 for r in results if r.outcome == "WIN")
    losses = sum(1 for r in results if r.outcome == "LOSS")
    draws = sum(1 for r in results if r.outcome == "DRAW")
    avg_p1 = sum(r.p1 for r in results) / n if n else 0
    avg_p2 = sum(r.p2 for r in results) / n if n else 0

    print("\u2500" * 40)
    print(f"Games played: {n}")
    print(f"Wins:   {wins}")
    print(f"Losses: {losses}")
    print(f"Draws:  {draws}")
    print(f"Win rate: {wins / n * 100:.1f}%" if n else "Win rate: n/a")
    print(f"Avg score: {avg_p1:.1f} vs {avg_p2:.1f}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
