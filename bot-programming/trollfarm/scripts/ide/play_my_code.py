#!/usr/bin/env python3
"""Submit the current bot to the CodinGame Troll-Farm IDE and report the result.

Pipeline (all in one logged-in browser session):
  1. `just submit`  — build + flatten + copy the bot to the clipboard (wl-copy).
  2. paste it into the IDE's Monaco editor (Ctrl+A, Ctrl+V).
  3. click "Play my code".
  4. read the new game id from the viewer's /share-replay/<id> link.
  5. fetch that game's result (scores/ranks) via the in-page gameResult API
     (same-origin fetch → uses your session cookies; sandbox replays need your
     user id as the 2nd arg, hence --user-id).

"Play my code" runs the IDE's test match; it does NOT submit to the arena.

Browser: a persistent Brave profile (`.cg-browser-profile`) keeps you logged in.
Log in once in the window that opens; later runs attach to that same window
(via the debug port) so there's no profile-lock dance.

Run:
  .venv-browser/bin/python scripts/play_my_code.py                 # full pipeline, random seed
  .venv-browser/bin/python scripts/play_my_code.py --seed 12345    # replay a fixed map (OPTIONS→Manual)
  .venv-browser/bin/python scripts/play_my_code.py --no-submit     # play what's in the editor
  .venv-browser/bin/python scripts/play_my_code.py --no-paste      # don't touch the editor

--seed pins the map (handy for tuning params on a specific game). It sets
OPTIONS→Manual seed=<n> and clicks PLAY MY CODE — the manual seed is honoured by
Play, NOT by "Replay in same conditions" (which just repeats the prior game).
"""

import argparse
import re
import subprocess
import sys
import time
from pathlib import Path

from selenium import webdriver
from selenium.webdriver.chrome.options import Options as ChromeOptions
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import _common as C

REPO = C.TROLLFARM_DIR  # has the Justfile (`just submit`)
PUZZLE_URL = f"https://www.codingame.com/ide/puzzle/{C.PUZZLE_ID}"
BRAVE = "/usr/bin/brave"
DEBUG_PORT = 9222
MY_USER_ID = 4083906  # tonigineer — needed to authorize your own sandbox replays

_LOWER = "translate(.,'ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz')"


def get_driver(args):
    """Attach to the existing Brave on the debug port, or launch a fresh one."""
    try:
        o = ChromeOptions()
        o.add_experimental_option("debuggerAddress", f"127.0.0.1:{args.debug_port}")
        d = webdriver.Chrome(options=o)
        print(f"Attached to existing Brave on :{args.debug_port}")
        return d
    except Exception:
        pass
    profile = Path(args.profile)
    profile.mkdir(parents=True, exist_ok=True)
    o = ChromeOptions()
    o.binary_location = args.browser_binary
    o.add_argument(f"--user-data-dir={profile}")
    o.add_argument(f"--remote-debugging-port={args.debug_port}")
    o.add_argument("--no-first-run")
    o.add_argument("--no-default-browser-check")
    o.add_argument("--start-maximized")
    o.add_experimental_option("detach", True)
    print(f"Launching {args.browser_binary} (profile {profile})")
    return webdriver.Chrome(options=o)


def dismiss_cookie_banner(driver) -> None:
    for by, sel in [
        (By.ID, "didomi-notice-agree-button"),
        (
            By.XPATH,
            f"//button[contains({_LOWER},'agree') or contains({_LOWER},'accept')]",
        ),
    ]:
        try:
            els = driver.find_elements(by, sel)
            if els and els[0].is_displayed():
                els[0].click()
                return
        except Exception:  # noqa: BLE001
            pass


def is_logged_in(driver) -> bool:
    for xp in [
        "//a[contains(@href,'login') or contains(@href,'signup')]",
        f"//*[self::a or self::button][contains({_LOWER},'log in') or contains({_LOWER},'sign in')]",
    ]:
        for el in driver.find_elements(By.XPATH, xp):
            try:
                if el.is_displayed():
                    return False
            except Exception:  # noqa: BLE001
                continue
    return True


