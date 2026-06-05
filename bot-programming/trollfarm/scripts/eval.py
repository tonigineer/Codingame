#!/usr/bin/env python3
"""Benchmark the Troll-Farm bot over many reproducible games.

Runs N games in parallel against a reference bot, parses the referee replay
log (``-l``) for scores + map metadata, and parses our bot's per-turn
``[INV]`` stderr (both shack inventories) for score trajectories, final-score
composition, wasted-fruit and game-length stats. Prints a summary, writes a
results JSON, and renders a multi-panel matplotlib figure.

Key facts about the referee jar (verified):
  * Run WITHOUT ``-s``. ``-s`` starts a web server that never exits and makes
    every game hang. Use ``-l <file>`` to dump the replay JSON instead; the
    process exits cleanly in ~0.15s/game.
  * Games are DETERMINISTIC per seed (same seed -> identical score).
  * Map size VARIES by seed (16x8 .. 22x11).
  * ``--seed`` is a *base* that seeds a local RNG; the actual per-game seeds
    are drawn across the full int64 range. Same base -> same seed set.

Examples:
  python3 eval.py                      # 100 games, base seed 1, vs gold-X
  python3 eval.py 1000 --seed 1 --jobs 24 --label baseline
  python3 eval.py --p2 ./trollfarm-ref-gold-3
"""

import argparse
import json
import random
import re
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict, dataclass, field
from pathlib import Path

import numpy as np

SCRIPT_DIR = Path(__file__).resolve().parent
GAME_DIR = SCRIPT_DIR.parent / "codingame"
JAR = "troll-farm-1.0-SNAPSHOT.jar"
DEFAULT_P1 = "./trollfarm"
DEFAULT_P2 = "./trollfarm-ref-gold-X"

INV_RE = re.compile(
    r"turn=(\d+) "
    r"me_shack\[P(\d+) L(\d+) A(\d+) B(\d+) I(\d+) W(\d+)\] "
    r"opp_shack\[P(\d+) L(\d+) A(\d+) B(\d+) I(\d+) W(\d+)\]"
)
WOOD_POINTS = 4


@dataclass
class GameResult:
    index: int
    seed: int
    p1: int
    p2: int
    width: int
    height: int
    water: int
    iron_mines: int
    shack_dist: int        # Manhattan distance between the two shacks
    game_length: int       # last turn our bot saw
    # Final shack composition (from our [INV] stderr).
    my_fruit: int
    my_wood: int
    my_iron: int
    opp_fruit: int
    opp_wood: int
    opp_iron: int
    # Score trajectory: list of (turn, my_score, opp_score). Not serialized.
    traj: list = field(default_factory=list, repr=False, compare=False)

    @property
    def margin(self) -> int:
        return self.p1 - self.p2

    @property
    def area(self) -> int:
        return self.width * self.height

    @property
    def outcome(self) -> str:
        if self.p1 > self.p2:
            return "WIN"
        if self.p1 < self.p2:
            return "LOSS"
        return "DRAW"


def parse_map(log: dict) -> tuple[int, int, int, int, int]:
    """Extract (width, height, water, iron_mines, shack_dist) from the log."""
    view0 = next((v for v in log["views"] if v and "{" in v), None)
    if view0 is None:
        raise ValueError("no usable view frame in replay log")
    payload = json.loads(view0[view0.index("{"):])
    im = payload["global"]["inputmodule"]
    lines = im.split("\n")
    width, height = (int(v) for v in lines[0].split())
    grid = lines[1:1 + height]
    water = sum(row.count("~") for row in grid)
    iron = sum(row.count("+") for row in grid)
    shacks: dict[str, tuple[int, int]] = {}
    for y, row in enumerate(grid):
        for x, cell in enumerate(row):
            if cell in "01":
                shacks[cell] = (x, y)
    if "0" in shacks and "1" in shacks:
        (ax, ay), (bx, by) = shacks["0"], shacks["1"]
        shack_dist = abs(ax - bx) + abs(ay - by)
    else:
        shack_dist = -1
    return width, height, water, iron, shack_dist


