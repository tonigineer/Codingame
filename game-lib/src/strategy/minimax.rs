use crate::strategy::Strategy;
use crate::{Game, GameError};
use ahash::AHashMap;

#[derive(PartialEq)]
pub enum TranspositionType {
    Exact,
    UpperBound,
    LowerBound,
}

pub struct Minimax {
    pub max_depth: u64,
    pub transpositions: AHashMap<u64, (f32, u64, TranspositionType)>,
    pub move_score: f32,
    n_cached_transposition: u64,
    n_eval_terminal_state: u64,
    n_eval_game_state: u64,
    compute_time_ns: u128,
}

impl Minimax {
    pub fn new(max_depth: u64) -> Self {
        Minimax {
            max_depth,
            transpositions: AHashMap::new(),
            move_score: 0.0,
            n_cached_transposition: 0,
            n_eval_terminal_state: 0,
            n_eval_game_state: 0,
            compute_time_ns: 0,
        }
    }

    pub fn get_move<G>(&mut self, game: &mut G, side: G::PlayerMask) -> Result<G::Move, GameError>
    where
        G: Game + Clone,
        G::Move: Clone,
        <G as Game>::PlayerMask: Eq,
    {
        let mut best_score: Option<(f32, G::Move)> = None;
        let alpha = f32::MIN;
        let beta = f32::MAX;

        self.transpositions.clear();
        self.n_cached_transposition = 0;
        self.n_eval_terminal_state = 0;
        self.n_eval_game_state = 0;

        let start = std::time::Instant::now();
        for mv in game.get_possible_moves() {
            let mut next_game = game.clone();
            next_game.apply_move(mv);

            let score = self.minimax(&mut next_game, &side, 1, alpha, beta);

            best_score = Some(match best_score {
                None => (score, mv),
                Some((best_score, best_mv)) => {
                    if score > best_score {
                        (score, mv)
                    } else {
                        (best_score, best_mv)
                    }
                }
            });
        }

        self.compute_time_ns = start.elapsed().as_nanos();

        if let Some((score, mv)) = best_score {
            self.move_score = score;
            Ok(mv)
        } else {
            Err(GameError::NoMovesAvailable)
        }
    }

    fn game_state_score<G>(&mut self, game: &G, side: &G::PlayerMask) -> f32
    where
        G: Game + Clone,
    {
        game.get_game_state_score(side)
    }

    fn terminal_score<G>(&mut self, game: &G, my_side: &G::PlayerMask, depth: u64) -> Option<f32>
    where
        G: Game + Clone,
        <G as Game>::PlayerMask: Eq,
    {
        if let Some(winner) = game.get_winner() {
            if winner == *my_side {
                return Some(1.0 / (depth as f32));
            } else {
                return Some(-1.0 / (depth as f32));
            }
        }

        if game.is_finished() {
            return Some(0.0);
        }

        None
    }

    fn minimax<G>(
        &mut self,
        game: &mut G,
        my_side: &G::PlayerMask,
        depth: u64,
        mut alpha: f32,
        mut beta: f32,
    ) -> f32
    where
        G: Game + Clone,
        <G as Game>::PlayerMask: Eq,
    {
        if depth > self.max_depth {
            self.n_eval_game_state += 1;
            let score = self.game_state_score(game, my_side);
            return score;
        }

        if let Some(score) = self.terminal_score(game, my_side, depth) {
            self.n_eval_terminal_state += 1;
            return score;
        }

        let game_state_hash = game.get_game_state_hash();
        if let Some((score_seen, depth_seen, transposition_type)) =
            self.transpositions.get(&game_state_hash)
        {
            if *depth_seen >= depth {
                self.n_cached_transposition += 1;

                match transposition_type {
                    TranspositionType::Exact => {
                        return *score_seen;
                    }
                    TranspositionType::LowerBound => {
                        alpha = alpha.max(*score_seen);
                    }
                    TranspositionType::UpperBound => {
                        beta = beta.min(*score_seen);
                    }
                }

                if alpha >= beta {
                    return *score_seen;
                }
            }
        }

        let maximizing = *my_side == game.get_current_player();

        let mut best_score = match maximizing {
            true => f32::MIN,
            false => f32::MAX,
        };

        for mv in game.clone().get_possible_moves() {
            game.apply_move(mv);
            let score = self.minimax(game, my_side, depth + 1, alpha, beta);
            game.undo_move(mv);

            if maximizing {
                best_score = best_score.max(score);
                if best_score >= beta {
                    break;
                }
                alpha = alpha.max(best_score);
            } else {
                best_score = best_score.min(score);
                if best_score <= alpha {
                    break;
                }
                beta = beta.min(best_score);
            };
        }

        let transposition_type = match (best_score <= alpha, best_score >= beta) {
            (true, _) => TranspositionType::UpperBound,
            (_, true) => TranspositionType::LowerBound,
            _ => TranspositionType::Exact,
        };

        self.transpositions
            .insert(game_state_hash, (best_score, depth, transposition_type));

        best_score
    }
}

impl Strategy for Minimax {
    fn compute_move<G>(&mut self, game: &G) -> Result<G::Move, GameError>
    where
        G: Game + Clone,
        G::Move: Clone,
        <G as Game>::PlayerMask: Eq,
    {
        let mut new_game = game.clone();
        let side = game.get_current_player();

        self.get_move(&mut new_game, side)
    }
}