def wait_for_ide(driver, timeout) -> bool:
    """Wait until logged in and the Monaco editor is present."""
    deadline = time.time() + timeout
    warned = False
    while time.time() < deadline:
        dismiss_cookie_banner(driver)
        if is_logged_in(driver) and driver.find_elements(
            By.CSS_SELECTOR, ".monaco-editor"
        ):
            return True
        if not warned and not is_logged_in(driver):
            print(
                f"Not logged in yet — log in to CodinGame in the Brave window "
                f"(waiting up to {timeout}s)."
            )
            warned = True
        time.sleep(2)
    return False


def run_submit() -> None:
    """`just submit` → build + flatten + wl-copy the bot to the clipboard."""
    print("Running `just submit` (build + flatten + copy to clipboard) ...")
    r = subprocess.run(["just", "submit"], cwd=REPO, text=True, capture_output=True)
    tail = (r.stdout + r.stderr).strip().splitlines()[-3:]
    for line in tail:
        print("  " + line)
    if r.returncode != 0:
        raise SystemExit(
            f"`just submit` failed (exit {r.returncode}); see output above."
        )


def paste_code(driver) -> None:
    """Select-all + paste the clipboard into the Monaco editor."""
    driver.switch_to.default_content()
    ed = driver.find_element(By.CSS_SELECTOR, ".monaco-editor")
    ed.click()
    ActionChains(driver).key_down(Keys.CONTROL).send_keys("a").key_up(
        Keys.CONTROL
    ).perform()
    ActionChains(driver).key_down(Keys.CONTROL).send_keys("v").key_up(
        Keys.CONTROL
    ).perform()
    time.sleep(1.5)
    # Soft confirmation: show the top of what's now in the editor.
    ActionChains(driver).key_down(Keys.CONTROL).send_keys(Keys.HOME).key_up(
        Keys.CONTROL
    ).perform()
    time.sleep(0.3)
    top = driver.execute_script(
        "const e=document.querySelector('.view-lines');return e?e.innerText.slice(0,80):'';"
    )
    print(f"  editor now starts with: {top.strip()[:80]!r}")


def current_game_id(driver):
    """Read the /share-replay/<id> game id from inside the viewer iframe(s)."""
    driver.switch_to.default_content()
    gid = None
    for f in driver.find_elements(By.TAG_NAME, "iframe"):
        try:
            driver.switch_to.frame(f)
            hrefs = driver.execute_script(
                "return Array.from(document.querySelectorAll(\"a[href*='replay']\")).map(a=>a.href);"
            )
            for h in hrefs:
                m = re.search(r"replay/(\d+)", h)
                if m:
                    gid = m.group(1)
                    break
        except Exception:  # noqa: BLE001
            pass
        finally:
            driver.switch_to.default_content()
        if gid:
            break
    return gid


def set_game_mode(driver, seed) -> str:
    """OPTIONS tab → Manual with a fixed seed (if given) or Auto (random).

    NB: the manual seed is honoured by *PLAY MY CODE*, not "Replay in same
    conditions" (that one ignores the options panel and repeats the prior game).
    """
    driver.switch_to.default_content()
    for t in driver.find_elements(By.CSS_SELECTOR, ".ide-tab"):
        if t.text.strip().upper() == "OPTIONS":
            driver.execute_script("arguments[0].click();", t)
            break
    time.sleep(0.8)
    label = "Manual" if seed is not None else "Auto"
    for lbl in driver.find_elements(By.XPATH, f"//label[normalize-space()='{label}']"):
        driver.execute_script("arguments[0].click();", lbl)
        break
    time.sleep(0.3)
    if seed is None:
        return "auto (random)"
    ta = driver.find_element(By.CSS_SELECTOR, "textarea.options-text")
    ta.click()
    ta.send_keys(Keys.CONTROL, "a")
    ta.send_keys(Keys.DELETE)
    ta.send_keys(f"seed={seed}")
    # make Angular's ng-model register the new value
    driver.execute_script(
        "arguments[0].dispatchEvent(new Event('input',{bubbles:true}));"
        "arguments[0].dispatchEvent(new Event('change',{bubbles:true}));"
        "arguments[0].blur();",
        ta,
    )
    time.sleep(0.3)
    return f"manual seed={seed}"


def click_play(driver) -> str:
    driver.switch_to.default_content()
    btn = driver.find_element(By.CSS_SELECTOR, "button.play")
    label = (btn.text or "").strip()
    driver.execute_script("arguments[0].click();", btn)
    return label


