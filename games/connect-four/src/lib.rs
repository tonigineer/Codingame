use common::Game;
use common::Player;
use std::fmt::Write as _;
use std::io::{self, Write};

const ZOBRIST_SIDE_TO_MOVE: u64 = 0x8A24_B6DF_19E4_7C90;

const ZOBRIST: [[u64; 42]; 2] = [
    [
        0x950E_87D7_F560_6615,
        0x2C61_275C_9E6B_6CF8,
        0x1F00_BCA0_042D_B923,
        0x6DBC_A290_A9EA_B706,
        0x4C10_A4FE_30CF_FDDA,
        0xF26F_FF4C_C4FD_394D,
        0x6814_A2BC_786A_6D2D,
        0xA26B_351E_6C80_42C5,
        0x5476_0E7F_BC05_1C6C,
        0xD4C0_8880_A5A4_666D,
        0x2961_0AE0_EED8_F1E7,
        0xC34B_D8E2_FE52_13E5,
        0x6C50_AFB6_E9FB_123D,
        0x6F28_D015_A2AA_0B9D,
        0x4E38_5994_EBAC_94AF,
        0x194F_9545_ADBA_52CE,
        0xC675_CE05_588F_882F,
        0x57DE_8C05_1D4B_7EF2,
        0xD998_EFD8_2733_E933,
        0x6DF2_16C3_3F8F_3201,
        0x11DC_6F3F_CB57_D5D8,
        0x8860_A847_2202_5E05,
        0x3317_6469_AA6E_F630,
        0x6075_07EB_C5B8_64D7,
        0x7A2F_1108_8D29_B146,
        0xDA10_FAAA_6FC2_4B83,
        0x2DE2_88F1_2FCB_9940,
        0xB989_37DF_EF04_1066,
        0xDD4B_712E_D355_871E,
        0xC5B7_9031_4A2E_3224,
        0x07FD_C889_FA01_7ED7,
        0x81EE_ADD7_1198_BF15,
        0x3A46_305C_425A_7DE1,
        0xAAAB_C8D3_66E0_440D,
        0x3371_364F_C51D_1A5E,
        0x4763_DD19_1AC4_4B70,
        0x0165_90C5_5646_E6D0,
        0x0B7A_6E1D_81E4_B9E7,
        0xE5A2_A8BE_F16E_981A,
        0x1167_FBA4_A292_7979,
        0x3D01_AC0F_1B53_4B87,
        0xD27A_5F0F_5532_C867,
    ],
    [
        0xEE26_CBC0_358B_24D3,
        0x9BDB_39B2_CA3C_6A00,
        0x8DE0_6FBE_1A74_1555,
        0xD625_7B49_2186_C8B5,
        0xDEE7_539C_5394_45F3,
        0x4307_513F_1EC1_B0B1,
        0x1D79_0BCA_EFFD_4D2D,
        0xDE18_F50A_43CF_423A,
        0xD36C_78AB_3537_A844,
        0x64B5_E3F8_1A29_3B3B,
        0xE8EE_F3D6_7646_F8A9,
        0xA88D_379D_B047_719D,
        0xF177_D49F_03DD_C3BF,
        0xA745_FDD5_5296_5BCA,
        0xD0B6_A46A_7048_DACA,
        0xFCE7_9398_852E_0400,
        0x760C_9B75_6320_DBE3,
        0x4E52_B419_8027_1E94,
        0x293F_6584_8AA1_8F43,
        0x520E_015E_444E_D0F2,
        0x793F_F51B_B0BA_F029,
        0x7AD9_5556_8F86_A26A,
        0x1C72_0603_EC86_02D9,
        0xD08E_7565_D487_D342,
        0x3102_8829_0B43_DBFB,
        0xD50C_A99E_8E59_EA07,
        0x6C24_E82C_6DBB_AC73,
        0xB3E6_17BC_719C_B81B,
        0x29B0_8AB5_D58F_3AE5,
        0x4E5C_9DA0_F7F5_6CFD,
        0x07E7_39F4_0EC6_B03D,
        0xCF04_E03B_48D7_70A4,
        0x81C1_D6F0_21C3_F8B1,
        0x7F42_3F3D_A4AB_72E2,
        0xCBE1_8AD8_610E_00D1,
        0xF776_F2F6_3E43_B9C8,
        0xE7B2_F12F_62A1_E7C2,
        0x64A7_C5F4_A8E3_43D9,
        0xF125_F301_7E8C_4278,
        0x9384_F2BB_776B_28DD,
        0xAE91_A8DA_D2C7_B77F,
        0x1B15_FA29_C19B_5B56,
    ],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerMask {
    Red,
    Yellow,
}

impl common::Player for PlayerMask {
    fn other(&self) -> Self {
        match self {
            PlayerMask::Red => PlayerMask::Yellow,
            PlayerMask::Yellow => PlayerMask::Red,
        }
    }

    fn index(&self) -> usize {
        match self {
            PlayerMask::Red => 0,
            PlayerMask::Yellow => 1,
        }
    }

    fn symbol(&self) -> char {
        // Fullwidth letters so stones line up with the fullwidth column digits.
        match self {
            PlayerMask::Red => '\u{FF32}',    // Ｒ
            PlayerMask::Yellow => '\u{FF39}', // Ｙ
        }
    }
}

impl PlayerMask {
    #[must_use]
    pub fn colored_symbol(&self) -> String {
        match self {
            PlayerMask::Red => format!("\x1b[31m{}\x1b[0m", self.symbol()),
            PlayerMask::Yellow => format!("\x1b[33m{}\x1b[0m", self.symbol()),
        }
    }
}

/// Bitboard, column-major with one sentinel bit above each column
/// (bit `col * (H + 1) + row`).
#[derive(Debug, Clone, Copy)]
pub struct Board {
    /// Every stone on the board, both colors.
    pub both: u64,
    /// The stones of the player to move (flips meaning every ply).
    pub single: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    #[must_use]
    pub fn new() -> Self {
        Self { both: 0, single: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectFour<const W: usize, const H: usize> {
    pub board: Board,
    pub current_player: PlayerMask,
}

impl<const W: usize, const H: usize> Default for ConnectFour<W, H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize, const H: usize> ConnectFour<W, H> {
    /// Columns in center-out order — the strongest-first ordering for move
    /// generation, which is what makes alpha-beta cut early.
    const MOVE_ORDER: [usize; W] = Self::move_order();

    /// The top cell of every column; all set ⇒ the board is full.
    const TOP_MASK_ALL: u64 = Self::top_mask_all();

    #[must_use]
    pub fn new() -> Self {
        const {
            assert!((H + 1) * W <= 64, "bitboard must fit in a u64");
            assert!(W * H <= 42, "ZOBRIST table only covers 42 cells");
        }
        Self {
            board: Board::new(),
            current_player: PlayerMask::Red,
        }
    }

    const fn bottom_mask(col: usize) -> u64 {
        1u64 << (col * (H + 1))
    }

    const fn top_mask(col: usize) -> u64 {
        1u64 << (col * (H + 1) + (H - 1))
    }

    const fn column_mask(col: usize) -> u64 {
        ((1u64 << H) - 1) << (col * (H + 1))
    }

    const fn move_order() -> [usize; W] {
        let mut order = [0; W];
        let center = (W - 1) / 2;
        let mut i = 0;
        while i < W {
            order[i] = if i % 2 == 0 {
                center - i / 2
            } else {
                center + i.div_ceil(2)
            };
            i += 1;
        }
        order
    }

    const fn top_mask_all() -> u64 {
        let mut acc = 0;
        let mut col = 0;
        while col < W {
            acc |= Self::top_mask(col);
            col += 1;
        }
        acc
    }

    /// Whether `stones` (one player's stones) contain four in a row.
    fn has_won(stones: u64) -> bool {
        // The four line directions as bit strides: vertical neighbours are 1
        // apart, horizontal ones H+1 (a column plus its sentinel bit),
        // diagonals H and H+2.
        let strides = [1, H + 1, H + 2, H];
        strides.into_iter().any(|s| {
            let m = stones & (stones >> s);
            m & (m >> (2 * s)) != 0
        })
    }

    /// Map a board bit index to a 0-based cell id, skipping sentinel bits.
    fn cell_id(bit_index: usize) -> Option<usize> {
        let col = bit_index / (H + 1);
        let row = bit_index % (H + 1);
        (col < W && row < H).then_some(col * H + row)
    }

    /// XOR the Zobrist keys of every stone in `stones`.
    fn hash_stones(mut stones: u64, keys: &[u64; 42]) -> u64 {
        let mut h = 0;
        while stones != 0 {
            let idx = stones.trailing_zeros() as usize;
            stones &= stones - 1;
            if let Some(cell) = Self::cell_id(idx) {
                h ^= keys[cell];
            }
        }
        h
    }
}

impl<const W: usize, const H: usize> Game for ConnectFour<W, H> {
    type PlayerMask = PlayerMask;
    type Move = usize;

    fn get_possible_moves(&self) -> impl Iterator<Item = Self::Move> {
        Self::MOVE_ORDER
            .into_iter()
            .filter(move |&col| self.board.both & Self::top_mask(col) == 0)
    }

    fn apply_move(&mut self, chosen_move: Self::Move) {
        let mv =
            (self.board.both + Self::bottom_mask(chosen_move)) & Self::column_mask(chosen_move);
        self.board.single ^= self.board.both;
        self.board.both |= mv;

        self.current_player = self.current_player.other();
    }

    fn undo_move(&mut self, chosen_move: Self::Move) {
        let next =
            (self.board.both + Self::bottom_mask(chosen_move)) & Self::column_mask(chosen_move);
        let mv = if next != 0 {
            next >> 1
        } else {
            Self::top_mask(chosen_move)
        };

        self.board.both ^= mv;
        self.board.single ^= self.board.both;

        self.current_player = self.current_player.other();
    }

    fn get_current_player(&self) -> Self::PlayerMask {
        self.current_player
    }

    fn is_finished(&self) -> bool {
        (self.board.both & Self::TOP_MASK_ALL) == Self::TOP_MASK_ALL || self.get_winner().is_some()
    }

    fn get_winner(&self) -> Option<PlayerMask> {
        if Self::has_won(self.board.single) {
            return Some(self.current_player);
        }

        if Self::has_won(self.board.both ^ self.board.single) {
            return Some(self.current_player.other());
        }

        None
    }

    fn render(&self) {
        print!("\x1B[2J\x1B[H"); // clear screen

        for r in (0..H).rev() {
            let mut line = String::with_capacity(W * 2 - 1);

            for c in 0..W {
                line.push('|');
                line.push(' ');

                let bit = 1u64 << (c * (H + 1) + r);

                if (self.board.both & bit) == 0 {
                    line.push('\u{3000}');
                } else if (self.board.single & bit) != 0 {
                    line.push_str(&self.current_player.colored_symbol());
                } else {
                    line.push_str(&self.current_player.other().colored_symbol());
                }

                line.push(' ');
            }

            println!("{line}|");
        }

        let mut bottom_line = String::with_capacity(W * 4);
        for i in 0..W {
            #[allow(clippy::cast_possible_truncation)]
            let digit = char::from_u32(0xFF10 + i as u32).unwrap_or('?');
            let _ = write!(bottom_line, " {digit} +");
        }

        println!("+{bottom_line}");

        if let Some(w) = self.get_winner() {
            println!(" Winner: {}", w.colored_symbol());
        }

        let _ = io::stdout().flush();
    }

    fn get_game_state_score(&self, _player: &Self::PlayerMask) -> f32 {
        const TWO_WEIGHT: f32 = 1.0 / 3.0;
        const THREE_WEIGHT: f32 = 2.0 / 3.0;

        /// Count 2-in-a-rows and 3-in-a-rows over the same bit strides as
        /// `has_won`: 1 (vertical), H+1 (horizontal), H+2 and H (diagonals).
        fn count_sequences<const H: usize>(stones: u64) -> (u32, u32) {
            let mut n_two = 0u32;
            let mut n_three = 0u32;

            for s in [1, H + 1, H + 2, H] {
                let m = stones & (stones >> s);
                n_two += m.count_ones();
                n_three += (m & (m >> s)).count_ones();
            }

            (n_two, n_three)
        }

        #[allow(clippy::cast_precision_loss)]
        fn normalized_diff(player: u32, other: u32) -> f32 {
            let total = player + other;
            if total == 0 {
                0.0
            } else {
                (player as f32 - other as f32) / total as f32
            }
        }

        let (n_two, n_three) = count_sequences::<H>(self.board.single);
        let (n_two_other, n_three_other) =
            count_sequences::<H>(self.board.both ^ self.board.single);

        let n_two_score = normalized_diff(n_two, n_two_other);
        let n_three_score = normalized_diff(n_three, n_three_other);

        let combined_score = n_two_score * TWO_WEIGHT + n_three_score * THREE_WEIGHT;
        combined_score / 2.0
    }

    fn get_game_state_hash(&self) -> u64 {
        let me = self.current_player;
        let mut h = Self::hash_stones(self.board.single, &ZOBRIST[me.index()])
            ^ Self::hash_stones(
                self.board.both ^ self.board.single,
                &ZOBRIST[me.other().index()],
            );

        if matches!(me, PlayerMask::Red) {
            h ^= ZOBRIST_SIDE_TO_MOVE;
        }

        h
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_connect_four_initial_state() {
        let game: ConnectFour<7, 6> = ConnectFour::new();

        assert_eq!(game.board.both, 0);
        assert_eq!(game.board.single, 0);

        assert_eq!(game.current_player, PlayerMask::Red);
        assert_eq!(game.get_current_player_index(), 0);
    }
}
