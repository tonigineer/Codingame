use std::io::{self, Write};

use crate::Game;

const ZOBRIST_SIDE_TO_MOVE: u64 = 0x8A24_B6DF_19E4_7C90;

const ZOBRIST_TABLE: [[u64; 9]; 2] = [
    [
        0xD5D2_2C1E_4B6B_2A2D,
        0x6AC5_3F90_2B3C_1159,
        0x8F52_5A17_6B92_0E7B,
        0x3BD0_5E43_A9E3_B1F4,
        0xC1B7_2F81_2D9C_4F23,
        0x75A4_1D62_E38A_6C91,
        0x9E61_9C04_57AD_334A,
        0x12F7_6AB9_8C01_DD2E,
        0x4C8B_EE17_017F_9B85,
    ],
    [
        0xA94B_2E39_F0C4_7A1D,
        0x51E9_0D84_0D7C_0A3B,
        0xF2B1_5C6F_6CE1_8452,
        0x0B7C_9F23_2B18_5F67,
        0xE7D3_1A90_9F42_CE08,
        0x28C6_7ED2_34A1_90D5,
        0xB3A8_0B53_1E2C_77F9,
        0x59FE_2A44_E8D6_1C0E,
        0x84D9_34B0_2C57_AA11,
    ],
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerMask {
    X,
    O,
}

impl PlayerMask {
    pub fn other(&self) -> Self {
        match &self {
            PlayerMask::X => PlayerMask::O,
            PlayerMask::O => PlayerMask::X,
        }
    }

    pub fn index(&self) -> usize {
        match &self {
            PlayerMask::X => 0,
            PlayerMask::O => 1,
        }
    }

    pub fn symbol(&self) -> char {
        match &self {
            PlayerMask::X => 'X',
            PlayerMask::O => 'O',
        }
    }

    pub fn colored_symbol(&self) -> String {
        match self {
            PlayerMask::X => format!("\x1b[34m{}\x1b[0m", self.symbol()),
            PlayerMask::O => format!("\x1b[32m{}\x1b[0m", self.symbol()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub x_board: u16,
    pub o_board: u16,
}

impl Board {
    pub fn new() -> Self {
        Self {
            x_board: 0u16,
            o_board: 0u16,
        }
    }

    pub fn get(&self, mark: PlayerMask) -> u16 {
        match mark {
            PlayerMask::X => self.x_board,
            PlayerMask::O => self.o_board,
        }
    }
}

#[derive(Debug)]
pub struct TicTacToe {
    pub board: Board,
    pub current_player: PlayerMask,
}

impl TicTacToe {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_player: PlayerMask::X,
        }
    }
}

impl Game for TicTacToe {
    type PlayerMask = PlayerMask;
    type Move = usize;

    fn get_possible_moves(&self) -> impl Iterator<Item = Self::Move> {
        const BITS: [u16; 9] = [1, 2, 4, 8, 16, 32, 64, 128, 256];
        let board = self.board.x_board | self.board.o_board;
        (0..=8).filter(move |&i| (board & BITS[i]) == 0)
    }

    fn apply_move(&mut self, chosen_move: Self::Move) {
        match self.current_player {
            PlayerMask::X => self.board.x_board |= 1 << chosen_move,
            PlayerMask::O => self.board.o_board |= 1 << chosen_move,
        }

        self.current_player = self.current_player.other();
    }

    fn undo_move(&mut self, chosen_move: Self::Move) {
        self.current_player = self.current_player.other();

        match self.current_player {
            PlayerMask::X => self.board.x_board &= !(1 << chosen_move),
            PlayerMask::O => self.board.o_board &= !(1 << chosen_move),
        }
    }

    fn get_current_player_index(&self) -> usize {
        self.current_player.index()
    }

    fn get_current_player_symbol(&self) -> char {
        self.current_player.symbol()
    }

    fn get_current_player(&self) -> Self::PlayerMask {
        self.current_player
    }

    fn is_finished(&self) -> bool {
        const FULL: u16 = (1 << 9) - 1; // 0b1_1111_1111 == 0x1FF
        ((self.board.x_board | self.board.o_board) == FULL) || self.get_winner().is_some()
    }

    fn get_winner(&self) -> Option<PlayerMask> {
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

        for &m in &WINS {
            if self.board.x_board & m == m {
                return Some(PlayerMask::X);
            }
            if self.board.o_board & m == m {
                return Some(PlayerMask::O);
            }
        }

        None
    }

    fn render(&self) {
        print!("\x1B[2J\x1B[H"); // clear screen

        for r in 0..3 {
            let mut line = String::with_capacity(8);
            for c in 0..3 {
                let idx = r * 3 + c;
                let bit = 1 << idx;

                line.push(' ');

                if self.board.x_board & bit != 0 {
                    line.push_str(&PlayerMask::X.colored_symbol());
                } else if self.board.o_board & bit != 0 {
                    line.push_str(&PlayerMask::O.colored_symbol());
                } else {
                    line.push_str(&idx.to_string());
                }

                line.push(' ');

                if c < 2 {
                    line.push('|');
                }
            }

            print!("{}\n", line);

            if r < 2 {
                print!("---+---+---\n");
            }
        }
        print!("\n");

        if let Some(w) = self.get_winner() {
            print!(" Winner: {}\n", w.colored_symbol());
        }

        let _ = io::stdout().flush();
    }

    fn get_game_state_score(&self, _player: &Self::PlayerMask) -> f32 {
        // INFO: Tic-Tac-Toe is a solved game where perfect play can be achieved through
        // exhaustive search. Therefore, heuristic evaluation of intermediate states
        // is unnecessary, and we return a neutral score.

        0f32
    }

    fn get_game_state_hash(&self) -> u64 {
        let mut h = 0u64;
        for i in 0..9 {
            let bit = 1u16 << i;
            if (self.board.x_board & bit) != 0 {
                h ^= ZOBRIST_TABLE[PlayerMask::X.index()][i];
            } else if (self.board.o_board & bit) != 0 {
                h ^= ZOBRIST_TABLE[PlayerMask::O.index()][i];
            }
        }

        if matches!(self.current_player, PlayerMask::X) {
            h ^= ZOBRIST_SIDE_TO_MOVE;
        }

        h
    }
}

#[cfg(test)]
mod tests {
    use crate::games::ttt::*;
    use crate::Game;

    #[test]
    fn test_tictactoe_initial_state() {
        let game = TicTacToe::new();

        assert_eq!(game.board.get(PlayerMask::X), 0);
        assert_eq!(game.board.get(PlayerMask::O), 0);

        assert_eq!(game.current_player, PlayerMask::X);
        assert_eq!(game.get_current_player_index(), 0);
    }
}
