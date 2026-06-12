use crate::search::Strategy;
use crate::{Game, GameError};
use ahash::AHashMap;

#[derive(PartialEq)]
pub enum TranspositionType {
    Exact,
    UpperBound,
    LowerBound,
}

/// Depth-limited negamax with alpha-beta pruning and a transposition table.
///
/// All scores are from the perspective of the player to move at the node
/// being evaluated, so the game's `evaluate` must be zero-sum.
pub struct Minimax<G: Game> {
    pub max_depth: usize,
    pub transpositions: AHashMap<u64, (f32, usize, TranspositionType)>,
    pub move_score: f32,
    /// One reusable move buffer per ply, so the hot loop neither clones the
    /// game nor allocates.
    move_buffers: Vec<Vec<G::Move>>,
    n_cached_transposition: u64,
    n_eval_terminal_state: u64,
    n_eval_game_state: u64,
    compute_time_ns: u128,
}

impl<G: Game> Minimax<G> {
    pub fn new(max_depth: usize) -> Self {
        Minimax {
            max_depth,
            transpositions: AHashMap::new(),
            move_score: 0.0,
            move_buffers: Vec::new(),
            n_cached_transposition: 0,
            n_eval_terminal_state: 0,
            n_eval_game_state: 0,
            compute_time_ns: 0,
        }
    }

    pub fn get_move(&mut self, game: &mut G) -> Result<G::Move, GameError> {
        self.transpositions.clear();
        self.n_cached_transposition = 0;
        self.n_eval_terminal_state = 0;
        self.n_eval_game_state = 0;

        let mut alpha = f32::MIN;
        let beta = f32::MAX;
        let mut best: Option<(f32, G::Move)> = None;

        let start = std::time::Instant::now();
        let moves: Vec<G::Move> = game.get_possible_moves().collect();
        for mv in moves {
            game.apply_move(mv);
            let score = -self.negamax(game, 1, -beta, -alpha);
            game.undo_move(mv);

            if best.is_none_or(|(best_score, _)| score > best_score) {
                best = Some((score, mv));
            }
            alpha = alpha.max(score);
        }
        self.compute_time_ns = start.elapsed().as_nanos();

        if let Some((score, mv)) = best {
            self.move_score = score;
            Ok(mv)
        } else {
            Err(GameError::NoMovesAvailable)
        }
    }

    fn negamax(&mut self, game: &mut G, depth: usize, mut alpha: f32, mut beta: f32) -> f32 {
        if let Some(score) = self.terminal_score(game, depth) {
            self.n_eval_terminal_state += 1;
            return score;
        }

        if depth > self.max_depth {
            self.n_eval_game_state += 1;
            return game.evaluate();
        }

        let game_state_hash = game.get_game_state_hash();
        if let Some((score_seen, depth_seen, transposition_type)) =
            self.transpositions.get(&game_state_hash)
        {
            // The search horizon is an absolute ply count, so an entry is
            // only as deep as ours if it was searched from the same or an
            // earlier ply.
            if *depth_seen <= depth {
                self.n_cached_transposition += 1;

                match transposition_type {
                    TranspositionType::Exact => return *score_seen,
                    TranspositionType::LowerBound => alpha = alpha.max(*score_seen),
                    TranspositionType::UpperBound => beta = beta.min(*score_seen),
                }

                if alpha >= beta {
                    return *score_seen;
                }
            }
        }

        let alpha_original = alpha;

        // Take this ply's buffer out of `self` so it can be iterated while
        // `self.negamax` recurses (each deeper ply uses its own buffer).
        if self.move_buffers.len() <= depth {
            self.move_buffers.resize_with(depth + 1, Vec::new);
        }
        let mut moves = std::mem::take(&mut self.move_buffers[depth]);
        moves.clear();
        moves.extend(game.get_possible_moves());

        let mut best_score = f32::MIN;
        for &mv in &moves {
            game.apply_move(mv);
            let score = -self.negamax(game, depth + 1, -beta, -alpha);
            game.undo_move(mv);

            best_score = best_score.max(score);
            alpha = alpha.max(best_score);
            if alpha >= beta {
                break;
            }
        }

        self.move_buffers[depth] = moves;

        let transposition_type = if best_score <= alpha_original {
            TranspositionType::UpperBound
        } else if best_score >= beta {
            TranspositionType::LowerBound
        } else {
            TranspositionType::Exact
        };

        self.transpositions
            .insert(game_state_hash, (best_score, depth, transposition_type));

        best_score
    }

    fn terminal_score(&self, game: &G, depth: usize) -> Option<f32> {
        #[allow(clippy::cast_precision_loss)]
        let depth = depth as f32;

        if game.get_winner().is_some() {
            // Whoever won made the previous move, so from the player to
            // move's perspective the position is lost; scaling by depth makes
            // near results outweigh distant ones.
            return Some(-1.0 / depth);
        }

        if game.is_finished() {
            return Some(0.0);
        }

        None
    }
}

impl<G: Game> Strategy<G> for Minimax<G> {
    fn compute_move(&mut self, game: &G) -> Result<G::Move, GameError> {
        self.get_move(&mut game.clone())
    }
}