def parse_inv(log: dict):
    """Parse our [INV] stderr lines into a trajectory + final composition.

    Returns (traj, final) where traj = [(turn, my_score, opp_score), ...] and
    final = (game_length, my_fruit, my_wood, my_iron, opp_fruit, opp_wood,
    opp_iron). [INV] is printed at the start of each of our turns, so the last
    line is the start-of-last-turn state (it equals the final score in
    practice, since the game ends with idle trolls).
    """
    traj: list[tuple[int, int, int]] = []
    final = None
    for blk in log["errors"].get("0", []):
        if not blk:
            continue
        for line in blk.split("\n"):
            if not line.startswith("[INV]"):
                continue
            m = INV_RE.search(line)
            if not m:
                continue
            g = [int(x) for x in m.groups()]
            turn = g[0]
            mf, mi, mw = g[1] + g[2] + g[3] + g[4], g[5], g[6]
            of, oi, ow = g[7] + g[8] + g[9] + g[10], g[11], g[12]
            traj.append((turn, mf + WOOD_POINTS * mw, of + WOOD_POINTS * ow))
            final = (turn, mf, mw, mi, of, ow, oi)
    return traj, final


def run_game(index: int, seed: int, p1: str, p2: str) -> GameResult:
    import subprocess

    with tempfile.NamedTemporaryFile(
        prefix=f"troll_{index}_", suffix=".json", delete=True
    ) as tmp:
        cmd = [
            "java", "-jar", f"./{JAR}",
            "-p1", p1, "-p2", p2,
            "-seed", str(seed),
            "-l", tmp.name,
        ]
        proc = subprocess.run(
            cmd, cwd=GAME_DIR,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
            timeout=120,
        )
        if proc.returncode != 0:
            tail = "\n".join(proc.stdout.splitlines()[-30:])
            raise RuntimeError(
                f"Game {index} (seed {seed}) failed (exit {proc.returncode}):\n{tail}"
            )
        log = json.loads(Path(tmp.name).read_text())

    scores = log["scores"]
    width, height, water, iron_mines, shack_dist = parse_map(log)
    traj, final = parse_inv(log)
    if final is None:
        final = (0, 0, 0, 0, 0, 0, 0)
    glen, mf, mw, mi, of, ow, oi = final
    return GameResult(
        index=index, seed=seed,
        p1=int(scores["0"]), p2=int(scores["1"]),
        width=width, height=height, water=water, iron_mines=iron_mines,
        shack_dist=shack_dist, game_length=glen,
        my_fruit=mf, my_wood=mw, my_iron=mi,
        opp_fruit=of, opp_wood=ow, opp_iron=oi,
        traj=traj,
    )


def summarize(results: list[GameResult], p1: str, p2: str, elapsed: float) -> dict:
    n = len(results)
    wins = sum(1 for r in results if r.outcome == "WIN")
    losses = sum(1 for r in results if r.outcome == "LOSS")
    draws = sum(1 for r in results if r.outcome == "DRAW")
    margins = np.array([r.margin for r in results])
    p1s = np.array([r.p1 for r in results])
    p2s = np.array([r.p2 for r in results])

    print("─" * 60)
    print(f"P1: {p1}")
    print(f"P2: {p2}")
    print(f"Games: {n}   ({elapsed:.1f}s, {elapsed / n:.2f}s/game)")
    print("─" * 60)
    print(f"Wins:   {wins:4d}  ({wins / n * 100:.1f}%)")
    print(f"Losses: {losses:4d}  ({losses / n * 100:.1f}%)")
    print(f"Draws:  {draws:4d}  ({draws / n * 100:.1f}%)")
    print(f"Avg score:  {p1s.mean():.1f} vs {p2s.mean():.1f}")
    print(f"Avg margin: {margins.mean():+.1f}  (std {margins.std():.1f}, "
          f"min {margins.min():+d}, max {margins.max():+d})")

    # Score composition (means).
    mf = np.mean([r.my_fruit for r in results])
    mw = np.mean([r.my_wood for r in results])
    of = np.mean([r.opp_fruit for r in results])
    ow = np.mean([r.opp_wood for r in results])
    gl = np.array([r.game_length for r in results])
    print("─" * 60)
    print("Avg final composition (us vs opp):")
    print(f"  leftover fruit (1pt): {mf:5.1f}  vs {of:5.1f}")
    print(f"  wood          (4pt): {mw:5.1f}  vs {ow:5.1f}   "
          f"-> wood pts {mw * 4:5.1f} vs {ow * 4:5.1f}")
    print(f"  avg game length: {gl.mean():.0f} turns "
          f"(min {gl.min()}, max {gl.max()})")

    print("─" * 60)
    print("By map size:")
    sizes: dict[tuple[int, int], list[GameResult]] = {}
    for r in results:
        sizes.setdefault((r.width, r.height), []).append(r)
    print(f"  {'map':>8}  {'games':>5}  {'win%':>5}  {'avg margin':>10}")
    for (w, h), rs in sorted(sizes.items(), key=lambda kv: kv[0][0] * kv[0][1]):
        ws = sum(1 for r in rs if r.outcome == "WIN")
        am = sum(r.margin for r in rs) / len(rs)
        print(f"  {w:>3}x{h:<3}  {len(rs):>5}  {ws / len(rs) * 100:>4.0f}%  {am:>+10.1f}")
    print("─" * 60)

    serial = []
    for r in results:
        d = asdict(r)
        d.pop("traj", None)
        serial.append(d)
    return {
        "p1": p1, "p2": p2, "games": n,
        "wins": wins, "losses": losses, "draws": draws,
        "win_rate": wins / n,
        "avg_p1": float(p1s.mean()), "avg_p2": float(p2s.mean()),
        "avg_margin": float(margins.mean()), "margin_std": float(margins.std()),
        "avg_my_fruit": float(mf), "avg_opp_fruit": float(of),
        "avg_my_wood": float(mw), "avg_opp_wood": float(ow),
        "avg_game_length": float(gl.mean()),
        "elapsed_s": elapsed,
        "results": serial,
    }


