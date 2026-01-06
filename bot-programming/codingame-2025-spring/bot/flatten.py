#!/usr/bin/env python3
"""
flatten.py – produce a single-file main.rs for CodinGame submission
Run from project root:  python flatten.py
"""
from pathlib import Path
import re
import sys

project_dir = Path(__file__).parent / "src"
mod_decl    = re.compile(r'^\s*mod\s+(\w+)\s*;.*$', re.MULTILINE)

def inline(file_path: Path) -> str:
    """Return file contents with every `mod foo;` replaced by its inlined source."""
    src = file_path.read_text()

    def repl(m):
        name = m.group(1)
        child = project_dir / f"{name}.rs"
        if not child.exists():      # keep declaration if file is missing
            return m.group(0)
        body = inline(child)        # recurse
        # indent module body by 4 spaces
        indented = "\n".join("    " + line for line in body.splitlines())
        return f"mod {name} {{\n{indented}\n}}"

    return mod_decl.sub(repl, src)

if __name__ == "__main__":
    flat = inline(project_dir / "main.rs")
    out  = project_dir / "main.rs.flattened"
    out.write_text(flat)
    print(f"✅  Wrote {out.relative_to(project_dir.parent)} ({len(flat.splitlines())} lines)")
