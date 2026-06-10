"""Shared library for the ide/ tools (no CLI).

Provides the path anchors, the puzzle-id convention, and the browser
primitives: attach to (or launch) a persistent logged-in Brave profile over
the Chrome debug port, wait for the IDE, inject code into the Monaco editor
with verification, set game options, play, and fetch results via the
same-origin gameResult API. Log in once in the window that opens; the login
persists in the profile across runs and puzzles.
"""

import io
import re
import socket
import subprocess
import sys
import time
from pathlib import Path

from selenium import webdriver
from selenium.webdriver.chrome.options import Options as ChromeOptions
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys

# --- path anchors / conventions ----------------------------------------------

TOOLS_DIR = Path(__file__).resolve().parents[1]  # .../tools
WORKSPACE_DIR = TOOLS_DIR.parent  # cargo workspace root
BOTS_DIR = WORKSPACE_DIR / "bots"  # one crate per arena bot
BROWSER_PROFILE = TOOLS_DIR / ".cg-browser-profile"  # logged-in browser profile

BRAVE = "/usr/bin/brave"
DEBUG_PORT = 9222
MY_USER_ID = 4083906  # tonigineer — authorizes access to own sandbox replays

# Convention: bot crates are named after the puzzle's URL slug (the <id> in
# codingame.com/multiplayer/bot-programming/<id>), so the crate name itself is
# the puzzle id. This map holds the exceptions; --puzzle overrides both.
PUZZLE_IDS = {
    "trollfarm": "spring-challenge-2026-troll-farm",
    "ultimate-tic-tac-toe": "tic-tac-toe",
}


def bot_dir(name: str) -> Path:
    """Crate directory of an arena bot (bots/<name>)."""
    return BOTS_DIR / name


def puzzle_id(bot: str, override: str | None = None) -> str:
    """Resolve a bot name to its puzzle id: override, exception map, or the name."""
    return override or PUZZLE_IDS.get(bot, bot)


def line_buffer_stdout() -> None:
    """Line-buffer stdout so progress streams when piped (tee/redirect).

    The isinstance check narrows for the type checker (typeshed declares
    sys.stdout as TextIO, without `reconfigure`) and skips correctly when
    stdout has been replaced.
    """
    if isinstance(sys.stdout, io.TextIOWrapper):
        sys.stdout.reconfigure(line_buffering=True)


_LOWER = "translate(.,'ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz')"


def add_browser_args(ap, bot_default: str = "trollfarm") -> None:
    """Add the argparse options shared by every ide/ CLI."""
    ap.add_argument("--bot", default=bot_default, help="bot crate in bots/")
    ap.add_argument(
        "--puzzle", default=None, help="CodinGame puzzle id (default: from --bot)"
    )
    ap.add_argument("--profile", default=str(BROWSER_PROFILE))
    ap.add_argument("--browser-binary", default=BRAVE)
    ap.add_argument("--debug-port", type=int, default=DEBUG_PORT)
    ap.add_argument(
        "--user-id",
        type=int,
        default=MY_USER_ID,
        help="your CodinGame user id (authorizes your sandbox replays)",
    )
    ap.add_argument(
        "--timeout", type=int, default=300, help="seconds to wait for login/result"
    )


def puzzle_url(args) -> str:
    return f"https://www.codingame.com/ide/puzzle/{puzzle_id(args.bot, args.puzzle)}"


def flattened_code(args) -> str:
    """Return the bot's single-file submission (built by `just submit`)."""
    return (bot_dir(args.bot) / "src" / "main.rs.flattened").read_text()


def run_submit(args) -> None:
    """Run `just submit` in the bot's crate dir (build + flatten)."""
    bot = bot_dir(args.bot)
    print(f"Running `just submit` in {bot} ...")
    r = subprocess.run(["just", "submit"], cwd=bot, text=True, capture_output=True)
    for line in (r.stdout + r.stderr).strip().splitlines()[-3:]:
        print("  " + line)
    if r.returncode != 0:
        raise SystemExit(f"`just submit` failed (exit {r.returncode}); see above.")


def retry(fn, tries=3, wait=2.0, label=""):
    """Call `fn` with retries; Brave's CDP link drops sporadically
    ('Promise was collected') and usually recovers on the next attempt."""
    for i in range(tries):
        try:
            return fn()
        except Exception as e:  # noqa: BLE001
            if i == tries - 1:
                raise
            print(f"  retry {label or fn}: {str(e).splitlines()[0][:80]}")
            time.sleep(wait)
    return None


# --- driver / session -------------------------------------------------------


def _port_open(port: int) -> bool:
    with socket.socket() as s:
        s.settimeout(1.0)
        return s.connect_ex(("127.0.0.1", port)) == 0


def get_driver(args):
    """Attach to the existing Brave on the debug port, or launch a fresh one."""
    # Probe the port first: chromedriver HANGS (no timeout) on a debuggerAddress
    # nobody listens on, so only attempt the attach if something is there.
    if _port_open(args.debug_port):
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
    """Wait until logged in and the Monaco editor is present.

    The poll body is exception-guarded: during page loads the CDP link drops
    sporadically and find_elements can throw — treat that as "not ready yet"
    and keep polling.
    """
    deadline = time.time() + timeout
    warned = False
    while time.time() < deadline:
        try:
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
        except Exception as e:  # noqa: BLE001
            print(f"  ide poll: {str(e).splitlines()[0][:80]} (retrying)")
        time.sleep(2)
    return False