def stats_markdown(summary: dict, label: str) -> str:
    """One-run headline table for summary.md (so two runs read side by side)."""
    s = summary
    return (
        f"## `{label}`  ({s['p1']}  vs  {s['p2']})\n\n"
        f"| metric | value |\n|---|---|\n"
        f"| games | {s['games']} |\n"
        f"| win / loss / draw | {s['wins']} / {s['losses']} / {s['draws']} |\n"
        f"| **win rate** | **{s['win_rate'] * 100:.1f}%** |\n"
        f"| avg score (us vs opp) | {s['avg_p1']:.1f} vs {s['avg_p2']:.1f} |\n"
        f"| **avg margin** | **{s['avg_margin']:+.1f}** (std {s['margin_std']:.1f}) |\n"
        f"| leftover fruit (us vs opp) | {s['avg_my_fruit']:.1f} vs {s['avg_opp_fruit']:.1f} |\n"
        f"| wood (us vs opp) | {s['avg_my_wood']:.1f} vs {s['avg_opp_wood']:.1f} "
        f"(= {s['avg_my_wood'] * WOOD_POINTS:.0f} vs {s['avg_opp_wood'] * WOOD_POINTS:.0f} pts) |\n"
        f"| avg game length | {s['avg_game_length']:.0f} turns |\n"
    )


# Each plot is a (filename-stem, title, draw_fn). draw_fn(ax, D) reads the
# precomputed data dict D and draws onto a single axis, so the same function
# renders both the standalone file and the combined overview. The stems are
# the stable filenames referenced by eval.md (keep them in sync).
def _trend(ax, x, y, fmt):
    if len(set(x.tolist())) > 1:
        z = np.polyfit(x, y, 1)
        xr = np.array([x.min(), x.max()])
        ax.plot(xr, np.polyval(z, xr), "tab:orange", lw=2, label=fmt.format(z[0]))
        ax.legend()


def _plot_data(results: list["GameResult"]) -> dict:
    D: dict = {"results": results}
    D["p1"] = np.array([r.p1 for r in results])
    D["p2"] = np.array([r.p2 for r in results])
    D["margin"] = D["p1"] - D["p2"]
    D["area"] = np.array([r.area for r in results])
    D["shack_dist"] = np.array([r.shack_dist for r in results])
    D["water"] = np.array([r.water for r in results])
    D["gl"] = np.array([r.game_length for r in results])
    D["myf"] = np.array([r.my_fruit for r in results])
    D["opf"] = np.array([r.opp_fruit for r in results])
    D["myw"] = np.array([r.my_wood for r in results])
    D["opw"] = np.array([r.opp_wood for r in results])
    D["colors"] = ["tab:green" if m > 0 else "tab:red" if m < 0 else "tab:gray"
                   for m in D["margin"]]
    grid = np.linspace(0, 1, 50)
    myc, opc = [], []
    for r in results:
        if len(r.traj) < 2:
            continue
        t = np.array([p[0] for p in r.traj], dtype=float)
        t = (t - t.min()) / (t.max() - t.min() + 1e-9)
        myc.append(np.interp(grid, t, [p[1] for p in r.traj]))
        opc.append(np.interp(grid, t, [p[2] for p in r.traj]))
    D["grid"] = grid
    D["myc"] = np.array(myc) if myc else None
    D["opc"] = np.array(opc) if opc else None
    return D


