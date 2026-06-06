#!/usr/bin/env python3
"""Find all game replays for a player and download them.

Usage:
  python3 get_player_games.py <pseudo|user_id> [--limit N]
  python3 get_player_games.py tonigineer
  python3 get_player_games.py tonigineer --limit 5
  python3 get_player_games.py 4083906             # by user ID
  python3 get_player_games.py --list-top 20       # list top N players
"""

import json
import sys
import time
from pathlib import Path

import requests

LEADERBOARD_API = (
    "https://www.codingame.com/services/Leaderboards/getFilteredPuzzleLeaderboard"
)
GAMES_API = (
    "https://www.codingame.com/services/gamesPlayersRanking/findLastBattlesByAgentId"
)
REPLAY_API = "https://www.codingame.com/services/gameResult/findByGameId"
FILTER = {"active": False, "keyword": "", "column": "", "filter": ""}
PAYLOAD = ["spring-challenge-2026-troll-farm", "", "global", FILTER]
SCRIPT_DIR = Path(__file__).resolve().parent
REPLAYS_DIR = SCRIPT_DIR.parent / "replays"


def fetch_leaderboard() -> dict:
    r = requests.post(LEADERBOARD_API, json=PAYLOAD, timeout=30)
    r.raise_for_status()
    return r.json()


def find_player(data: dict, query: str) -> dict | None:
    query_lower = query.lower()
    for u in data.get("users", []):
        if query_lower in (u.get("pseudo") or "").lower():
            return u
        if str(u.get("codingamer", {}).get("userId")) == query:
            return u
    return None


def fetch_game_ids(agent_id: int) -> list[int]:
    r = requests.post(GAMES_API, json=[agent_id, None], timeout=30)
    r.raise_for_status()
    return [g["gameId"] for g in r.json() if "gameId" in g]


def fetch_replay(game_id: int) -> dict:
    r = requests.post(REPLAY_API, json=[game_id, None], timeout=30)
    r.raise_for_status()
    return r.json()


def describe_game(data: dict) -> str:
    agents = data.get("agents", [])
    scores = data.get("scores", [])
    parts = []
    for a in agents:
        name = a.get("codingamer", {}).get("pseudo", "?")
        idx = a.get("index", 0)
        score = scores[idx] if idx < len(scores) else "?"
        parts.append(f"{name}({score})")
    frames = len(data.get("frames", []))
    return " vs ".join(parts) + f"  [{frames} frames]"


def list_top(data: dict, n: int = 20):
    print(f"{'Rank':>5} {'Pseudo':<20} {'Score':>7} {'Lang':<10} {'Country':<6}")
    print("-" * 55)
    for u in data.get("users", [])[:n]:
        ca = u["codingamer"]
        score = f"{u['score']:.2f}"
        lang = u.get("programmingLanguage", "?")
        country = ca.get("countryId", "") or ""
        print(
            f"{u['rank']:>5} {u['pseudo'][:20]:<20} {score:>7} {lang[:10]:<10} {country:<6}"
        )


def main():
    if not sys.argv[1:]:
        print(__doc__)
        return 1

    limit = None
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    for a in sys.argv[1:]:
        if a.startswith("--limit="):
            limit = int(a.split("=", 1)[1])
        elif a == "--limit" and len(sys.argv) > sys.argv.index(a) + 1:
            limit = int(sys.argv[sys.argv.index(a) + 1])

    if args and args[0] == "--list-top":
        n = int(args[1]) if len(args) > 1 else 20
        print("Fetching leaderboard...", file=sys.stderr)
        data = fetch_leaderboard()
        list_top(data, n)
        return 0

    query = args[0] if args else ""
    print(f"Fetching leaderboard...", file=sys.stderr)
    data = fetch_leaderboard()
    print(f"Done. {data['count']} players on leaderboard.", file=sys.stderr)

    player = find_player(data, query)
    if not player:
        print(f"Player '{query}' not found in top {data['count']}.")
        names = [u["pseudo"] for u in data["users"][:10]]
        print(f"Top 10: {', '.join(names)}")
        return 1

    pseudo = player["pseudo"]
    agent_id = player["agentId"]
    print(
        f"\nPlayer: {pseudo}  (agentId={agent_id}, userId={player['codingamer']['userId']})"
    )

    print(f"Fetching game list...", file=sys.stderr)
    game_ids = fetch_game_ids(agent_id)

    if not game_ids:
        print("No games found.")
        return 0

    if limit:
        game_ids = game_ids[:limit]

    print(f"Found {len(game_ids)} games. Downloading replays...\n")

    out_dir = REPLAYS_DIR / pseudo
    out_dir.mkdir(parents=True, exist_ok=True)

    for i, gid in enumerate(game_ids, 1):
        out_path = out_dir / f"{gid}.json"
        if out_path.exists():
            print(f"  [{i}/{len(game_ids)}] {gid} — already exists, skipping")
            continue

        try:
            replay = fetch_replay(gid)
            desc = describe_game(replay)
            out_path.write_text(json.dumps(replay, indent=2))
            print(f"  [{i}/{len(game_ids)}] {gid}  {desc}")
        except Exception as e:
            print(f"  [{i}/{len(game_ids)}] {gid}  ERROR: {e}")

        time.sleep(0.3)

    print(f"\nAll replays saved to {out_dir}/")


if __name__ == "__main__":
    main()
