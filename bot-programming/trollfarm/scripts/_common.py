"""Shared paths and helpers for the Troll-Farm tooling scripts.

Every script lives in a category subfolder (`local/`, `replays/`, `ide/`) and
reaches this module via a small bootstrap at the top of the file:

    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # scripts/
    import _common as C

Path anchors are derived from this file's location, so nothing hard-codes
parent-directory counts and scripts keep working wherever they are run from.
"""

import os
import random
import subprocess
from pathlib import Path

# --- path anchors ----------------------------------------------------------
SCRIPTS_DIR = Path(__file__).resolve().parent          # .../trollfarm/scripts
TROLLFARM_DIR = SCRIPTS_DIR.parent                     # .../trollfarm
BOT_GAMES_DIR = TROLLFARM_DIR.parent                   # .../bot-programming (flatten.py)
WORKSPACE_DIR = BOT_GAMES_DIR.parent                   # cargo workspace root (Cargo.toml)
GAME_DIR = TROLLFARM_DIR / "codingame"                 # referee jar + bot binaries
EVAL_DIR = TROLLFARM_DIR / "eval"                      # benchmark/sweep outputs
REPLAYS_DIR = TROLLFARM_DIR / "replays"                # downloaded arena replays

# --- well-known names ------------------------------------------------------
JAR = "troll-farm-1.0-SNAPSHOT.jar"
TUNING_BIN = "trollfarm-tuning"                         # the --features tuning build
DEFAULT_REF = "./trollfarm-ref-gold-X"                 # default benchmark opponent
PUZZLE_ID = "spring-challenge-2026-troll-farm"

INT64_MIN, INT64_MAX = -(2**63), 2**63 - 1


def make_seeds(n: int, base: int) -> list[int]:
    """The benchmark's reproducible game-seed draw: a base seeds a local RNG
    that draws `n` full-int64 seeds. Same base -> same set (so eval/tune/sweep
    all replay the *same* maps for a given base)."""
    rng = random.Random(base)
    return [rng.randint(INT64_MIN, INT64_MAX) for _ in range(n)]


def set_tf_env(config: dict) -> None:
    """Apply a {param: value} config as `TF_*` env overrides for the tuning bot.
    Floats use repr() so large sentinels (1e9) don't become scientific notation."""
    for name, value in config.items():
        os.environ[f"TF_{name.upper()}"] = repr(value) if isinstance(value, float) else str(value)


def build_tuning_bot() -> str:
    """Build the `--features tuning` bot and deploy it as GAME_DIR/TUNING_BIN.

    Returns the P1 command (relative to GAME_DIR) the referee should launch.
    The tuning build reads `TF_*` env overrides at runtime; see set_tf_env.
    """
    print("Building --features tuning bot ...")
    subprocess.run(
        ["cargo", "build", "--release", "-p", "trollfarm", "--features", "tuning"],
        cwd=WORKSPACE_DIR, check=True,
    )
    dst = GAME_DIR / TUNING_BIN
    dst.write_bytes((WORKSPACE_DIR / "target/release/trollfarm").read_bytes())
    dst.chmod(0o755)
    print(f"Deployed {dst}")
    return f"./{TUNING_BIN}"
