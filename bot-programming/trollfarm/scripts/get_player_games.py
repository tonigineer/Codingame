#!/usr/bin/env python3
"""Fetch the Troll-Farm leaderboard and find stats for a player.

Also demonstrates the APIs that work vs what doesn't for getting games.

Usage:
  python3 get_player_games.py <pseudo|user_id>
  python3 get_player_games.py delineate        # find by pseudo
  python3 get_player_games.py yamo
  python3 get_player_games.py --list-top 20    # list top N players
"""

import json, sys, requests

LEADERBOARD_API = "https://www.codingame.com/services/Leaderboards/getFilteredChallengeLeaderboard"
FILTER = {"active": False, "keyword": "", "column": "", "filter": ""}
PAYLOAD = ["spring-challenge-2026-troll-farm", "", "global", FILTER]


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


def print_player_info(p: dict):
    ca = p["codingamer"]
    print(f"  Pseudo:        {p['pseudo']}")
    print(f"  User ID:       {ca['userId']}")
    print(f"  Public handle: {ca['publicHandle']}")
    print(f"  Country:       {ca.get('countryId', '?')}")
    print(f"  Rank:          #{p['rank']} global  #{p['localRank']} league")
    print(f"  Score:         {p['score']:.2f}")
    print(f"  Language:      {p['programmingLanguage']}")
    print(f"  League:        division {p['league']['divisionIndex']+1}/{p['league']['divisionCount']}")
    print(f"  Agent ID:      {p['agentId']}")
    print(f"  Test session:  {p['testSessionHandle']}")
    print(f"  Tests passed:  {p.get('percentage', '?')}%")
    print()
    print("  NOTE: No public API exists to list all games for a player.")
    print("  However, you can fetch individual game replays if you have a game ID:")
    print("    https://www.codingame.com/replay/<game_id>")
    print(f"  Or use the API:")
    print(f'    curl -X POST https://www.codingame.com/services/gameResult/findByGameId \\')
    print(f'      -H "Content-Type: application/json" \\')
    print(f'      -d \'[<game_id>, null]\'')


def list_top(data: dict, n: int = 20):
    print(f"{'Rank':>5} {'Pseudo':<20} {'Score':>7} {'Lang':<10} {'Country':<6}")
    print("-" * 55)
    for u in data.get("users", [])[:n]:
        ca = u["codingamer"]
        score = f"{u['score']:.2f}"
        lang = u.get("programmingLanguage", "?")
        country = ca.get("countryId", "") or ""
        print(f"{u['rank']:>5} {u['pseudo'][:20]:<20} {score:>7} {lang[:10]:<10} {country:<6}")


def main():
    if not sys.argv[1:]:
        print(__doc__.strip())
        return 1

    print("Fetching leaderboard (2022 players)...", file=sys.stderr)
    data = fetch_leaderboard()
    print(f"Done. {data['count']} players.", file=sys.stderr)
    print()

    if sys.argv[1] == "--list-top":
        n = int(sys.argv[2]) if len(sys.argv) > 2 else 20
        list_top(data, n)
        return 0

    query = sys.argv[1]
    player = find_player(data, query)
    if player:
        print_player_info(player)
    else:
        print(f"Player '{query}' not found on leaderboard.")
        # Show some names as hint
        names = [u["pseudo"] for u in data["users"][:10]]
        print(f"Top 10: {', '.join(names)}")
        return 1


if __name__ == "__main__":
    main()
