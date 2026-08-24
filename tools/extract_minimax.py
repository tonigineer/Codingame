"""Extract the minimax harness from common/ into one game-agnostic Rust file.

Pulls `GameError`, the `Player`/`Game`/`Strategy` traits and the negamax
`Minimax` (with ahash swapped for std `HashMap`) out of common/src, and
appends a skeleton game plus `main()` — define the game, parse the referee
input, done. This covers the CodinGame case flatten.py can't: a bot that
would otherwise depend on the `common` crate (inlining `use common::…` is
an open TODO there).

Usage:
  python tools/extract_minimax.py    # rustc-typecheck, then copy to clipboard
"""

import argparse
import subprocess
import tempfile
from pathlib import Path

COMMON_SRC = Path(__file__).resolve().parent.parent / "common" / "src"

HEADER = """\
// Minimax harness extracted from common/ by tools/extract_minimax.py.
// Don't edit the harness section — regenerate it instead. Your game goes
// in the section at the bottom.
#![allow(dead_code)]

use std::collections::HashMap;
"""

GAME_TEMPLATE = """\
// ===================== your game (replace the todo!()s) ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Me,
    Foe,
}

impl Player for Mark {
    fn other(&self) -> Self {
        match self {
            Mark::Me => Mark::Foe,
            Mark::Foe => Mark::Me,
        }
    }

    fn index(&self) -> usize {
        match self {
            Mark::Me => 0,
            Mark::Foe => 1,
        }
    }

    fn symbol(&self) -> char {
        match self {
            Mark::Me => 'M',
            Mark::Foe => 'F',
        }
    }
}

#[derive(Debug, Clone)]
pub struct MyGame {
    // TODO: the position (bitboards encourage themselves) ...
    current_player: Mark,
}

impl Game for MyGame {
    type PlayerMask = Mark;
    type Move = usize;

    fn get_current_player(&self) -> Mark {
        self.current_player
    }

    fn apply_move(&mut self, chosen_move: usize) {
        // TODO: place the move AND flip self.current_player.
        let _ = chosen_move;
        todo!()
    }

    fn undo_move(&mut self, chosen_move: usize) {
        // TODO: the exact inverse of apply_move (must also flip the player).
        let _ = chosen_move;
        todo!()
    }

    fn get_possible_moves(&self) -> impl Iterator<Item = usize> {
        // TODO: legal moves, strongest first — alpha-beta prunes off this
        // order. (Empty placeholder because `todo!()` can't infer the
        // opaque iterator type.)
        std::iter::empty()
    }

    fn is_finished(&self) -> bool {
        todo!()
    }

    fn get_winner(&self) -> Option<Mark> {
        todo!()
    }

    fn render(&self) {}

    fn evaluate(&self) -> f32 {
        // Heuristic for the player to move, zero-sum, inside (-1.0, 1.0) so
        // it can never outrank a real win (terminals score ±1/depth). Only
        // consulted at the depth horizon — 0.0 is fine for games the search
        // solves outright.
        0.0
    }

    fn get_game_state_hash(&self) -> u64 {
        // TODO: Zobrist hash. MUST mix in the side to move, or transposed
        // positions with opposite movers alias in the transposition table.
        todo!()
    }
}

fn main() {
    // TODO: parse the referee input into a MyGame. Re-derive whose turn it
    // is from the position itself — compute_move plays for the player to
    // move, so no separate "which side am I" bookkeeping is needed.
    let game = MyGame {
        current_player: Mark::Me,
    };

    let mut bot = Minimax::new(9);
    let chosen_move = bot.compute_move(&game).expect("no legal moves");
    println!("{chosen_move}");
}
"""


def top_level_items(src: str) -> list[str]:
    """Split a Rust file into top-level items (doc comments and attributes
    stay attached to the item they precede). Brace counting is naive but
    sufficient: string literals in these files only contain balanced braces.
    """
    items: list[str] = []
    buf: list[str] = []
    depth = 0

    for line in src.splitlines():
        stripped = line.strip()
        if depth == 0 and not buf and not stripped:
            continue

        buf.append(line)
        depth += line.count("{") - line.count("}")

        ends_item = stripped.endswith(("}", ";")) and not stripped.startswith(
            ("//", "#[")
        )
        if depth == 0 and ends_item:
            items.append("\n".join(buf))
            buf = []

    if buf:
        items.append("\n".join(buf))
    return items


def header_of(item: str) -> str:
    """The first line of an item that is neither a comment nor an attribute."""
    for line in item.splitlines():
        stripped = line.strip()
        if stripped and not stripped.startswith(("//", "#[")):
            return stripped
    return ""


def harness_items() -> list[str]:
    """The harness, in dependency order: error + traits, then the searcher."""
    lib = top_level_items((COMMON_SRC / "lib.rs").read_text())
    mod = top_level_items((COMMON_SRC / "search" / "mod.rs").read_text())
    minimax = top_level_items((COMMON_SRC / "search" / "minimax.rs").read_text())

    items = [
        it
        for it in lib
        if not header_of(it).startswith(("use ", "pub mod "))
        and "Competition" not in header_of(it)
    ]
    items += [it for it in mod if "trait Strategy" in header_of(it)]
    items += [
        it.replace("AHashMap", "HashMap")
        for it in minimax
        if not header_of(it).startswith("use ")
    ]
    return items


def assemble() -> str:
    harness = "\n\n".join(harness_items())
    return (
        f"{HEADER}\n"
        "// ========================== harness (generated) ==========================="
        f"\n\n{harness}\n\n{GAME_TEMPLATE}"
    )


def rustc_check(code: str) -> int:
    """Typecheck the generated template with rustc (metadata only, no binary)."""
    with tempfile.TemporaryDirectory(prefix="minimax-template-") as tmp:
        rs = Path(tmp) / "main.rs"
        rs.write_text(code)
        result = subprocess.run(
            ["rustc", "--edition", "2024", "--emit=metadata", "--out-dir", tmp, rs],
            capture_output=True,
            text=True,
        )

    if result.returncode == 0:
        print("\033[92mrustc check passed\033[0m")
    else:
        print(f"\033[91mrustc check failed\033[0m\n{result.stderr}")
    return result.returncode


def copy_to_clipboard(text: str) -> None:
    """Copy via wl-copy with stdout/stderr detached: the clipboard daemon it
    forks inherits our fds and would otherwise hold a caller's captured pipe
    open."""
    subprocess.run(
        ["wl-copy"],
        input=text,
        text=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def main() -> int:
    argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    ).parse_args()

    code = assemble()

    if (rc := rustc_check(code)) != 0:
        return rc

    copy_to_clipboard(code)
    print(f"\033[94mCopied\033[0m to clipboard ({len(code.splitlines())} lines)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
