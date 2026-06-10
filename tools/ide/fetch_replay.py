"""Fetch one CodinGame replay (full gameResult JSON) and print a summary.

Two mutually exclusive modes:
  <id|url>  fetch that finished game via the anonymous public API; private
            sandbox replays transparently fall back to the logged-in browser.
  --ide     fetch the game currently in the IDE viewer; if there is none,
            play one first with the current bot (the play.py flow).

The full JSON (frames, stderr, summaries) is saved to
bots/<bot>/replays/replay_<id>.json.

Usage:
  uv run --project tools tools/ide/fetch_replay.py 891040557
  uv run --project tools tools/ide/fetch_replay.py https://www.codingame.com/replay/891040557
  uv run --project tools tools/ide/fetch_replay.py --ide
"""

import argparse
import json
import re
import time

import requests

import _browser as B
import play

API = "https://www.codingame.com/services/gameResult/findByGameId"


def fetch_public(game_id: int) -> dict | None:
    """Fetch via the anonymous public API; None when the replay is private."""
    r = requests.post(API, json=[game_id, None], timeout=30)
    if r.status_code == 422 and (r.json() or {}).get("code") == "UNAUTHORIZED":
        return None
    r.raise_for_status()
    return r.json()


def ensure_session(args):
    """Return a driver on a logged-in codingame.com page, opening the IDE if
    needed. Retried as a whole: attaching and navigating over the CDP link
    fails sporadically and a fresh attempt usually succeeds."""

    def _connect():
        driver = B.get_driver(args)
        try:
            B.find_ide_tab(driver)
        except RuntimeError:
            driver.get(B.puzzle_url(args))
            if not B.wait_for_ide(driver, args.timeout):
                raise SystemExit("IDE/login not ready in time.") from None
            time.sleep(3)  # cold load: let the IDE finish mounting
        return driver

    return B.retry(_connect, tries=3, wait=3.0, label="ide session")


def ide_game_id(driver, args) -> int:
    """Return the game currently in the IDE viewer. If the viewer is empty,
    play one first via the verified play.py flow rather than clicking Play
    on whatever happens to sit in the editor."""
    gid = B.current_game_id(driver)
    if not gid:
        print("No game in the viewer — playing one with the current bot first ...")
        code = B.flattened_code(args)
        print(f"  flattened bot: {len(code)} chars, {code.splitlines()[0]!r}")
        gid = play.play_game(driver, args, code)
        if not gid:
            raise SystemExit("no game id appeared after Play.")
    print(f"Using IDE game {gid}")
    return int(gid)


def parse_game_id(arg: str) -> int:
    m = re.search(r"replay/(\d+)", arg)
    return int(m.group(1)) if m else int(arg)


def referee_info(text: str | None) -> dict:
    return {m.group(1): m.group(2) for m in re.finditer(r"(\w+)=(\S+)", text or "")}


def summarize(data: dict) -> None:
    print("\n=== PLAYERS ===")
    for a in data.get("agents") or []:
        name = (a.get("codingamer") or {}).get("pseudo", "(boss)")
        idx = a.get("index", 0)
        scores = data.get("scores") or []
        score = scores[idx] if idx < len(scores) else "?"
        print(f"  Player {idx + 1}: {name} (score {score})")

    info = referee_info(data.get("refereeInput"))
    if info:
        print("\n=== REFEREE ===")
        for k, v in info.items():
            print(f"  {k}: {v}")

    frames = data.get("frames") or []
    print("\n=== FRAMES ===")
    print(f"  Total: {len(frames)}")
    if frames:
        print(f"  Last frame summary: {(frames[-1].get('summary') or '')[:200]}")

    print("\n=== SCORES ===")
    print(f"  {data.get('scores')}")


def main() -> int:
    B.line_buffer_stdout()
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "game",
        nargs="?",
        help="replay URL or game id of a finished game",
    )
    ap.add_argument(
        "--ide",
        action="store_true",
        help="operate the IDE instead: current viewer game, or play one",
    )
    B.add_browser_args(ap)
    args = ap.parse_args()

    if args.ide == bool(args.game):
        ap.error("pass either a game id/URL (finished game) or --ide, not both")

    if args.ide:
        driver = ensure_session(args)
        game_id = ide_game_id(driver, args)
        data = B.fetch_result_full(driver, game_id, args.user_id)
    else:
        game_id = parse_game_id(args.game)
        print(f"Fetching replay {game_id} (public API) ...")
        data = fetch_public(game_id)
        if data is None:
            print("Not public — fetching through the logged-in browser ...")
            driver = ensure_session(args)
            data = B.fetch_result_full(driver, game_id, args.user_id)
    if data.get("err") or data.get("message"):
        print(f"Could not fetch: {data.get('err') or data.get('message')}")
        return 1

    summarize(data)

    out_dir = B.bot_dir(args.bot) / "replays"
    out_dir.mkdir(parents=True, exist_ok=True)
    out = out_dir / f"replay_{game_id}.json"
    out.write_text(json.dumps(data, indent=1))
    print(f"\nFull replay saved to {out} ({out.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