def fetch_result(driver, game_id, user_id):
    driver.switch_to.default_content()
    driver.set_script_timeout(30)
    return driver.execute_async_script(
        r"""
        const cb = arguments[arguments.length - 1];
        fetch('/services/gameResult/findByGameId', {
          method:'POST', headers:{'Content-Type':'application/json'},
          credentials:'include', body: JSON.stringify([arguments[0], arguments[1]])
        }).then(r=>r.json()).then(j=>cb({
          scores:j.scores, ranks:j.ranks, referee:(j.refereeInput||'').trim(),
          agents:(j.agents||[]).map(a=>({i:a.index, name:(a.codingamer&&a.codingamer.pseudo)||'(boss)', score:a.score})),
          err:j.message||null
        })).catch(e=>cb({err:String(e)}));
    """,
        int(game_id),
        int(user_id),
    )


def report(game_id, res, expected_seed=None) -> int:
    print("\n" + "═" * 48)
    print(f"Game {game_id}  https://www.codingame.com/share-replay/{game_id}")
    if res.get("err"):
        print(f"Could not read result: {res['err']}")
        return 1
    if res.get("referee"):
        print(f"  {res['referee']}")
        if expected_seed is not None and f"seed={expected_seed}" not in res["referee"]:
            print(f"  ⚠ requested seed={expected_seed} but game used a different seed!")
    scores, ranks, agents = res.get("scores"), res.get("ranks"), res.get("agents", [])
    for a in agents:
        i = a["i"]
        _rank = ranks[i] if ranks else "?"
        score = scores[i] if scores else "?"
        tag = "  ← WIN" if ranks and ranks[i] == 0 else ""
        print(f"  {a['name']:<16} score {score}{tag}")
    if ranks and len(ranks) == 2:
        print(
            "Result:",
            "you win"
            if ranks[0] == 0
            else ("draw" if scores and scores[0] == scores[1] else "you lose"),
        )
    return 0


def main() -> int:
    sys.stdout.reconfigure(line_buffering=True)  # stream progress even when piped (tee/redirect)
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--url", default=PUZZLE_URL)
    ap.add_argument("--profile", default=str(REPO / ".cg-browser-profile"))
    ap.add_argument("--browser-binary", default=BRAVE)
    ap.add_argument("--debug-port", type=int, default=DEBUG_PORT)
    ap.add_argument(
        "--user-id",
        type=int,
        default=MY_USER_ID,
        help="your CodinGame user id (authorizes your sandbox replays)",
    )
    ap.add_argument(
        "--no-submit",
        action="store_true",
        help="skip `just submit` (use current clipboard/editor)",
    )
    ap.add_argument(
        "--no-paste",
        action="store_true",
        help="don't paste — play whatever is in the editor",
    )
    ap.add_argument(
        "--seed",
        default=None,
        help="play a fixed seed (OPTIONS→Manual); omit for a random seed. "
        "Lets you replay a specific map while tuning params.",
    )
    ap.add_argument(
        "--timeout",
        type=int,
        default=300,
        help="seconds to wait for login / the result",
    )
    args = ap.parse_args()

    driver = get_driver(args)
    # Navigate to the IDE only if we're not already there (keeps an attached session put).
    if "ide/puzzle/spring-challenge-2026-troll-farm" not in (driver.current_url or ""):
        driver.get(args.url)
    if not wait_for_ide(driver, args.timeout):
        print("IDE/login not ready in time.", file=sys.stderr)
        return 2

    if not args.no_submit:
        run_submit()
    if not args.no_paste:
        print("Pasting bot into the editor ...")
        paste_code(driver)

    print(f"Game options: {set_game_mode(driver, args.seed)}")

    before = current_game_id(driver)
    print(f"Clicking Play (previous game id: {before}) ...")
    click_play(driver)

    print("Waiting for the match to finish ...")
    deadline = time.time() + args.timeout
    gid = None
    while time.time() < deadline:
        gid = current_game_id(driver)
        if gid and gid != before:
            break
        time.sleep(2)
    if not gid or gid == before:
        print(
            "No new game id appeared (match may still be computing). "
            "Re-run, or grab the /share-replay/<id> link manually.",
            file=sys.stderr,
        )
        return 1

    print(f"New game id: {gid} — fetching result ...")
    return report(gid, fetch_result(driver, gid, args.user_id), expected_seed=args.seed)


if __name__ == "__main__":
    sys.exit(main())
