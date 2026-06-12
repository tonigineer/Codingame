"""Replay the pinned arena seeds in the IDE against the real arena opponents.

For each entry in the seeds file (`--seeds-file`, default
bots/<game>/eval/arena_bench_seeds.json: {"lost": [...], "won": [...]} with
`opp`, `seed`, and `result` per entry) it selects the opponent in the PLAYERS
tab, pins the seed, plays, and records the result. The bot is injected once
at the start; the editor tail is re-verified before every game and re-pasted
on mismatch. Results are realistic rather than exactly reproducible —
opponents may have updated their bots since the original games.

Usage:
  uv run --project tools tools/ide/bench.py --game trollfarm             # all games
  uv run --project tools tools/ide/bench.py --game trollfarm --no-paste  # code already in the editor
  uv run --project tools tools/ide/bench.py --game trollfarm --skip 4    # resume after 4 games
  uv run --project tools tools/ide/bench.py --game trollfarm --limit 2   # quick spot check
"""

import argparse
import json
import sys
import time
from pathlib import Path

from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys

import _browser as B


def open_players_tab(driver) -> None:
    for t in driver.find_elements(By.CSS_SELECTOR, ".ide-tab"):
        if t.text.strip().upper() == "PLAYERS":
            driver.execute_script("arguments[0].click();", t)
            time.sleep(0.8)
            return
    raise RuntimeError("PLAYERS tab not found")


def opponent_nicknames(driver) -> list:
    return driver.execute_script(
        "return Array.from(document.querySelectorAll('.scroll-panel .agent .nickname'))"
        ".map(n => n.textContent.trim());"
    )


def set_opponent(driver, name: str) -> None:
    """Make `name` the player-2 agent (delete current, search, Add)."""
    open_players_tab(driver)
    if name in opponent_nicknames(driver):
        return
    # delete whatever non-me agent occupies slot 2 (boss or a player)
    driver.execute_script(
        "const b = document.querySelector('.scroll-panel .agent:not(.me) .delete-button');"
        "if (b) b.click();"
    )
    time.sleep(1.0)
    btns = driver.find_elements(By.CSS_SELECTOR, ".add-player")
    if not btns:
        raise RuntimeError("no empty agent slot to fill")
    driver.execute_script("arguments[0].click();", btns[0])
    time.sleep(1.2)
    driver.execute_script(
        "document.querySelector('.cg-popup .popup-container input.field').focus();"
    )
    el = driver.switch_to.active_element
    el.send_keys(name)
    time.sleep(1.5)
    el.send_keys(Keys.ENTER)
    time.sleep(2.0)
    r = driver.execute_script(
        """
        const cards = Array.from(document.querySelectorAll('.cg-popup .popup-container .player-add-card'));
        const c = cards.find(x => (x.textContent||'').includes(arguments[0]));
        if (!c) return 'card not found';
        const b = c.querySelector('button');
        if (!b) return 'no Add button';
        b.click(); return 'ok';
        """,
        name,
    )
    if r != "ok":
        raise RuntimeError(f"could not add opponent {name}: {r}")
    time.sleep(1.5)
    if name not in opponent_nicknames(driver):
        raise RuntimeError(f"agent panel does not show {name} after Add")


def play_seed(driver, seed: str, user_id: int, timeout: int = 300, code: str = ""):
    if code:
        # Re-verify the editor before every game: stray keystrokes into the
        # focused window mid-bench can append junk and void games with
        # compile errors (score -2). Re-paste on mismatch.
        try:
            B.verify_editor_tail(driver, code)
        except Exception as e:  # noqa: BLE001
            print(f"  editor corrupted ({str(e)[:80]}) — re-pasting")
            B.set_via_browser_clipboard(driver, code)
    print(f"  options: {B.set_game_mode(driver, seed)}")
    before = B.current_game_id(driver)
    B.click_play(driver)
    gid = B.wait_for_new_game(driver, before, timeout)
    if not gid:
        return None, None
    res = B.retry(lambda: B.fetch_result(driver, gid, user_id), label="result")
    return gid, res


def main() -> int:
    B.line_buffer_stdout()
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--no-submit", action="store_true", help="skip just submit")
    ap.add_argument("--no-paste", action="store_true", help="don't touch the editor")
    ap.add_argument(
        "--seeds-file",
        default=None,
        help="pinned-seeds JSON (default: bots/<game>/eval/arena_bench_seeds.json)",
    )
    ap.add_argument(
        "--skip", type=int, default=0, help="skip the first N games (resume)"
    )
    ap.add_argument(
        "--limit", type=int, default=0, help="play at most N games (0 = all)"
    )
    B.add_browser_args(ap)
    args = ap.parse_args()

    seeds_file = Path(
        args.seeds_file or B.bot_dir(args.game) / "eval" / "arena_bench_seeds.json"
    )
    cfg = json.loads(seeds_file.read_text())
    games = [dict(e, tag="LOST") for e in cfg["lost"]] + [
        dict(e, tag="WON") for e in cfg["won"]
    ]
    games.sort(key=lambda e: e["opp"])  # group by opponent → fewer agent switches

    if not args.no_submit:
        B.run_submit(args)
    driver = B.get_driver(args)
    B.find_ide_tab(driver)
    if not B.wait_for_ide(driver, 120):
        print("IDE/login not ready", file=sys.stderr)
        return 2

    code = B.flattened_code(args)
    if not args.no_paste:
        print(f"pasting bot ({len(code)} chars, {code.splitlines()[0]!r})")
        B.retry(
            lambda: B.set_via_browser_clipboard(driver, code), tries=4, label="paste"
        )

    rows = []
    for i, e in enumerate(games):
        if i < args.skip:
            continue
        if args.limit and len(rows) >= args.limit:
            break
        print(
            f"[{i + 1}/{len(games)}] {e['tag']} vs {e['opp']} (arena {e['result']}) seed={e['seed']}"
        )
        B.retry(lambda: set_opponent(driver, e["opp"]), label="set opponent")
        gid, res = play_seed(driver, e["seed"], args.user_id, code=code)
        if not gid or not res or res.get("err") or not res.get("scores"):
            print("  !! no result, aborting (resume with --skip", i, ")")
            break
        my, opp = res["scores"][0], res["scores"][1]
        outcome = "WIN " if my > opp else ("LOSS" if my < opp else "DRAW")
        print(f"  -> {gid}: {my}-{opp}  {outcome}")
        rows.append((e, gid, my, opp, outcome))

    print("\n══════════ IDE ARENA BENCH ══════════")
    wins = losses = 0
    for e, gid, my, opp, outcome in rows:
        wins += outcome == "WIN "
        losses += outcome == "LOSS"
        print(
            f"{e['tag']:<4} vs {e['opp']:<10} arena {e['result']:<9} now {my}-{opp}  {outcome}  ({gid})"
        )
    print(f"total: {wins}W {losses}L of {len(rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