def _p_score_scatter(ax, D):
    lim = max(D["p1"].max(), D["p2"].max()) + 5
    ax.plot([0, lim], [0, lim], "k--", alpha=0.4, lw=1)
    ax.scatter(D["p2"], D["p1"], c=D["colors"], alpha=0.5, edgecolors="none", s=18)
    ax.set_xlabel("opponent (P2)"); ax.set_ylabel("our (P1)")
    ax.set_title("Per-game scores (above line = win)")
    ax.set_xlim(0, lim); ax.set_ylim(0, lim)


def _p_margin_hist(ax, D):
    m = D["margin"]
    ax.hist(m, bins=30, color="tab:blue", alpha=0.8)
    ax.axvline(0, color="k", lw=1)
    ax.axvline(m.mean(), color="tab:orange", lw=2, label=f"mean {m.mean():+.1f}")
    ax.set_xlabel("margin (P1 - P2)"); ax.set_ylabel("games")
    ax.set_title("Margin distribution"); ax.legend()


def _p_margin_cdf(ax, D):
    sm = np.sort(D["margin"])
    ax.plot(sm, np.linspace(0, 100, len(sm)), color="tab:blue")
    ax.axvline(0, color="k", lw=1)
    loss = 100 * np.mean(D["margin"] < 0)
    ax.set_xlabel("margin (P1 - P2)"); ax.set_ylabel("cumulative %")
    ax.set_title(f"Margin CDF ({loss:.0f}% of games are losses)"); ax.grid(alpha=0.3)


def _p_winrate_by_size(ax, D):
    groups: dict[int, list] = {}
    for r in D["results"]:
        groups.setdefault(r.area, []).append(r)
    xs = sorted(groups)
    win_pct = [100 * sum(1 for r in groups[a] if r.margin > 0) / len(groups[a]) for a in xs]
    labels = [f"{groups[a][0].width}x{groups[a][0].height}\n(n={len(groups[a])})" for a in xs]
    bars = ax.bar(range(len(xs)), win_pct, color="tab:purple", alpha=0.8)
    ax.axhline(50, color="k", ls="--", alpha=0.5)
    ax.set_xticks(range(len(xs))); ax.set_xticklabels(labels, fontsize=8)
    ax.set_ylabel("win %"); ax.set_ylim(0, 100); ax.set_title("Win rate by map size")
    for b, p in zip(bars, win_pct):
        ax.text(b.get_x() + b.get_width() / 2, p + 1, f"{p:.0f}", ha="center", fontsize=8)


def _p_margin_vs_shackdist(ax, D):
    sd = D["shack_dist"]
    jit = (np.random.rand(len(sd)) - 0.5) * 0.6
    ax.scatter(sd + jit, D["margin"], c=D["colors"], alpha=0.5, edgecolors="none", s=18)
    ax.axhline(0, color="k", lw=1)
    _trend(ax, sd, D["margin"], "slope {:+.2f}/cell")
    ax.set_xlabel("shack Manhattan distance"); ax.set_ylabel("margin")
    ax.set_title("Margin vs shack distance")


def _p_margin_vs_water(ax, D):
    wt = D["water"]
    ax.scatter(wt, D["margin"], c=D["colors"], alpha=0.5, edgecolors="none", s=18)
    ax.axhline(0, color="k", lw=1)
    _trend(ax, wt, D["margin"], "slope {:+.2f}")
    ax.set_xlabel("water cells"); ax.set_ylabel("margin")
    ax.set_title("Margin vs water count")


