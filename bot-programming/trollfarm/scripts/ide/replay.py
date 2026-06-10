#!/usr/bin/env python3
"""Re-run a fixed-seed game in the IDE with the freshly-compiled bot.

For iterating on ONE specific map: pass `--seed` to pin OPTIONS → Manual to that
seed, then it pastes the current bot and clicks PLAY MY CODE, so every run is the
same map with the latest code. Reuses play_my_code.py's primitives.

Seed mechanics (CodinGame IDE):
  * PLAY MY CODE on **Automatic** draws a NEW seed; on **Manual** it sticks to
    the configured seed. "Replay in same conditions" reuses the LAST game's seed
    (even on Automatic). Both recompile the editor — so `--seed` + PLAY MY CODE
    is the reliable way to pin an exact map regardless of the last game.
Paste uses `just submit`'s clipboard + Ctrl+V (play_my_code.paste_code). The
editor's **vim mode MUST be OFF** — it intercepts Ctrl+A/Ctrl+V and silently
leaves the editor empty/stale.

Usage:
  python3 scripts/ide/replay.py --seed=-773904653721004000             # pin map: submit+paste+play
  python3 scripts/ide/replay.py --seed=-773904653721004000 --no-submit # reuse last flatten
"""
import argparse
import sys
import time
from pathlib import Path

from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import play_my_code as pmc

IDE_MARK = "ide/puzzle/spring-challenge-2026-troll-farm"


def editor_top(driver) -> str:
    """The currently-rendered top of the Monaco editor (virtualized view)."""
    driver.switch_to.default_content()
    return driver.execute_script(
        "return (document.querySelector('.view-lines')||{innerText:''}).innerText;"
    )


def set_vim_off(driver) -> None:
    """Force the editor out of vim mode (Ctrl+V means 'visual block' in vim, so
    the clipboard paste is swallowed). The mode is persisted here."""
    driver.switch_to.default_content()
    driver.execute_script(
        "localStorage.setItem('ngStorage-userProperty-codeEditorConfig',"
        " JSON.stringify({mode:'normal'}));"
    )


def paste_and_verify(driver, tries=6) -> bool:
    """Ctrl+A + Ctrl+V, then confirm our code actually landed (the flattened
    first line is `// flattened ...`). Retries because a fresh IDE can reload the
    default template over the paste, and the first Ctrl+V can miss focus."""
    for i in range(1, tries + 1):
        driver.switch_to.default_content()
        driver.find_element(By.CSS_SELECTOR, ".monaco-editor").click()
        time.sleep(0.3)
        ActionChains(driver).key_down(Keys.CONTROL).send_keys("a").key_up(
            Keys.CONTROL
        ).perform()
        time.sleep(0.2)
        ActionChains(driver).key_down(Keys.CONTROL).send_keys("v").key_up(
            Keys.CONTROL
        ).perform()
        time.sleep(1.5)
        # Scroll to the top: .view-lines is virtualized and after a paste the view
        # sits at the cursor (end of file), so line 1 (the // flattened stamp)
        # isn't rendered until we jump there.
        ActionChains(driver).key_down(Keys.CONTROL).send_keys(Keys.HOME).key_up(
            Keys.CONTROL
        ).perform()
        time.sleep(0.5)
        top = editor_top(driver)
        if "flattened" in top or "mod bot" in top:
            print(f"  paste OK (attempt {i})")
            return True
        print(f"  paste attempt {i} did not land (top: {top[:45]!r}); retrying")
        time.sleep(1.0)
    return False


def wait_ready(driver, timeout) -> bool:
    """Wait for the Monaco editor, tolerating transient CDP hiccups.

    We attach to an already-open, logged-in IDE, so this only needs the editor
    to be present — and it swallows the occasional `Promise was collected` /
    context-destroyed error that a sync call hits mid-render.
    """
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            if driver.find_elements(By.CSS_SELECTOR, ".monaco-editor"):
                return True
        except Exception:  # noqa: BLE001 — transient CDP error, retry
            pass
        time.sleep(2)
    return False


