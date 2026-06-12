"""Flatten a bot crate into a single CodinGame submission file.

Recursively replaces every `mod foo;` in bots/<pkg>/src/main.rs with the
inlined module source (comment lines stripped, indentation compacted),
stamps the build time, writes src/main.rs.flattened, and copies the result
to the clipboard (wl-copy).

Usage:
  uv run --project tools tools/flatten.py trollfarm    # or: just submit
"""

import argparse
import re
import subprocess
from datetime import datetime
from pathlib import Path

BOTS_DIR = Path(__file__).resolve().parent.parent / "bots"
MOD_DECL = re.compile(r"^\s*(?:pub\s+)?mod\s+(\w+)\s*;.*$", re.MULTILINE)


def resolve_module(parent_dir: Path, name: str) -> Path | None:
    """Locate a module as parent_dir/<name>.rs or parent_dir/<name>/mod.rs."""
    for candidate in (parent_dir / f"{name}.rs", parent_dir / name / "mod.rs"):
        if candidate.exists():
            return candidate
    return None


def inline(file_path: Path) -> str:
    """Return the file's source with every `mod foo;` replaced by its inlined
    module body."""
    src = file_path.read_text()
    parent_dir = file_path.parent

    def repl(m: re.Match[str]) -> str:
        name = m.group(1)
        child = resolve_module(parent_dir, name)
        if child is None:
            return m.group(0)
        # Strip blank lines and full-line comments. Only a real `//` prefix
        # qualifies: a bare `/` also matches rustfmt's division continuation
        # lines (`/ troll.movement_speed.max(1);`) and silently breaks the
        # pasted build (compile error, score -2 in the IDE).
        body = "\n".join(
            "    " + line
            for line in inline(child).splitlines()
            if line.strip() and not line.strip().startswith("//")
        )
        return f"mod {name} {{\n{body}\n}}\n"

    return MOD_DECL.sub(repl, src)


def stamp_build_time(flat: str, ts: str) -> str:
    """Stamp the build time, after comment stripping so it survives: rewrite
    the `FLATTENED_AT` constant (readable at runtime) and prepend a
    `// flattened <ts>` line (visible at the top of the pasted code)."""
    flat = re.sub(
        r'(const FLATTENED_AT: &str = )"[^"]*"', rf'\g<1>"{ts}"', flat, count=1
    )
    return f"// flattened {ts}\n" + flat


def copy_to_clipboard(text: str) -> None:
    """Copy via wl-copy with stdout/stderr detached: the clipboard daemon it
    forks inherits our fds and would otherwise hold a caller's captured pipe
    open (`just submit` under capture_output=True hangs waiting for EOF)."""
    subprocess.run(
        ["wl-copy"],
        input=text,
        text=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("pkg", help="bot crate in bots/")
    args = ap.parse_args()

    src_dir = BOTS_DIR / args.pkg / "src"
    flat = inline(src_dir / "main.rs")
    flat = "\n".join(line.replace("    ", " ") for line in flat.splitlines()) + "\n"
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    flat = stamp_build_time(flat, ts)

    out = src_dir / "main.rs.flattened"
    out.write_text(flat)
    copy_to_clipboard(flat)

    print(
        f"\033[92mWrote\033[0m {out.relative_to(src_dir.parent)} "
        f"({len(flat.splitlines())} lines)"
    )
    print(f"\033[94mCopied\033[0m to clipboard  \033[90m(flattened {ts})\033[0m")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
