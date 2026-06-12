"""Download a player's arena replays via the anonymous public API.

The arena is selected by `--game` (its puzzle URL slug comes from
`_browser.PUZZLE_SLUGS`). Replays are saved to
bots/<game>/replays/<pseudo>/<game_id>.json; existing files are skipped, so
re-running tops up with new games. Equivalent: `just get-games <player>`
from the bot dir.

Usage:
  uv run --project tools tools/ide/fetch_player_games.py --game trollfarm tonigineer
  uv run --project tools tools/ide/fetch_player_games.py --game soak-overflow 4083906 --limit 5
  uv run --project tools tools/ide/fetch_player_games.py --game soak-overflow --list-top 20
"""

import argparse
import json
import sys
import time

import requests

import _browser as B

LEADERBOARD_API = (
    "https://www.codingame.com/services/Leaderboards/getFilteredPuzzleLeaderboard"
)
GAMES_API = (
    "https://www.codingame.com/services/gamesPlayersRanking/findLastBattlesByAgentId"
)
REPLAY_API = "https://www.codingame.com/services/gameResult/findByGameId"
FILTER = {"active": False, "keyword": "", "column": "", "filter": ""}


def fetch_leaderboard(puzzle: str) -> dict:
    r = requests.post(LEADERBOARD_API, json=[puzzle, "", "global", FILTER], timeout=30)
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


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("query", nargs="?", help="player pseudo or user id")
    ap.add_argument("--limit", type=int, default=None, help="download at most N games")
    ap.add_argument(
        "--list-top", type=int, default=None, metavar="N", help="list top N players"
    )
    ap.add_argument(
        "--game", required=True, help="game whose arena to query (bot crate in bots/)"
    )
    args = ap.parse_args()

    puzzle = B.puzzle_slug(args.game)
    print(f"Fetching leaderboard ({puzzle}) ...", file=sys.stderr)
    data = fetch_leaderboard(puzzle)

    if args.list_top is not None:
        list_top(data, args.list_top)
        return 0
    if not args.query:
        ap.error("query (pseudo or user id) required unless --list-top is given")

    print(f"Done. {data['count']} players on leaderboard.", file=sys.stderr)
    player = find_player(data, args.query)
    if not player:
        print(f"Player '{args.query}' not found in top {data['count']}.")
        names = [u["pseudo"] for u in data["users"][:10]]
        print(f"Top 10: {', '.join(names)}")
        return 1

    pseudo = player["pseudo"]
    agent_id = player["agentId"]
    print(
        f"\nPlayer: {pseudo}  (agentId={agent_id}, userId={player['codingamer']['userId']})"
    )

    print("Fetching game list...", file=sys.stderr)
    game_ids = fetch_game_ids(agent_id)
    if not game_ids:
        print("No games found.")
        return 0
    if args.limit:
        game_ids = game_ids[: args.limit]

    print(f"Found {len(game_ids)} games. Downloading replays...\n")
    out_dir = B.bot_dir(args.game) / "replays" / pseudo
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
        except Exception as e:  # noqa: BLE001
            print(f"  [{i}/{len(game_ids)}] {gid}  ERROR: {e}")
        time.sleep(0.3)

    print(f"\nAll replays saved to {out_dir}/")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
