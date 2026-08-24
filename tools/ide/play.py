"""Play one game with the current bot in the CodinGame IDE and report it.

Injects bots/<game>/src/main.rs.flattened into the editor (verified), clicks
PLAY MY CODE, and fetches the result. This runs the IDE test match; it does
not submit to the arena. `--seed` pins the map (OPTIONS -> Manual); `--submit`
runs `just submit` in the bot crate first. On a new puzzle, set the editor
language to Rust once by hand — CodinGame remembers it per puzzle.
Equivalent: `just ide [seed]` from the bot dir.

Usage:
  uv run --project tools tools/ide/play.py --game trollfarm
  uv run --project tools tools/ide/play.py --game trollfarm --seed 12345 --submit
"""

import argparse
import sys
import time

import _browser as B


def play_game(driver, args, code: str, seed=None) -> str | None:
    """Inject `code` into the editor (verified), play, return the new game id.

    The IDE must be ready (logged in, Monaco present). Raises SystemExit when
    the editor content cannot be verified; playing unverified code produces
    empty-editor games (score -2).
    """
    B.set_editor_code(driver, code)
    time.sleep(1.0)

    n = B.editor_text_length(driver)
    if n < 0:
        # monaco not exposed — verify via the rendered first line instead
        driver.switch_to.default_content()
        top = driver.execute_script(
            "const e=document.querySelector('.view-lines');return e?e.innerText.slice(0,80):'';"
        )
        print(f"editor top: {top.strip()[:80]!r}")
        if "flattened" not in top:
            raise SystemExit("Editor does NOT contain the bot — aborting.")
    else:
        print(f"editor model length: {n} (expected ~{len(code)})")
        if abs(n - len(code)) > 100:
            raise SystemExit("Editor content length mismatch — aborting.")

    print(f"Game options: {B.set_game_mode(driver, seed)}")
    before = B.current_game_id(driver)
    print(f"Clicking Play (previous game id: {before}) ...")
    B.click_play(driver)

    print("Waiting for the match to finish ...")
    return B.wait_for_new_game(driver, before, args.timeout)


def main() -> int:
    B.line_buffer_stdout()
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--seed", default=None, help="pin the map (OPTIONS→Manual)")
    ap.add_argument(
        "--submit",
        action="store_true",
        help="run `just submit` in the bot dir first (rebuild + reflatten)",
    )
    B.add_browser_args(ap)
    args = ap.parse_args()

    if args.submit:
        B.run_submit(args)
    code = B.flattened_code(args)
    print(
        f"Loaded flattened bot: {len(code)} chars, first line {code.splitlines()[0]!r}"
    )

    driver = B.get_driver(args)
    driver.get(B.puzzle_url(args))  # fresh load — clears any hung editor state
    if not B.wait_for_ide(driver, args.timeout):
        print("IDE/login not ready in time.", file=sys.stderr)
        return 2
    time.sleep(3)

    gid = play_game(driver, args, code, args.seed)
    if not gid:
        print("No new game id appeared.", file=sys.stderr)
        return 1

    print(f"New game id: {gid} — fetching result ...")
    return B.report(
        gid, B.fetch_result(driver, gid, args.user_id), expected_seed=args.seed
    )


if __name__ == "__main__":
    raise SystemExit(main())
