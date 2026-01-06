use crate::strategy::Strategy;
use crate::strategy::common::prompt_user_move;

pub mod games;
pub mod strategy;

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

pub trait Game {
    type PlayerMask;
    type Move: Copy + Clone;

    // TODO: Could be done better, by using the type PlayerMask of trait
    fn get_current_player_index(&self) -> usize;

    fn get_current_player_symbol(&self) -> char;

    fn get_current_player(&self) -> Self::PlayerMask;

    // Actual game
    fn apply_move(&mut self, chosen_move: Self::Move);

    fn undo_move(&mut self, chosen_move: Self::Move);

    fn get_possible_moves(&self) -> impl Iterator<Item = Self::Move>;

    fn is_finished(&self) -> bool;

    fn get_winner(&self) -> Option<Self::PlayerMask>;

    fn render(&self);

    // Additional information for strategies, could be a dedicated trait?
    fn get_game_state_score(&self, player: &Self::PlayerMask) -> f32;

    fn get_game_state_hash(&self) -> u64;
}

pub enum PlayerType<S: Strategy> {
    Human,
    AI(S),
}

pub struct Competition<G: Game, S: Strategy> {
    pub game: G,
    first_player: PlayerType<S>,
    second_player: PlayerType<S>,
    pub turn: u32,
}

impl<G: Game<Move = usize>, S: Strategy> Competition<G, S>
where
    G: Clone,
    <G as Game>::PlayerMask: Eq,
{
    pub fn new(game: G, first_player: PlayerType<S>, second_player: PlayerType<S>) -> Self {
        Competition {
            game,
            first_player,
            second_player,
            turn: 0,
        }
    }

    pub fn start(&mut self, render_game: bool) -> Result<(), GameError> {
        if render_game {
            self.game.render();
        }

        while !self.game.is_finished() {
            let player = self.determine_player();
            let chosen_move = self.get_move_for_player(player)?;
            self.game.apply_move(chosen_move);

            self.turn += 1;

            if render_game {
                self.game.render();
            }
        }
        Ok(())
    }

    pub fn determine_player(&self) -> &PlayerType<S> {
        [&self.first_player, &self.second_player][self.game.get_current_player_index()]
    }

    pub fn get_move_for_player(&self, player: &PlayerType<S>) -> Result<usize, GameError> {
        match player {
            PlayerType::AI(strategy) => strategy.compute_move(&self.game),
            PlayerType::Human => Ok(prompt_user_move(&self.game)),
        }
    }
}
