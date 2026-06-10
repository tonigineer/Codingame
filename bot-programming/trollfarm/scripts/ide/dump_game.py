#!/usr/bin/env python3
"""Dump the FULL gameResult JSON (frames, stderr, summaries) for an IDE game.

Attaches to the already-running logged-in Brave (same as play_my_code.py) and
calls the same-origin /services/gameResult/findByGameId endpoint, but saves the
whole response instead of just scores — for post-mortem debugging of a game.

Usage:
  .venv/bin/python scripts/ide/dump_game.py 892676311
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
import _common as C
import play_my_code as pmc


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("game_id", type=int)
    ap.add_argument("--user-id", type=int, default=pmc.MY_USER_ID)
    ap.add_argument("--debug-port", type=int, default=pmc.DEBUG_PORT)
    ap.add_argument("--profile", default=str(C.TROLLFARM_DIR / ".cg-browser-profile"))
    ap.add_argument("--browser-binary", default=pmc.BRAVE)
    args = ap.parse_args()

    driver = pmc.get_driver(args)
    driver.set_script_timeout(60)
    res = driver.execute_async_script(
        r"""
        const cb = arguments[arguments.length - 1];
        fetch('/services/gameResult/findByGameId', {
          method:'POST', headers:{'Content-Type':'application/json'},
          credentials:'include', body: JSON.stringify([arguments[0], arguments[1]])
        }).then(r=>r.json()).then(cb).catch(e=>cb({err:String(e)}));
        """,
        args.game_id,
        args.user_id,
    )
    out = C.EVAL_DIR / f"ide_game_{args.game_id}.json"
    out.write_text(json.dumps(res, indent=1))
    print(f"saved {out} ({out.stat().st_size} bytes), keys: {sorted(res)[:20]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
