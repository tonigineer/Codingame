#!/usr/bin/env python3
"""Fetch and parse a CodinGame Troll-Farm replay from a URL or game ID.

Usage:
  python3 parse_replay.py https://www.codingame.com/replay/891040557
  python3 parse_replay.py 891040557
"""

import json, re, sys
from pathlib import Path
import requests

API = "https://www.codingame.com/services/gameResult/findByGameId"

def fetch_replay(game_id: int) -> dict:
    r = requests.post(API, json=[game_id, None], timeout=30)
    r.raise_for_status()
    return r.json()


def get_first_view_grid(data: dict) -> list[str]:
    """Extract the static map grid from the first frame's view field."""
    if not data["frames"]:
        return []
    raw = data["frames"][0].get("view", "")
    # view = "<frame_nr>\n<json>"
    if "\n" not in raw:
        return []
    _, json_str = raw.split("\n", 1)
    view_data = json.loads(json_str)
    inputmodule = view_data.get("global", {}).get("inputmodule", "")
    lines = inputmodule.split("\n")
    # grid is line 1 onwards (line 0 is "width height")
    return lines[1:] if len(lines) > 1 else []


def parse_referee_input(text: str) -> dict:
    info = {}
    for m in re.finditer(r"(\w+)=(\S+)", text):
        info[m.group(1)] = m.group(2)
    return info


def describe_grid(grid: list[str]) -> dict:
    width = len(grid[0]) if grid else 0
    height = len(grid)
    water = sum(row.count("~") for row in grid)
    iron = sum(row.count("+") for row in grid)
    shacks = {}
    for y, row in enumerate(grid):
        for x, ch in enumerate(row):
            if ch in "01":
                shacks[f"P{int(ch) + 1}"] = (x, y)
    shack_dist = -1
    if "P1" in shacks and "P2" in shacks:
        (ax, ay), (bx, by) = shacks["P1"], shacks["P2"]
        shack_dist = abs(ax - bx) + abs(ay - by)
    return {
        "width": width, "height": height, "water": water,
        "iron_mines": iron, "shacks": shacks, "shack_distance": shack_dist,
    }


def main():
    if len(sys.argv) < 2:
        print(__doc__.strip())
        return 1

    arg = sys.argv[1]
    m = re.search(r"replay/(\d+)", arg)
    game_id = int(m.group(1)) if m else int(arg)

    print(f"Fetching replay {game_id} ...")
    data = fetch_replay(game_id)

    print("\n=== PLAYERS ===")
    for a in data["agents"]:
        codingamer = a["codingamer"]
        print(f"  Player {a['index'] + 1}: {codingamer['pseudo']} "
              f"(score {data['scores'][a['index']]})")

    print(f"\n=== MAP ===")
    ref_info = parse_referee_input(data["refereeInput"])
    print(f"  Seed: {ref_info.get('seed', '?')}")
    grid = get_first_view_grid(data)
    ginfo = describe_grid(grid)
    print(f"  Size: {ginfo['width']}x{ginfo['height']}")
    print(f"  Water cells: {ginfo['water']}")
    print(f"  Iron mines: {ginfo['iron_mines']}")
    print(f"  Shack distance: {ginfo['shack_distance']}")
    print(f"  Shacks: {ginfo['shacks']}")
    print("\n  Grid:")
    for row in grid:
        print(f"    {row}")

    print(f"\n=== FRAMES ===")
    print(f"  Total: {len(data['frames'])}")
    if data["frames"]:
        last = data["frames"][-1]
        print(f"  Last frame summary: {last.get('summary', '')[:200]}")

    print(f"\n=== SCORES ===")
    print(f"  {data['scores']}")

    # Save raw JSON for further analysis
    replays_dir = Path(__file__).resolve().parent.parent / "replays"
    replays_dir.mkdir(parents=True, exist_ok=True)
    out = replays_dir / f"replay_{game_id}.json"
    out.write_text(json.dumps(data, indent=2))
    print(f"\nFull replay saved to {out}")


if __name__ == "__main__":
    sys.exit(main())
