#!/usr/bin/env python3
"""THE standard bench: replay the 10 pinned arena seeds in the IDE against
the REAL opponents (eval/arena_bench_seeds.json: 5 lost + 5 won games).

For each entry it sets the actual arena opponent (PLAYERS tab → delete agent
→ add-player → search + Add), pins the seed (OPTIONS → Manual), clicks PLAY
MY CODE and records the result. The current bot is pasted once at the start
(same clipboard-free injection + head/tail verification as
play_no_clipboard.py). Games are grouped by opponent to minimize agent
switching.

Caveats: opponents may have updated their arena bot since the original game,
and their bots may be timing-nondeterministic — same-seed results are
realistic, not perfectly reproducible.

Usage:
  .venv/bin/python scripts/ide/bench_ide.py                # submit+paste, all 10
  .venv/bin/python scripts/ide/bench_ide.py --no-paste     # code already in editor
  .venv/bin/python scripts/ide/bench_ide.py --skip 4       # resume after 4 games
"""
import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import _common as C
import play_my_code as pmc
import play_no_clipboard as pnc

SEEDS_FILE = C.EVAL_DIR / "arena_bench_seeds.json"


def retry(fn, tries=3, wait=2.0, label=""):
    """Brave's CDP link sporadically drops ('Promise was collected'); retry."""
    for i in range(tries):
        try:
            return fn()
        except Exception as e:  # noqa: BLE001
            if i == tries - 1:
                raise
            print(f"  retry {label or fn}: {str(e).splitlines()[0][:80]}")
            time.sleep(wait)
    return None


def find_ide_tab(driver) -> None:
    for h in driver.window_handles:
        driver.switch_to.window(h)
        if "ide/puzzle" in (driver.current_url or ""):
            return
    raise RuntimeError("no IDE tab open")


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
        # Re-verify the editor before EVERY game: stray keystrokes into the
        # focused window mid-bench once appended junk ("ewaas") and voided 6
        # games with compile errors (score -2). Re-paste on mismatch.
        try:
            pnc.verify_editor_tail(driver, code)
        except Exception as e:  # noqa: BLE001
            print(f"  editor corrupted ({str(e)[:80]}) — re-pasting")
            pnc.set_via_browser_clipboard(driver, code)
    print(f"  options: {pmc.set_game_mode(driver, seed)}")
    before = pmc.current_game_id(driver)
    pmc.click_play(driver)
    deadline = time.time() + timeout
    gid = None
    while time.time() < deadline:
        gid = retry(lambda: pmc.current_game_id(driver), label="game id")
        if gid and gid != before:
            break
        time.sleep(2)
    if not gid or gid == before:
        return None, None
    res = retry(lambda: pmc.fetch_result(driver, gid, user_id), label="result")
    return gid, res


def main() -> int:
    sys.stdout.reconfigure(line_buffering=True)
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--no-submit", action="store_true", help="skip just submit")
    ap.add_argument("--no-paste", action="store_true", help="don't touch the editor")
    ap.add_argument("--skip", type=int, default=0, help="skip the first N games (resume)")
    ap.add_argument("--limit", type=int, default=0, help="play at most N games (0 = all)")
    ap.add_argument("--user-id", type=int, default=pmc.MY_USER_ID)
    ap.add_argument("--debug-port", type=int, default=pmc.DEBUG_PORT)
    ap.add_argument("--profile", default=str(C.TROLLFARM_DIR / ".cg-browser-profile"))
    ap.add_argument("--browser-binary", default=pmc.BRAVE)
    args = ap.parse_args()

    cfg = json.loads(SEEDS_FILE.read_text())
    games = [dict(e, tag="LOST") for e in cfg["lost"]] + [dict(e, tag="WON") for e in cfg["won"]]
    games.sort(key=lambda e: e["opp"])  # group by opponent → fewer agent switches

    if not args.no_submit:
        subprocess.run(["just", "submit"], cwd=C.TROLLFARM_DIR, capture_output=True)
    driver = pmc.get_driver(args)
    find_ide_tab(driver)
    if not pmc.wait_for_ide(driver, 120):
        print("IDE/login not ready", file=sys.stderr)
        return 2

    code = (C.TROLLFARM_DIR / "src" / "main.rs.flattened").read_text()
    if not args.no_paste:
        print(f"pasting bot ({len(code)} chars, {code.splitlines()[0]!r})")
        retry(lambda: pnc.set_via_browser_clipboard(driver, code), tries=4, label="paste")

    rows = []
    for i, e in enumerate(games):
        if i < args.skip:
            continue
        if args.limit and len(rows) >= args.limit:
            break
        print(f"[{i + 1}/{len(games)}] {e['tag']} vs {e['opp']} (arena {e['result']}) seed={e['seed']}")
        retry(lambda: set_opponent(driver, e["opp"]), label="set opponent")
        gid, res = play_seed(driver, e["seed"], args.user_id, code=code)
        if not gid or not res or res.get("err") or not res.get("scores"):
            print("  !! no result, aborting (resume with --skip", i, ")")
            break
        my, opp = res["scores"][0], res["scores"][1]
        outcome = "WIN " if my > opp else ("LOSS" if my < opp else "DRAW")
        print(f"  -> {gid}: {my}-{opp}  {outcome}")
        rows.append((e, gid, my, opp, outcome))

    print("\n══════════ IDE ARENA BENCH ══════════")
    w = l = 0
    for e, gid, my, opp, outcome in rows:
        w += outcome == "WIN "
        l += outcome == "LOSS"
        print(f"{e['tag']:<4} vs {e['opp']:<10} arena {e['result']:<9} now {my}-{opp}  {outcome}  ({gid})")
    print(f"total: {w}W {l}L of {len(rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