def fetch_with_retry(driver, gid, user_id, tries=3):
    """fetch_result, retrying the transient `Promise was collected` CDP error."""
    for _ in range(tries):
        try:
            return pmc.fetch_result(driver, gid, user_id)
        except Exception:  # noqa: BLE001
            time.sleep(2)
    return {"err": "fetch failed after retries"}


def main() -> int:
    sys.stdout.reconfigure(line_buffering=True)
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--url", default=pmc.PUZZLE_URL)
    ap.add_argument("--profile", default=str(pmc.REPO / ".cg-browser-profile"))
    ap.add_argument("--browser-binary", default=pmc.BRAVE)
    ap.add_argument("--debug-port", type=int, default=pmc.DEBUG_PORT)
    ap.add_argument("--user-id", type=int, default=pmc.MY_USER_ID)
    ap.add_argument("--no-submit", action="store_true", help="skip `just submit`")
    ap.add_argument("--no-paste", action="store_true", help="don't touch the editor")
    ap.add_argument(
        "--seed",
        default=None,
        help="pin the map: set OPTIONS→Manual to this seed before playing (PLAY MY CODE path)",
    )
    ap.add_argument(
        "--replay",
        action="store_true",
        help="click 'Replay in same conditions' (keeps last game's opponent + seed) "
        "instead of PLAY MY CODE; ignores --seed",
    )
    ap.add_argument("--timeout", type=int, default=300)
    args = ap.parse_args()

    driver = pmc.get_driver(args)
    if IDE_MARK not in (driver.current_url or ""):
        driver.get(args.url)
    if not wait_ready(driver, args.timeout):
        print("Monaco editor not ready in time.", file=sys.stderr)
        return 2

    if not args.no_submit:
        pmc.run_submit()
    if not args.no_paste:
        set_vim_off(driver)
        print("Pasting bot into the editor (clipboard + Ctrl+V, verify+retry) ...")
        if not paste_and_verify(driver):
            # NB: do NOT reload to fix vim — a reload resets OPTIONS + PLAYERS
            # (opponent back to Boss), which breaks 'Replay in same conditions'.
            print(
                "Paste failed — the editor's vim mode is ON (Ctrl+V = visual block). "
                "Turn vim OFF in the IDE and re-run. (Not reloading, to preserve the "
                "configured opponent + seed.)",
                file=sys.stderr,
            )
            return 3

    before = pmc.current_game_id(driver)
    if args.replay:
        # Replay in same conditions: re-run the LAST game's opponent + seed with
        # the freshly-pasted code, without touching OPTIONS/PLAYERS.
        driver.switch_to.default_content()
        btn = driver.find_element(By.CSS_SELECTOR, "button.replay")
        label = (btn.text or "").strip()
        driver.execute_script("arguments[0].click();", btn)
        print(f"Clicking '{label}' (same opponent+seed; previous id: {before}) ...")
    else:
        if args.seed is not None:
            print(f"Game options: {pmc.set_game_mode(driver, args.seed)}")
        # PLAY MY CODE recompiles; Manual seed (--seed) pins the map. Resets the
        # opponent to whatever PLAYERS holds (not the replay button's memory).
        print(f"Clicking '{pmc.click_play(driver)}' (recompiles; previous id: {before}) ...")

    print("Waiting for the match to finish ...")
    deadline = time.time() + args.timeout
    gid = None
    while time.time() < deadline:
        gid = pmc.current_game_id(driver)
        if gid and gid != before:
            break
        time.sleep(2)
    if not gid or gid == before:
        print("No new game id appeared (match may still be computing).", file=sys.stderr)
        return 1

    print(f"New game id: {gid} — fetching result ...")
    return pmc.report(gid, fetch_with_retry(driver, gid, args.user_id), expected_seed=args.seed)


if __name__ == "__main__":
    sys.exit(main())
