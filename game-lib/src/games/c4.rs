use crate::Game;

const WIDHT: usize = 7;
const HEIGHT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerMask {
    Red,
    Yellow,
}

impl PlayerMask {
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
            PlayerMask::Red => '🔴',
            PlayerMask::Yellow => '🟡',
        }
    }
}

pub struct Board {
    both: u64,
    single: u64,
}

impl Board {
    fn new() -> Self {
        Self { both: 0, single: 0 }
    }
}

pub struct ConnectFour {
    board: Board,
    current_player: PlayerMask,
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

    fn get_possible_moves(&self) -> impl Iterator<Item = usize> {
        (0..=8).filter(|&_| true)
    }

    fn apply_move(&mut self, chosen_move: usize) {
        self.current_player = self.current_player.other();
    }

    fn undo_move(&mut self, chosen_move: usize) {}

    fn get_current_player_index(&self) -> usize {
        self.current_player.index()
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_winner(&self) -> Option<PlayerMask> {
        None
    }

    fn render(&self) {
        unimplemented!("Rendering for connect-four not implemented yet")
    }
}

#[cfg(test)]
mod tests {
    use crate::games::c4::*;
    use crate::Game;

    #[test]
    fn test_connect_four_initial_state() {
        let game = ConnectFour::new();

        assert_eq!(game.board.both, 0);
        assert_eq!(game.board.single, 0);

        assert_eq!(game.current_player, PlayerMask::Red);
        assert_eq!(game.get_current_player_index(), 0);
    }
}
