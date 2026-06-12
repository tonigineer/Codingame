use std::fmt::Write as _;
use std::io::{self, Write};

use common::Game;
use common::Player;

const ZOBRIST_SIDE_TO_MOVE: u64 = 0x3F1C_A9E2_5B70_D846;

/// 2 x 81 cell keys plus 9 constraint keys would be unwieldy as literals, so
/// the Zobrist tables are generated at compile time from a splitmix64 stream.
const fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const fn zobrist_keys<const N: usize>(seed: u64) -> [u64; N] {
    let mut keys = [0u64; N];
    let mut i = 0;
    while i < N {
        keys[i] = splitmix64(seed.wrapping_add((i as u64) << 32));
        i += 1;
    }
    keys
}

const ZOBRIST_TABLE: [[u64; 81]; 2] = [
    zobrist_keys(0xD5D2_2C1E_4B6B_2A2D),
    zobrist_keys(0xA94B_2E39_F0C4_7A1D),
];

/// Same stones with a different constraint allow different moves, so the
/// constraint is part of the position identity (no key for "play anywhere").
const ZOBRIST_CONSTRAINT: [u64; 9] = zobrist_keys(0x4C8B_EE17_017F_9B85);

const FULL: u16 = (1 << 9) - 1; // 0b1_1111_1111 == 0x1FF

const WINS: [u16; 8] = [
    0b000_000_111,
    0b000_111_000,
    0b111_000_000,
    0b001_001_001,
    0b010_010_010,
    0b100_100_100,
    0b100_010_001,
    0b001_010_100,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMask {
    X,
    O,
}

impl common::Player for PlayerMask {
    fn other(&self) -> Self {
        match self {
            PlayerMask::X => PlayerMask::O,
            PlayerMask::O => PlayerMask::X,
        }
    }

    fn index(&self) -> usize {
        match self {
            PlayerMask::X => 0,
            PlayerMask::O => 1,
        }
    }

    fn symbol(&self) -> char {
        match self {
            PlayerMask::X => 'X',
            PlayerMask::O => 'O',
        }
    }
}

impl PlayerMask {
    #[must_use]
    pub fn colored_symbol(&self) -> String {
        match self {
            PlayerMask::X => format!("\x1b[34m{}\x1b[0m", self.symbol()),
            PlayerMask::O => format!("\x1b[32m{}\x1b[0m", self.symbol()),
        }
    }
}

/// One 3x3 tic-tac-toe board as a pair of bitmasks. Doubles as the main
/// board, where bit `b` stands for "small board `b` won by that player".
#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub x_board: u16,
    pub o_board: u16,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    #[must_use]
    pub fn new() -> Self {
        Self {
            x_board: 0u16,
            o_board: 0u16,
        }
    }

    #[must_use]
    pub fn get(&self, mark: PlayerMask) -> u16 {
        match mark {
            PlayerMask::X => self.x_board,
            PlayerMask::O => self.o_board,
        }
    }

    #[must_use]
    pub fn occupied(&self) -> u16 {
        self.x_board | self.o_board
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        self.occupied() == FULL
    }

    #[must_use]
    pub fn winner(&self) -> Option<PlayerMask> {
        for &m in &WINS {
            if self.x_board & m == m {
                return Some(PlayerMask::X);
            }
            if self.o_board & m == m {
                return Some(PlayerMask::O);
            }
        }

        None
    }
}

/// Ultimate tic-tac-toe: a 3x3 of small tic-tac-toe boards. A move is
/// `board * 9 + cell` (both `0..9`, row-major). The cell you play in sends
/// the opponent to the small board with that index; if that board is already
/// won or full, they may play in any open board instead.
#[derive(Debug, Clone)]
pub struct UltimateTicTacToe {
    pub boards: [Board; 9],
    /// Bit `b` set = small board `b` won by that player; `winner()` on this
    /// board is the winner of the whole game.
    pub macro_board: Board,
    pub current_player: PlayerMask,
    /// The small board the next move is confined to (`None` = any open board).
    pub constraint: Option<usize>,
    /// Constraints are decided by the opponent's last move, not derivable
    /// from the move being undone — so undo needs them stacked.
    constraint_history: Vec<Option<usize>>,
}

impl Default for UltimateTicTacToe {
    fn default() -> Self {
        Self::new()
    }
}

impl UltimateTicTacToe {
    #[must_use]
    pub fn new() -> Self {
        Self {
            boards: [Board::new(); 9],
            macro_board: Board::new(),
            current_player: PlayerMask::X,
            constraint: None,
            constraint_history: Vec::new(),
        }
    }

    /// A small board accepts moves while nobody has won it and it still has
    /// empty cells.
    #[must_use]
    pub fn is_board_open(&self, board: usize) -> bool {
        self.macro_board.occupied() & (1 << board) == 0 && !self.boards[board].is_full()
    }
}

impl Game for UltimateTicTacToe {
    type PlayerMask = PlayerMask;
    type Move = usize;

    fn get_possible_moves(&self) -> impl Iterator<Item = Self::Move> {
        let boards = self.boards;
        let constraint = self.constraint;
        let open: u16 = (0..9)
            .filter(|&b| self.is_board_open(b))
            .fold(0, |mask, b| mask | (1 << b));

        (0..81).filter(move |&m| {
            let (board, cell) = (m / 9, m % 9);
            let board_allowed = match constraint {
                Some(forced) => board == forced,
                None => open & (1 << board) != 0,
            };
            board_allowed && boards[board].occupied() & (1 << cell) == 0
        })
    }

