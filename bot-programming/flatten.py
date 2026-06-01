import re
import subprocess
import sys
from pathlib import Path

project_dir = Path(__file__).parent / sys.argv[1] / "src"
mod_decl = re.compile(r"^\s*(?:pub\s+)?mod\s+(\w+)\s*;.*$", re.MULTILINE)


def resolve_module(parent_dir: Path, name: str) -> Path | None:
    """Resolve a module name to its file path, checking both patterns:
    1. parent_dir/name.rs        (file module)
    2. parent_dir/name/mod.rs    (directory module)
    """
    file_mod = parent_dir / f"{name}.rs"
    if file_mod.exists():
        return file_mod

    dir_mod = parent_dir / name / "mod.rs"
    if dir_mod.exists():
        return dir_mod

    return None


def inline(file_path: Path) -> str:
    """Return file contents with every `mod foo;` replaced by its inlined source."""
    src = file_path.read_text()
    parent_dir = file_path.parent

    def repl(m):
        name = m.group(1)
        child = resolve_module(parent_dir, name)
        if child is None:
            return m.group(0)
        body = inline(child)

        indented = "\n".join(
            "    " + line
            for line in body.splitlines()
            if not line.strip().startswith("/") and len(line.strip()) > 0
        )
        return f"mod {name} {{\n{indented}\n}}\n"

    return mod_decl.sub(repl, src)


if __name__ == "__main__":
    flat = inline(project_dir / "main.rs")
    flat = "\n".join([line.replace("    ", " ") for line in flat.splitlines()])
    flat += "\n"
    out = project_dir / "main.rs.flattened"
    out.write_text(flat)
    subprocess.run(["wl-copy"], input=flat, text=True)

    print(
        f"\033[92mWrote\033[0m {out.relative_to(project_dir.parent)} ({len(flat.splitlines())} lines)"
    )

    print("\033[94mCopied\033[0m to clipboard")
