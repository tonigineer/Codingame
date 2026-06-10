import re
import subprocess
import sys
from datetime import datetime
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

    # Stamp the build time. Done LAST, after comment stripping, so it survives:
    #  1. rewrite the `FLATTENED_AT` constant so the bot can eprintln it at runtime
    #     (a comment can't be read at runtime);
    #  2. prepend a `// flattened <ts>` first line for eyeballing the pasted code.
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    flat = re.sub(
        r'(const FLATTENED_AT: &str = )"[^"]*"', rf'\g<1>"{ts}"', flat, count=1
    )
    flat = f"// flattened {ts}\n" + flat

    out = project_dir / "main.rs.flattened"
    out.write_text(flat)
    # wl-copy forks a clipboard daemon that inherits our fds; point its
    # stdout/stderr at /dev/null so it can't hold a caller's captured pipe open
    # (otherwise `just submit` under capture_output=True hangs forever waiting
    # for EOF). stdin still carries the payload.
    subprocess.run(
        ["wl-copy"],
        input=flat,
        text=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    print(
        f"\033[92mWrote\033[0m {out.relative_to(project_dir.parent)} ({len(flat.splitlines())} lines)"
    )
    print(f"\033[94mCopied\033[0m to clipboard  \033[90m(flattened {ts})\033[0m")