def find_ide_tab(driver) -> None:
    for h in driver.window_handles:
        driver.switch_to.window(h)
        if "ide/puzzle" in (driver.current_url or ""):
            return
    raise RuntimeError("no IDE tab open")


# --- editor injection -------------------------------------------------------


def editor_text_length(driver) -> int:
    driver.switch_to.default_content()
    return driver.execute_script(
        "if (window.monaco && monaco.editor.getModels().length)"
        "  return monaco.editor.getModels()[0].getValueLength();"
        "return -1;"
    )


def set_via_monaco(driver, code: str) -> bool:
    driver.switch_to.default_content()
    return driver.execute_script(
        "if (window.monaco && monaco.editor.getModels().length) {"
        "  monaco.editor.getModels()[0].setValue(arguments[0]); return true; }"
        "return false;",
        code,
    )


def click_editor(driver, timeout: float = 15.0) -> None:
    """Focus the Monaco editor, polling until it is clickable.

    On a cold page load the editor mounts late and can re-mount right after
    first appearing, so a one-shot find_element races (stale element).
    """
    deadline = time.time() + timeout
    while True:
        try:
            driver.switch_to.default_content()
            driver.find_element(By.CSS_SELECTOR, ".monaco-editor").click()
            return
        except Exception:  # noqa: BLE001
            if time.time() > deadline:
                raise
            time.sleep(0.5)


def paste_code(driver) -> None:
    """Select-all + paste the clipboard into the Monaco editor."""
    click_editor(driver)
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


def set_via_browser_clipboard(driver, code: str) -> None:
    """Write the code into the browser's clipboard (navigator.clipboard) and
    paste it — independent of the unreliable OS/Wayland clipboard handoff."""
    driver.execute_cdp_cmd(
        "Browser.grantPermissions",
        {
            "permissions": ["clipboardReadWrite", "clipboardSanitizedWrite"],
            "origin": "https://www.codingame.com",
        },
    )
    driver.set_script_timeout(30)
    res = None
    for attempt in range(15):  # writeText needs OS window focus; wait for it
        driver.execute_cdp_cmd("Page.bringToFront", {})
        driver.execute_script("window.focus();")
        res = driver.execute_async_script(
            "const cb=arguments[arguments.length-1];"
            "navigator.clipboard.writeText(arguments[0]).then(()=>cb('ok')).catch(e=>cb(String(e)));",
            code,
        )
        if res == "ok":
            break
        print(f"  clipboard.writeText attempt {attempt + 1}: {res} (retrying)")
        time.sleep(2)
    if res != "ok":
        raise RuntimeError(f"clipboard.writeText failed: {res}")
    paste_code(driver)
    verify_editor_tail(driver, code)


def verify_editor_tail(driver, code: str) -> None:
    """Verify the end of the editor matches `code`.

    Stray keystrokes into the focused window can append junk after the paste
    (compile error, score -2), which a head check cannot see; Monaco
    virtualizes long files, so jump to the bottom and compare there.
    """
    click_editor(driver)
    ActionChains(driver).key_down(Keys.CONTROL).send_keys(Keys.END).key_up(
        Keys.CONTROL
    ).perform()
    time.sleep(0.5)
    bottom = driver.execute_script(
        "const e=document.querySelector('.view-lines');return e?e.innerText:'';"
    )
    norm = lambda s: re.sub(r"[\s ]+", "", s)  # noqa: E731
    want = norm(code)[-30:]
    got = norm(bottom)
    if want not in got:
        raise RuntimeError(
            f"editor tail mismatch: expected ...{want!r}, editor ends ...{got[-60:]!r}"
        )
    print(f"  editor tail verified (ends with ...{want[-20:]!r})")
    ActionChains(driver).key_down(Keys.CONTROL).send_keys(Keys.HOME).key_up(
        Keys.CONTROL
    ).perform()


def set_editor_code(driver, code: str) -> None:
    """Set the editor content: monaco API if exposed, else browser clipboard."""
    if set_via_monaco(driver, code):
        print("Editor set via monaco API.")
    else:
        print("monaco global not exposed; using browser-clipboard paste ...")
        set_via_browser_clipboard(driver, code)


# --- game control / results -------------------------------------------------


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
    """Set the OPTIONS tab to Manual with a fixed seed (if given) or Auto.

    The manual seed is honoured by PLAY MY CODE, not by "Replay in same
    conditions" (which ignores the options panel and repeats the prior game).
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


def wait_for_new_game(driver, before, timeout):
    """Poll until a game id different from `before` shows up (or time out)."""
    deadline = time.time() + timeout
    gid = None
    while time.time() < deadline:
        gid = retry(lambda: current_game_id(driver), label="game id")
        if gid and gid != before:
            return gid
        time.sleep(2)
    return None


def fetch_result(driver, game_id, user_id):
    """Fetch the scores/ranks summary of a game via the gameResult API."""
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


def fetch_result_full(driver, game_id, user_id):
    """Fetch the complete gameResult JSON (frames, stderr, summaries)."""
    driver.switch_to.default_content()
    driver.set_script_timeout(60)
    return driver.execute_async_script(
        r"""
        const cb = arguments[arguments.length - 1];
        fetch('/services/gameResult/findByGameId', {
          method:'POST', headers:{'Content-Type':'application/json'},
          credentials:'include', body: JSON.stringify([arguments[0], arguments[1]])
        }).then(r=>r.json()).then(cb).catch(e=>cb({err:String(e)}));
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
