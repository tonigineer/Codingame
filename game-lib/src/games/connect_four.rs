use crate::Game;
use std::io::{self, Write};
use crate::Player;

const WIDTH: usize = 7;
const HEIGHT: usize = 6;

pub const ZOBRIST_SIDE_TO_MOVE: u64 = 0x8A24_B6DF_19E4_7C90;

pub const ZOBRIST: [[u64; 42]; 2] = [
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

impl crate::Player for PlayerMask {
    fn other(&self) -> Self {
        match &self {
            PlayerMask::Red => PlayerMask::Yellow,
            PlayerMask::Yellow => PlayerMask::Red,
        }
    }

    fn index(&self) -> usize {
        match &self {
            PlayerMask::Red => 0,
            PlayerMask::Yellow => 1,
        }
    }

    fn symbol(&self) -> char {
        match &self {
            //  '🔴', '🟡'
            PlayerMask::Red => '\u{FF32}',
            PlayerMask::Yellow => '\u{FF33}',
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub both: u64,
    pub single: u64,
}

impl Board {
    fn new() -> Self {
        Self { both: 0, single: 0 }
    }

    const fn bottom_mask(col: usize) -> u64 {
        1u64 << (col * (HEIGHT + 1))
    }

    const fn top_mask(col: usize) -> u64 {
        1u64 << (col * (HEIGHT + 1) + (HEIGHT - 1))
    }

    const fn column_mask(col: usize) -> u64 {
        ((1u64 << HEIGHT) - 1) << (col * (HEIGHT + 1))
    }

    fn top_mask_all() -> u64 {
        (0..WIDTH).map(Board::top_mask).sum()
    }
}

#[derive(Clone)]
pub struct ConnectFour {
    pub board: Board,
    pub current_player: PlayerMask,
}

impl Default for ConnectFour {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectFour {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_player: PlayerMask::Red,
        }
    }
}

impl Game for ConnectFour {
    type PlayerMask = PlayerMask;
    type Move = usize;

    fn get_possible_moves(&self) -> impl Iterator<Item = Self::Move> {
        const fn center_out_order() -> [usize; WIDTH] {
            let mut arr = [0; WIDTH];
            let center = WIDTH / 2;
            let mut i = 0;
            while i < WIDTH {
                arr[i] = if i % 2 == 0 {
                    center - (i / 2)
                } else {
                    center + i.div_ceil(2)
                };
                i += 1;
            }
            arr
        }

        center_out_order()
            .into_iter()
            .filter(move |&idx| self.board.both & Board::top_mask(idx) == 0)
    }

    fn apply_move(&mut self, chosen_move: Self::Move) {
        let mv =
            (self.board.both + Board::bottom_mask(chosen_move)) & Board::column_mask(chosen_move);
        self.board.single ^= self.board.both;
        self.board.both |= mv;

        self.current_player = self.current_player.other();
    }

    fn undo_move(&mut self, chosen_move: Self::Move) {
        let next =
            (self.board.both + Board::bottom_mask(chosen_move)) & Board::column_mask(chosen_move);
        let mv = if next != 0 {
            next >> 1
        } else {
            Board::top_mask(chosen_move)
        };

        self.board.both ^= mv;
        self.board.single ^= self.board.both;

        self.current_player = self.current_player.other();
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
        (self.board.both & Board::top_mask_all()) == Board::top_mask_all()
            || self.get_winner().is_some()
    }

    fn get_winner(&self) -> Option<PlayerMask> {
        fn has_won(p: u64) -> bool {
            // vertical (↑)
            let mut m = p & (p >> 1);
            if (m & (m >> 2)) != 0 {
                return true;
            }

            // horizontal (→) : shift by (7)
            m = p & (p >> (HEIGHT + 1));
            if (m & (m >> (2 * (HEIGHT + 1)))) != 0 {
                return true;
            }

            // diagonal (↗) : shift by (8)
            m = p & (p >> (HEIGHT + 2));
            if (m & (m >> (2 * (HEIGHT + 2)))) != 0 {
                return true;
            }

            // diagonal (↘) : shift by (6)
            m = p & (p >> HEIGHT);
            if (m & (m >> (2 * HEIGHT))) != 0 {
                return true;
            }

            false
        }

        if has_won(self.board.single) {
            return Some(self.current_player);
        }

        if has_won(self.board.both ^ self.board.single) {
            return Some(self.current_player.other());
        }

        None
    }

    fn render(&self) {
        print!("\x1B[2J\x1B[H"); // clear screen

        for r in (0..HEIGHT).rev() {
            let mut line = String::with_capacity(WIDTH * 2 - 1);

            for c in 0..WIDTH {
                line.push('|');
                line.push(' ');

                let bit = 1u64 << (c * (HEIGHT + 1) + r);

                let chr = if (self.board.both & bit) == 0 {
                    '\u{3000}'
                } else if (self.board.single & bit) != 0 {
                    self.current_player.symbol()
                } else {
                    self.current_player.other().symbol()
                };

                line.push(chr);
                line.push(' ');
            }

            println!("{}|", line);
        }

        let bottom_line = (0..WIDTH)
            .map(|i| {
                format!(
                    " {} +",
                    std::char::from_u32(0xFF10 + i as u32).unwrap_or('?')
                )
            })
            .collect::<Vec<_>>()
            .join("");

        println!("+{}", bottom_line);

        if let Some(w) = self.get_winner() {
            println!(" Winner: {}", w.symbol());
        }

        let _ = io::stdout().flush();
    }

    fn get_game_state_score(&self, _player: &Self::PlayerMask) -> f32 {
        fn count_sequences(p: u64) -> (u32, u32) {
            let mut n_two = 0u32;
            let mut n_three = 0u32;

            // vertical (↑)
            let m = p & (p >> 1);
            n_two += m.count_ones();
            let n = m & (m >> 1);
            n_three += n.count_ones();

            // horizontal (→) : shift by (7)
            let m = p & (p >> (HEIGHT + 1));
            n_two += m.count_ones();
            let n = m & (m >> (HEIGHT + 1));
            n_three += n.count_ones();

            // diagonal (↗) : shift by (8)
            let m = p & (p >> (HEIGHT + 2));
            n_two += m.count_ones();
            let n = m & (m >> (HEIGHT + 2));
            n_three += n.count_ones();

            // Diagonal ↘ (shift by 6)
            let m = p & (p >> HEIGHT);
            n_two += m.count_ones();
            let n = m & (m >> HEIGHT);
            n_three += n.count_ones();
            (n_two, n_three)
        }

        let (n_two, n_three) = count_sequences(self.board.single);
        let (n_two_other, n_three_other) = count_sequences(self.board.both ^ self.board.single);

        const TWO_WEIGHT: f32 = 1.0 / 3.0;
        const THREE_WEIGHT: f32 = 2.0 / 3.0;

        fn normalized_diff(player: u32, other: u32) -> f32 {
            let total = player + other;
            if total == 0 {
                0.0
            } else {
                (player as f32 - other as f32) / total as f32
            }
        }

        let n_two_score = normalized_diff(n_two, n_two_other);
        let n_three_score = normalized_diff(n_three, n_three_other);

        let combined_score = n_two_score * TWO_WEIGHT + n_three_score * THREE_WEIGHT;
        combined_score / 2.0
    }

    fn get_game_state_hash(&self) -> u64 {
        const fn bit_to_cell_id(bit_index: usize) -> Option<usize> {
            let col = bit_index / (HEIGHT + 1);
            let row = bit_index % (HEIGHT + 1);

            if col < WIDTH && row < HEIGHT {
                Some(col * HEIGHT + row)
            } else {
                None // sentinel row or out of bounds
            }
        }

        let mut side_to_check = self.current_player;
        let mut h = 0u64;

        let mut a = self.board.single;
        while a != 0 {
            let b = a & (!a + 1);
            let idx = b.trailing_zeros() as usize;

            if let Some(cell) = bit_to_cell_id(idx) {
                h ^= ZOBRIST[side_to_check.index()][cell];
            }
            a ^= b;
        }

        side_to_check = side_to_check.other();

        let mut o = self.board.both ^ self.board.single;
        while o != 0 {
            let b = o & (!o + 1);
            let idx = b.trailing_zeros() as usize;

            if let Some(cell) = bit_to_cell_id(idx) {
                h ^= ZOBRIST[side_to_check.index()][cell];
            }
            o ^= b;
        }

        if matches!(side_to_check, PlayerMask::Red) {
            h ^= ZOBRIST_SIDE_TO_MOVE;
        }

        h
    }
}

#[cfg(test)]
mod tests {
    use crate::games::connect_four::*;

    #[test]
    fn test_connect_four_initial_state() {
        let game = ConnectFour::new();

        assert_eq!(game.board.both, 0);
        assert_eq!(game.board.single, 0);

        assert_eq!(game.current_player, PlayerMask::Red);
        assert_eq!(game.get_current_player_index(), 0);
    }
}
