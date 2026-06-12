# common

Shared library for two-player, turn-based, zero-sum games: the `Game`/`Player`
traits, an object-safe `Strategy` trait, a negamax `Minimax` searcher
(alpha-beta + transposition table), baseline bots, and a `Competition` runner.

Implement the `Game` trait for your game and every strategy in this crate —
and any future one — works on it unchanged. `games/tic-tac-toe` and
`games/connect-four` are complete reference implementations.

## Implementing a new game

### 1. The player type

A small `Copy + Eq` enum implementing `common::Player`:

```rust
use common::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Red,
    Blue,
}

impl Player for Mark {
    fn other(&self) -> Self { /* the opponent */ }
    fn index(&self) -> usize { /* 0 for the side that moves first, 1 for the other */ }
    fn symbol(&self) -> char { /* for rendering and prompts */ }
}
```

`index()` is load-bearing: `Competition` uses it to pick which strategy moves,
so the player who moves first in the initial position must be index `0`.

### 2. The game state

A `Clone`-able struct holding the position **and whose turn it is**:

```rust
#[derive(Debug, Clone)]
pub struct MyGame {
    board: /* ... bitboards encourage themselves ... */,
    current_player: Mark,
}
```

Search clones the state once per move computation and otherwise mutates it
with `apply_move`/`undo_move`, so cheap `Clone` is nice but not critical;
cheap apply/undo is what matters.

### 3. The `Game` impl

```rust
use common::Game;

impl Game for MyGame {
    type PlayerMask = Mark;
    type Move = usize; // anything Copy

    fn get_current_player(&self) -> Mark { self.current_player }

    fn apply_move(&mut self, chosen_move: usize) { /* place + flip current_player */ }
    fn undo_move(&mut self, chosen_move: usize) { /* exact inverse of apply_move */ }

    fn get_possible_moves(&self) -> impl Iterator<Item = usize> { /* legal moves */ }

    fn is_finished(&self) -> bool { /* win or draw */ }
    fn get_winner(&self) -> Option<Mark> { /* None while running or drawn */ }

    fn evaluate(&self) -> f32 { /* heuristic, see contract below */ }
    fn get_game_state_hash(&self) -> u64 { /* Zobrist, see contract below */ }

    fn render(&self) { /* print the board */ }
}
```

The contracts the search relies on:

| Method | Contract |
|--------|----------|
| `apply_move` / `undo_move` | Exact inverses, and **both flip `current_player`**. Search applies and undoes millions of moves on one state; any asymmetry corrupts every node above it. |
| `get_possible_moves` | Only legal moves; empty iff `is_finished()`. **Order strongest-first** — alpha-beta prunes off this ordering (connect-four yields center columns first; it's the difference between a fast and a useless search). |
| `get_winner` | Checked before the depth cutoff, at every node. Must be cheap. |
| `evaluate` | Score **from the perspective of the player to move**, zero-sum (`s` for one side means `-s` for the other), inside `(-1.0, 1.0)` so it can never outrank a real win (terminal nodes score `±1.0 / depth`). Only called at the depth horizon — returning `0.0` is fine for games minimax can solve outright (see tic-tac-toe). |
| `get_game_state_hash` | Keys the transposition table, so equal hashes are treated as equal positions. Zobrist-hash the stones **and XOR a side-to-move key** — without it, the same stones with the other side to move alias and the search returns garbage. |

### 4. Play it

```rust
use common::Competition;
use common::search::baseline::HumanPlayer;
use common::search::minimax::Minimax;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut competition = Competition::new(MyGame::new(), Minimax::new(10), HumanPlayer);
    competition.start(true)?; // true = render after every move
    Ok(())
}
```

`Minimax::new(depth)` searches `depth` plies; chain `.with_status_bar()` (or
set the public `status_bar` field) for a live, colored status line on stderr
while it computes — sampled ply, root-move progress, best score, nodes,
prunes, transposition hits and nodes/second. After the move, `Competition`
keeps the search's final summary visible under the board (see `status_line`
below). `HumanPlayer` reads moves from
stdin and additionally needs `Move: Eq + FromStr + Display` (a plain `usize`
move qualifies). `FirstPossibleMove` and `RandomMove` are baseline opponents
for tests; `competition.play_turn()` steps a single move when a test wants to
inspect the position mid-game.

## Implementing a new strategy

`Strategy` is object-safe and generic over the game, so a new bot is one impl
— no registration anywhere:

```rust
use common::search::Strategy;
use common::{Game, GameError};

pub struct MyBot;

impl<G: Game> Strategy<G> for MyBot {
    fn compute_move(&mut self, game: &G) -> Result<G::Move, GameError> {
        game.get_possible_moves()
            .next()
            .ok_or(GameError::NoMovesAvailable)
    }
}
```

Implement it for one concrete game instead (`impl Strategy<MyGame> for MyBot`)
when the bot needs game-specific knowledge.

`Strategy` has one optional method: `status_line()` returns a one-line
summary of the last `compute_move` (default `None`). Because every render
clears the screen, `Competition` reprints each player's line under the board
after each move — that's how minimax's last evaluation stays visible while
the human thinks.

## Testing a new game

Mirror `games/*/tests/`: drive full games through `Competition` and assert on
the outcome. The cheap, high-value cases:

- `FirstPossibleMove` vs itself — deterministic; pins the game logic.
- `RandomMove` vs itself — the game always terminates.
- Minimax beats `FirstPossibleMove`, and beats (or at least never loses to)
  `RandomMove`.
- Minimax vs Minimax ends in whatever the game's perfect-play result is — a
  strong whole-pipeline check (tic-tac-toe asserts the draw).

A unit test that `apply_move` then `undo_move` restores the position (compare
`get_game_state_hash`) catches the most common and most painful bug class
early.