def _p_game_length(ax, D):
    gl = D["gl"]
    ax.hist(gl, bins=30, color="teal", alpha=0.8)
    ax.axvline(gl.mean(), color="tab:orange", lw=2, label=f"mean {gl.mean():.0f}")
    ax.axvline(300, color="k", ls="--", alpha=0.5, label="max 300")
    ax.set_xlabel("game length (turns)"); ax.set_ylabel("games")
    ax.set_title("Game length"); ax.legend()


def _p_margin_vs_length(ax, D):
    gl = D["gl"]
    ax.scatter(gl, D["margin"], c=D["colors"], alpha=0.5, edgecolors="none", s=18)
    ax.axhline(0, color="k", lw=1)
    _trend(ax, gl, D["margin"], "slope {:+.3f}")
    ax.set_xlabel("game length (turns)"); ax.set_ylabel("margin")
    ax.set_title("Margin vs game length")


def _p_wasted_fruit(ax, D):
    myf, opf = D["myf"], D["opf"]
    bins = range(0, int(max(myf.max(), opf.max())) + 2)
    ax.hist(myf, bins=bins, alpha=0.6, label=f"us (mean {myf.mean():.1f})", color="tab:red")
    ax.hist(opf, bins=bins, alpha=0.6, label=f"opp (mean {opf.mean():.1f})", color="tab:green")
    ax.set_xlabel("leftover fruit in shack (1pt each, unconverted)")
    ax.set_ylabel("games"); ax.set_title("Wasted fruit at game end"); ax.legend()


def _p_composition(ax, D):
    comp = [
        ("our fruit", D["myf"].mean(), "tab:orange"),
        ("our wood", D["myw"].mean() * WOOD_POINTS, "tab:red"),
        ("opp fruit", D["opf"].mean(), "gold"),
        ("opp wood", D["opw"].mean() * WOOD_POINTS, "tab:green"),
    ]
    ax.bar([c[0] for c in comp], [c[1] for c in comp], color=[c[2] for c in comp])
    ax.set_ylabel("avg points"); ax.set_title("Score composition (points from fruit vs wood)")
    for i, c in enumerate(comp):
        ax.text(i, c[1] + 0.5, f"{c[1]:.1f}", ha="center", fontsize=9)
    ax.tick_params(axis="x", labelsize=8)


def _p_score_traj(ax, D):
    if D["myc"] is not None:
        g = D["grid"] * 100
        ax.plot(g, D["myc"].mean(0), color="tab:blue", lw=2, label="us")
        ax.fill_between(g, np.percentile(D["myc"], 25, 0),
                        np.percentile(D["myc"], 75, 0), color="tab:blue", alpha=0.2)
        ax.plot(g, D["opc"].mean(0), color="tab:red", lw=2, label="opp")
        ax.fill_between(g, np.percentile(D["opc"], 25, 0),
                        np.percentile(D["opc"], 75, 0), color="tab:red", alpha=0.2)
        ax.legend()
    ax.set_xlabel("game progress (%)"); ax.set_ylabel("score")
    ax.set_title("Mean score trajectory (band = 25-75 pct)")


def _p_margin_traj(ax, D):
    if D["myc"] is not None:
        g = D["grid"] * 100
        diff = D["myc"] - D["opc"]
        ax.plot(g, diff.mean(0), color="tab:purple", lw=2)
        ax.fill_between(g, np.percentile(diff, 25, 0),
                        np.percentile(diff, 75, 0), color="tab:purple", alpha=0.2)
        ax.axhline(0, color="k", lw=1)
    ax.set_xlabel("game progress (%)"); ax.set_ylabel("margin (us - opp)")
    ax.set_title("Mean margin trajectory (when the gap opens)")


PLOTS = [
    ("01_score_scatter", _p_score_scatter),
    ("02_margin_hist", _p_margin_hist),
    ("03_margin_cdf", _p_margin_cdf),
    ("04_winrate_by_mapsize", _p_winrate_by_size),
    ("05_margin_vs_shackdist", _p_margin_vs_shackdist),
    ("06_margin_vs_water", _p_margin_vs_water),
    ("07_game_length", _p_game_length),
    ("08_margin_vs_length", _p_margin_vs_length),
    ("09_wasted_fruit", _p_wasted_fruit),
    ("10_score_composition", _p_composition),
    ("11_score_trajectory", _p_score_traj),
    ("12_margin_trajectory", _p_margin_traj),
]


