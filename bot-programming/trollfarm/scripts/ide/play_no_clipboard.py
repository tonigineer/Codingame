#!/usr/bin/env python3
"""One-off: play the current flattened bot in the IDE WITHOUT the clipboard.

The Ctrl+V paste in play_my_code.py can silently leave the editor empty (vim
mode / clipboard handoff). This injects src/main.rs.flattened directly via CDP
Input.insertText after a select-all, verifies the editor content, then plays.
"""
import argparse
import re
import sys
import time
from pathlib import Path

from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import _common as C
import play_my_code as pmc


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


def set_via_browser_clipboard(driver, code: str) -> None:
    """Write the code into the BROWSER's clipboard (navigator.clipboard) and
    Ctrl+V it — fast, and independent of the OS/Wayland clipboard handoff."""
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
    pmc.paste_code(driver)
    verify_editor_tail(driver, code)


def verify_editor_tail(driver, code: str) -> None:
    """Check the END of the editor matches the code. A stray keystroke into
    the live window between paste and play once appended `llllllllllllll`
    (vim-style cursor keys autorepeating) → compile error, score -2. The
    head check can't see that; Monaco virtualizes, so jump to the bottom."""
    driver.switch_to.default_content()
    ed = driver.find_element(By.CSS_SELECTOR, ".monaco-editor")
    ed.click()
    ActionChains(driver).key_down(Keys.CONTROL).send_keys(Keys.END).key_up(
        Keys.CONTROL
    ).perform()
    time.sleep(0.5)
    bottom = driver.execute_script(
        "const e=document.querySelector('.view-lines');return e?e.innerText:'';"
    )
    norm = lambda s: re.sub(r"[\s ]+", "", s)  # noqa: E731
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


def cdp_select_all(driver) -> None:
    # Ctrl+A as raw CDP key events (modifiers bit 2 = Ctrl).
    for typ in ("rawKeyDown", "keyUp"):
        driver.execute_cdp_cmd(
            "Input.dispatchKeyEvent",
            {
                "type": typ,
                "key": "a",
                "code": "KeyA",
                "modifiers": 2,
                "windowsVirtualKeyCode": 65,
                "nativeVirtualKeyCode": 65,
            },
        )


def set_via_cdp(driver, code: str) -> None:
    # Focus the editor, select-all, then inject text — all via CDP, no clipboard.
    ed = driver.find_element(By.CSS_SELECTOR, ".monaco-editor")
    ed.click()
    time.sleep(0.5)
    for attempt in range(3):
        try:
            cdp_select_all(driver)
            time.sleep(0.3)
            driver.execute_cdp_cmd("Input.insertText", {"text": code})
            return
        except Exception as e:  # noqa: BLE001
            print(f"  insert attempt {attempt + 1} failed: {e}")
            time.sleep(2)
            driver.switch_to.default_content()
            ed = driver.find_element(By.CSS_SELECTOR, ".monaco-editor")
            ed.click()
    raise RuntimeError("could not inject code via CDP after 3 attempts")


def main() -> int:
    sys.stdout.reconfigure(line_buffering=True)
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", default=None)
    ap.add_argument("--timeout", type=int, default=300)
    ap.add_argument("--user-id", type=int, default=pmc.MY_USER_ID)
    ap.add_argument("--debug-port", type=int, default=pmc.DEBUG_PORT)
    ap.add_argument("--profile", default=str(C.TROLLFARM_DIR / ".cg-browser-profile"))
    ap.add_argument("--browser-binary", default=pmc.BRAVE)
    args = ap.parse_args()

    code = (C.TROLLFARM_DIR / "src" / "main.rs.flattened").read_text()
    print(f"Loaded flattened bot: {len(code)} chars, first line {code.splitlines()[0]!r}")

    driver = pmc.get_driver(args)
    driver.get(pmc.PUZZLE_URL)  # fresh load — clears any hung editor state
    if not pmc.wait_for_ide(driver, args.timeout):
        print("IDE/login not ready in time.", file=sys.stderr)
        return 2
    time.sleep(3)

    if set_via_monaco(driver, code):
        print("Editor set via monaco API.")
    else:
        print("monaco global not exposed; using browser-clipboard paste ...")
        set_via_browser_clipboard(driver, code)
    time.sleep(1.0)

    n = editor_text_length(driver)
    if n < 0:
        # monaco not exposed — verify via the rendered first line instead
        driver.switch_to.default_content()
        top = driver.execute_script(
            "const e=document.querySelector('.view-lines');return e?e.innerText.slice(0,80):'';"
        )
        print(f"editor top: {top.strip()[:80]!r}")
        if "flattened" not in top:
            print("Editor does NOT contain the bot — aborting.", file=sys.stderr)
            return 3
    else:
        print(f"editor model length: {n} (expected ~{len(code)})")
        if abs(n - len(code)) > 100:
            print("Editor content length mismatch — aborting.", file=sys.stderr)
            return 3

    print(f"Game options: {pmc.set_game_mode(driver, args.seed)}")
    before = pmc.current_game_id(driver)
    print(f"Clicking Play (previous game id: {before}) ...")
    pmc.click_play(driver)

    print("Waiting for the match to finish ...")
    deadline = time.time() + args.timeout
    gid = None
    while time.time() < deadline:
        gid = pmc.current_game_id(driver)
        if gid and gid != before:
            break
        time.sleep(2)
    if not gid or gid == before:
        print("No new game id appeared.", file=sys.stderr)
        return 1

    print(f"New game id: {gid} — fetching result ...")
    return pmc.report(gid, pmc.fetch_result(driver, gid, args.user_id), expected_seed=args.seed)


if __name__ == "__main__":
    raise SystemExit(main())