    fn apply_move(&mut self, chosen_move: Self::Move) {
        let (board, cell) = (chosen_move / 9, chosen_move % 9);

        match self.current_player {
            PlayerMask::X => self.boards[board].x_board |= 1 << cell,
            PlayerMask::O => self.boards[board].o_board |= 1 << cell,
        }

        if self.boards[board].winner() == Some(self.current_player) {
            match self.current_player {
                PlayerMask::X => self.macro_board.x_board |= 1 << board,
                PlayerMask::O => self.macro_board.o_board |= 1 << board,
            }
        }

        // The cell played decides where the opponent must answer; a won or
        // full target board frees them to play anywhere.
        self.constraint_history.push(self.constraint);
        self.constraint = self.is_board_open(cell).then_some(cell);

        self.current_player = self.current_player.other();
    }

    fn undo_move(&mut self, chosen_move: Self::Move) {
        self.current_player = self.current_player.other();

        let (board, cell) = (chosen_move / 9, chosen_move % 9);

        // The board was open before the move (it was playable), so any win
        // flag the mover holds on it was set by exactly this move.
        match self.current_player {
            PlayerMask::X => {
                self.boards[board].x_board &= !(1 << cell);
                self.macro_board.x_board &= !(1 << board);
            }
            PlayerMask::O => {
                self.boards[board].o_board &= !(1 << cell);
                self.macro_board.o_board &= !(1 << board);
            }
        }

        self.constraint = self
            .constraint_history
            .pop()
            .expect("undo_move without a matching apply_move");
    }

    fn get_current_player(&self) -> Self::PlayerMask {
        self.current_player
    }

    fn is_finished(&self) -> bool {
        self.get_winner().is_some() || (0..9).all(|b| !self.is_board_open(b))
    }

    fn get_winner(&self) -> Option<PlayerMask> {
        self.macro_board.winner()
    }

    fn render(&self) {
        print!("\x1B[2J\x1B[H"); // clear screen

        let playable: Vec<usize> = if self.is_finished() {
            Vec::new()
        } else {
            self.get_possible_moves().collect()
        };

        for big_row in 0..3 {
            for small_row in 0..3 {
                let mut line = String::new();
                for big_col in 0..3 {
                    let board = big_row * 3 + big_col;
                    for small_col in 0..3 {
                        let cell = small_row * 3 + small_col;
                        let idx = board * 9 + cell;
                        let bit = 1 << cell;

                        line.push(' ');

                        if self.boards[board].x_board & bit != 0 {
                            line.push(' ');
                            line.push_str(&PlayerMask::X.colored_symbol());
                        } else if self.boards[board].o_board & bit != 0 {
                            line.push(' ');
                            line.push_str(&PlayerMask::O.colored_symbol());
                        } else if playable.contains(&idx) {
                            let _ = write!(line, "{idx:>2}");
                        } else {
                            line.push_str(" ·");
                        }
                    }

                    if big_col < 2 {
                        line.push_str(" |");
                    }
                }
                println!("{line}");
            }

            if big_row < 2 {
                println!("----------+----------+---------");
            }
        }
        println!();

        let mut main = String::from(" Main board: ");
        for board in 0..9 {
            let bit = 1 << board;
            if self.macro_board.x_board & bit != 0 {
                main.push_str(&PlayerMask::X.colored_symbol());
            } else if self.macro_board.o_board & bit != 0 {
                main.push_str(&PlayerMask::O.colored_symbol());
            } else {
                main.push('·');
            }
            if board % 3 == 2 && board < 8 {
                main.push_str(" / ");
            }
        }
        println!("{main}");

        if let Some(w) = self.get_winner() {
            println!(" Winner: {}", w.colored_symbol());
        } else if self.is_finished() {
            println!(" Draw");
        } else {
            match self.constraint {
                Some(b) => println!(" Next move: board {b}"),
                None => println!(" Next move: any open board"),
            }
        }

        let _ = io::stdout().flush();
    }

    fn evaluate(&self) -> f32 {
        // INFO: No heuristic yet — fine for the baseline/human matchup this
        // crate ships with. Pointing minimax at ultimate tic-tac-toe needs a
        // real evaluation here first (the game is far too deep to solve).

        0f32
    }

    fn get_game_state_hash(&self) -> u64 {
        let mut h = 0u64;

        for board in 0..9 {
            for cell in 0..9 {
                let bit = 1u16 << cell;
                if self.boards[board].x_board & bit != 0 {
                    h ^= ZOBRIST_TABLE[PlayerMask::X.index()][board * 9 + cell];
                } else if self.boards[board].o_board & bit != 0 {
                    h ^= ZOBRIST_TABLE[PlayerMask::O.index()][board * 9 + cell];
                }
            }
        }

        if let Some(b) = self.constraint {
            h ^= ZOBRIST_CONSTRAINT[b];
        }

        if matches!(self.current_player, PlayerMask::X) {
            h ^= ZOBRIST_SIDE_TO_MOVE;
        }

        h
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_uttt_initial_state() {
        let game = UltimateTicTacToe::new();

        assert_eq!(game.macro_board.occupied(), 0);
        assert!((0..9).all(|b| game.boards[b].occupied() == 0));

        assert_eq!(game.current_player, PlayerMask::X);
        assert_eq!(game.get_current_player_index(), 0);

        assert_eq!(game.constraint, None);
        assert_eq!(game.get_possible_moves().count(), 81);
    }
}