def make_plots(results: list[GameResult], out_dir: Path, label: str) -> None:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    D = _plot_data(results)
    run_dir = out_dir / label
    run_dir.mkdir(parents=True, exist_ok=True)

    # Individual files (referenced by eval.md).
    for stem, fn in PLOTS:
        fig, ax = plt.subplots(figsize=(6.5, 5))
        fn(ax, D)
        fig.tight_layout()
        fig.savefig(run_dir / f"{stem}.png", dpi=100)
        plt.close(fig)

    # Combined overview.
    fig, axes = plt.subplots(4, 3, figsize=(18, 20))
    fig.suptitle(f"Troll-Farm benchmark — {label}  (n={len(results)})", fontsize=16)
    for (stem, fn), ax in zip(PLOTS, axes.flat):
        fn(ax, D)
    fig.tight_layout(rect=(0, 0, 1, 0.98))
    fig.savefig(run_dir / "00_overview.png", dpi=100)
    plt.close(fig)
    print(f"Plots written to {run_dir}/ ({len(PLOTS)} panels + overview)")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("num_games", type=int, nargs="?", default=100)
    parser.add_argument("--seed", type=int, default=1,
                        help="base seed for the RNG that draws game seeds (default 1). "
                             "Same base -> same full-i64 game seeds (reproducible).")
    parser.add_argument("--p1", default=DEFAULT_P1, help="player 1 command (bot under test)")
    parser.add_argument("--p2", default=DEFAULT_P2, help="player 2 command (reference)")
    parser.add_argument("--jobs", type=int, default=8, help="parallel games (default 8)")
    parser.add_argument("--label", default="current", help="label for output files")
    parser.add_argument("--out", default=str(SCRIPT_DIR.parent / "eval"),
                        help="output directory for plots and results JSON")
    parser.add_argument("--no-plot", action="store_true")
    args = parser.parse_args()

    if not GAME_DIR.exists():
        print(f"Game dir not found: {GAME_DIR}", file=sys.stderr)
        return 1

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    INT64_MIN, INT64_MAX = -(2 ** 63), 2 ** 63 - 1
    rng = random.Random(args.seed)
    seeds = [rng.randint(INT64_MIN, INT64_MAX) for _ in range(args.num_games)]
    results: list[GameResult] = []
    start = time.time()

    print(f"Running {args.num_games} games | base seed {args.seed} "
          f"(full-i64 game seeds) | {args.jobs} parallel\n"
          f"  P1={args.p1}  P2={args.p2}")

    failed: list[int] = []
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = {
            pool.submit(run_game, i, seeds[i], args.p1, args.p2): i
            for i in range(args.num_games)
        }
        done = 0
        for fut in as_completed(futures):
            i = futures[fut]
            try:
                results.append(fut.result())
            except Exception as exc:  # noqa: BLE001 - tolerate flaky games
                failed.append(i)
                print(f"  game {i} (seed {seeds[i]}) failed: {exc}; will retry")
            done += 1
            if done % 50 == 0 or done == args.num_games:
                rate = done / (time.time() - start)
                print(f"  {done}/{args.num_games} done ({rate:.1f} games/s)")

    # Retry failures once, sequentially (contention is the usual cause).
    for i in failed[:]:
        try:
            results.append(run_game(i, seeds[i], args.p1, args.p2))
            failed.remove(i)
        except Exception as exc:  # noqa: BLE001
            print(f"  game {i} (seed {seeds[i]}) failed again: {exc}; skipping")
    if failed:
        print(f"  skipped {len(failed)} game(s) after retry")

    results.sort(key=lambda r: r.index)
    elapsed = time.time() - start

    summary = summarize(results, args.p1, args.p2, elapsed)
    run_dir = out_dir / args.label
    run_dir.mkdir(parents=True, exist_ok=True)
    (run_dir / "results.json").write_text(json.dumps(summary, indent=2))
    (run_dir / "summary.md").write_text(stats_markdown(summary, args.label))
    print(f"Results written to {run_dir}/results.json + summary.md")

    if not args.no_plot:
        make_plots(results, out_dir, args.label)

    return 0


if __name__ == "__main__":
    sys.exit(main())
