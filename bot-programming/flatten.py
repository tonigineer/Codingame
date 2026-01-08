#!/usr/bin/env python3

import re
import subprocess
import sys
from pathlib import Path

project_dir = Path(__file__).parent / sys.argv[1] / "src"
mod_decl = re.compile(r"^\s*pub mod\s+(\w+)\s*;.*$", re.MULTILINE)


def inline(file_path: Path) -> str:
    """Return file contents with every `mod foo;` replaced by its inlined source."""
    src = file_path.read_text()

    def repl(m):
        name = m.group(1)
        child = project_dir / f"{name}.rs"
        if not child.exists():
            return m.group(0)
        body = inline(child)

        indented = "\n".join("    " + line for line in body.splitlines())
        return f"mod {name} {{\n{indented}\n}}\n"

    return mod_decl.sub(repl, src)


if __name__ == "__main__":
    flat = inline(project_dir / "main.rs")
    flat += "\n"
    out = project_dir / "main.rs.flattened"
    out.write_text(flat)
    subprocess.run(["wl-copy"], input=flat, text=True)

    print(
        f"\033[92mWrote\033[0m {out.relative_to(project_dir.parent)} ({len(flat.splitlines())} lines)"
    )

    print("\033[94mCopied\033[0m to clipboard")
