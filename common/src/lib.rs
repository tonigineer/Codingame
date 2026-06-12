use crate::search::Strategy;

pub mod search;

#[derive(Debug)]
pub enum GameError {
    InvalidMove(String),
    NoMovesAvailable,
    ParseError(String),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InvalidMove(msg) => write!(f, "Invalid move: {}", msg),
            GameError::NoMovesAvailable => write!(f, "No moves available"),
            GameError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for GameError {}

pub trait Player {
    fn other(&self) -> Self;
    fn index(&self) -> usize;
    fn symbol(&self) -> char;
}

pub trait Game: Clone {
    type PlayerMask: Player + Eq;
    type Move: Copy;

    fn get_current_player(&self) -> Self::PlayerMask;

    fn get_current_player_index(&self) -> usize {
        self.get_current_player().index()
    }

    fn get_current_player_symbol(&self) -> char {
        self.get_current_player().symbol()
    }

    fn apply_move(&mut self, chosen_move: Self::Move);

    fn undo_move(&mut self, chosen_move: Self::Move);

    fn get_possible_moves(&self) -> impl Iterator<Item = Self::Move>;

    fn is_finished(&self) -> bool;

    fn get_winner(&self) -> Option<Self::PlayerMask>;

    fn render(&self);

    /// Heuristic score of this position from the perspective of the player
    /// to move. Must be zero-sum: a position scoring `s` for one player
    /// scores `-s` for the other.
    fn evaluate(&self) -> f32;

    fn get_game_state_hash(&self) -> u64;
}

pub struct Competition<G: Game> {
    pub game: G,
    players: [Box<dyn Strategy<G>>; 2],
    pub turn: u32,
}

impl<G: Game> Competition<G> {
    pub fn new(
        game: G,
        first_player: impl Strategy<G> + 'static,
        second_player: impl Strategy<G> + 'static,
    ) -> Self {
        Competition {
            game,
            players: [Box::new(first_player), Box::new(second_player)],
            turn: 0,
        }
    }

    pub fn start(&mut self, render_game: bool) -> Result<(), GameError> {
        if render_game {
            self.game.render();
        }

        while !self.game.is_finished() {
            self.play_turn()?;

            if render_game {
                self.game.render();
            }
        }
        Ok(())
    }

    /// Play a single turn: the player to move picks a move and it is applied.
    pub fn play_turn(&mut self) -> Result<(), GameError> {
        let player = &mut self.players[self.game.get_current_player_index()];
        let chosen_move = player.compute_move(&self.game)?;
        self.game.apply_move(chosen_move);
        self.turn += 1;
        Ok(())
    }
}
