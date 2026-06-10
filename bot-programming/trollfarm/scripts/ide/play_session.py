#!/usr/bin/env python3
"""Play N games in a row vs the boss in the CodinGame IDE and summarise W-L-D.

A thin loop on top of the single-game primitives in ``play_my_code.py`` (driver
attach, submit, paste, play, result fetch) — that script stays a clean one-shot;
this one drives a whole session. The code is submitted/pasted once (the editor
keeps it between games); each game is a fresh RANDOM map via "Play my code".

Usage:
  python3 scripts/ide/play_session.py --games 10                       # submit+paste once, play 10
  python3 scripts/ide/play_session.py --games 10 --no-submit --no-paste  # code already in the editor
"""
import argparse
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import play_my_code as pmc

IDE_MARK = "ide/puzzle/spring-challenge-2026-troll-farm"


def summarize(results) -> None:
    """Aggregate a session into a W-L-D table (scored by margin)."""
    rows, wins, losses, draws, my_tot, opp_tot = [], 0, 0, 0, 0, 0
    for gid, res in results:
        scores = res.get("scores")
        if not scores or len(scores) < 2:
            continue
        my, opp = scores[0], scores[1]
        my_tot += my
        opp_tot += opp
        if my > opp:
            outcome, wins = "WIN ", wins + 1
        elif my < opp:
            outcome, losses = "LOSS", losses + 1
        else:
            outcome, draws = "DRAW", draws + 1
        rows.append((gid, my, opp, outcome))

    n = len(rows) or 1
    print("\n" + "═" * 52)
    print(f"SUMMARY — {len(rows)} games vs boss")
    for gid, my, opp, oc in rows:
        print(f"  {oc}  {my:>3}-{opp:<3} ({my - opp:+4d})  {gid}")
    print("─" * 52)
    print(f"  W-L-D: {wins}-{losses}-{draws}   win rate {100 * wins / n:.0f}%")
    print(f"  avg {my_tot / n:.1f} - {opp_tot / n:.1f}   margin {(my_tot - opp_tot) / n:+.1f}")


def play_one(driver, before, timeout, user_id):
    """Click Play, wait for a new game id, fetch its result. Returns (gid, res)."""
    pmc.click_play(driver)
    deadline = time.time() + timeout
    while time.time() < deadline:
        gid = pmc.current_game_id(driver)
        if gid and gid != before:
            return gid, pmc.fetch_result(driver, gid, user_id)
        time.sleep(2)
    return None, None


def main() -> int:
    sys.stdout.reconfigure(line_buffering=True)
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--games", type=int, default=10, help="games to play (default 10)")
    ap.add_argument("--url", default=pmc.PUZZLE_URL)
    ap.add_argument("--profile", default=str(pmc.REPO / ".cg-browser-profile"))
    ap.add_argument("--browser-binary", default=pmc.BRAVE)
    ap.add_argument("--debug-port", type=int, default=pmc.DEBUG_PORT)
    ap.add_argument("--user-id", type=int, default=pmc.MY_USER_ID)
    ap.add_argument("--no-submit", action="store_true", help="skip `just submit`")
    ap.add_argument("--no-paste", action="store_true", help="don't touch the editor")
    ap.add_argument("--timeout", type=int, default=300, help="per-game seconds to wait")
    args = ap.parse_args()

    driver = pmc.get_driver(args)
    if IDE_MARK not in (driver.current_url or ""):
        driver.get(args.url)
    if not pmc.wait_for_ide(driver, args.timeout):
        print("IDE/login not ready in time.", file=sys.stderr)
        return 2

    if not args.no_submit:
        pmc.run_submit()
    if not args.no_paste:
        print("Pasting bot into the editor ...")
        pmc.paste_code(driver)

    print(f"Game options: {pmc.set_game_mode(driver, None)}")  # None -> random maps

    results = []
    before = pmc.current_game_id(driver)
    for g in range(1, args.games + 1):
        print(f"\n──── Game {g}/{args.games} ──── (prev id: {before})")
        gid, res = play_one(driver, before, args.timeout, args.user_id)
        if not gid:
            print("  No new game id appeared; skipping.", file=sys.stderr)
            continue
        pmc.report(gid, res)
        results.append((gid, res))
        before = gid

    summarize(results)
    return 0


if __name__ == "__main__":
    sys.exit(main())
