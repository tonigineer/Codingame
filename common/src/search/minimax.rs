use crate::search::Strategy;
use crate::{Game, GameError};
use ahash::AHashMap;

const LABEL: &str = "\x1b[1;36mminimax\x1b[0m";
const SEP: &str = " \x1b[2m|\x1b[0m ";

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
    /// Live search statistics on stderr while computing (off by default);
    /// toggle directly or via [`Minimax::with_status_bar`].
    pub status_bar: bool,
    /// One reusable move buffer per ply, so the hot loop neither clones the
    /// game nor allocates.
    move_buffers: Vec<Vec<G::Move>>,
    n_nodes: u64,
    n_pruned: u64,
    n_cached_transposition: u64,
    n_eval_terminal_state: u64,
    n_eval_game_state: u64,
    compute_time_ns: u128,
    search_start: std::time::Instant,
    root_done: usize,
    root_total: usize,
}

impl<G: Game> Minimax<G> {
    pub fn new(max_depth: usize) -> Self {
        Minimax {
            max_depth,
            transpositions: AHashMap::new(),
            move_score: 0.0,
            status_bar: false,
            move_buffers: Vec::new(),
            n_nodes: 0,
            n_pruned: 0,
            n_cached_transposition: 0,
            n_eval_terminal_state: 0,
            n_eval_game_state: 0,
            compute_time_ns: 0,
            search_start: std::time::Instant::now(),
            root_done: 0,
            root_total: 0,
        }
    }

    #[must_use]
    pub fn with_status_bar(mut self) -> Self {
        self.status_bar = true;
        self
    }

    pub fn get_move(&mut self, game: &mut G) -> Result<G::Move, GameError> {
        self.transpositions.clear();
        self.n_nodes = 0;
        self.n_pruned = 0;
        self.n_cached_transposition = 0;
        self.n_eval_terminal_state = 0;
        self.n_eval_game_state = 0;
        self.move_score = 0.0;
        self.search_start = std::time::Instant::now();

        let mut alpha = f32::MIN;
        let beta = f32::MAX;
        let mut best: Option<(f32, G::Move)> = None;

        let moves: Vec<G::Move> = game.get_possible_moves().collect();
        self.root_done = 0;
        self.root_total = moves.len();
        self.draw_status(0);

        for mv in moves {
            game.apply_move(mv);
            let score = -self.negamax(game, 1, -beta, -alpha);
            game.undo_move(mv);

            if best.is_none_or(|(best_score, _)| score > best_score) {
                best = Some((score, mv));
                self.move_score = score;
            }
            alpha = alpha.max(score);

            self.root_done += 1;
            self.draw_status(0);
        }
        self.compute_time_ns = self.search_start.elapsed().as_nanos();
        self.clear_status();

        if let Some((score, mv)) = best {
            self.move_score = score;
            Ok(mv)
        } else {
            Err(GameError::NoMovesAvailable)
        }
    }

    fn negamax(&mut self, game: &mut G, depth: usize, mut alpha: f32, mut beta: f32) -> f32 {
        self.n_nodes += 1;
        if self.status_bar && self.n_nodes.is_multiple_of(65_536) {
            self.draw_status(depth);
        }

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
                self.n_pruned += 1;
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

    /// Redraw the status line in place on stderr. `ply` is where the search
    /// happens to be right now (0 = back at the root between moves).
    fn draw_status(&self, ply: usize) {
        if !self.status_bar {
            return;
        }

        // Leaf evaluations happen one ply past the horizon — clamp so the
        // line never reads "ply 16/15".
        let ply = ply.min(self.max_depth);
        let elapsed = self.search_start.elapsed().as_secs_f64();

        eprint!(
            "\r\x1b[2K {LABEL}  {} {ply:>2}/{:<2}{SEP}{} {:>2}/{:<2}{SEP}{} {}{SEP}{}",
            dim("ply"),
            self.max_depth,
            dim("root"),
            self.root_done,
            self.root_total,
            dim("best"),
            score_colored(self.move_score),
            self.stats_segment(elapsed),
        );
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    /// The stats tail shared by the live bar and the post-move summary.
    fn stats_segment(&self, elapsed: f64) -> String {
        #[allow(clippy::cast_precision_loss)]
        let nps = self.n_nodes as f64 / elapsed.max(1e-9);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let nps = nps as u64;

        format!(
            "{} {:>6}{SEP}{} {:>6}{SEP}{} {:>6}{SEP}{:>6} {}{SEP}{elapsed:>5.1}{}",
            dim("nodes"),
            fmt_count(self.n_nodes),
            dim("prunes"),
            fmt_count(self.n_pruned),
            dim("tt hits"),
            fmt_count(self.n_cached_transposition),
            fmt_count(nps),
            dim("n/s"),
            dim("s"),
        )
    }

    /// The board render that follows would clear the screen anyway in
    /// terminal play; headless callers shouldn't keep a half-line either.
    fn clear_status(&self) {
        if !self.status_bar {
            return;
        }

        eprint!("\r\x1b[2K");
        let _ = std::io::Write::flush(&mut std::io::stderr());
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

    /// The last search's summary; `Competition` keeps it visible under the
    /// board after the live bar is gone.
    fn status_line(&self) -> Option<String> {
        if !self.status_bar || self.compute_time_ns == 0 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        let elapsed = self.compute_time_ns as f64 / 1e9;

        Some(format!(
            " {LABEL}  {} {}{SEP}{} {}{SEP}{}",
            dim("score"),
            score_colored(self.move_score),
            dim("depth"),
            self.max_depth,
            self.stats_segment(elapsed),
        ))
    }
}

fn dim(s: &str) -> String {
    format!("\x1b[2m{s}\x1b[0m")
}

/// Green when the player the search just moved for is ahead, red when behind.
fn score_colored(score: f32) -> String {
    let s = format!("{score:+.3}");
    if score > 0.0 {
        format!("\x1b[32m{s}\x1b[0m")
    } else if score < 0.0 {
        format!("\x1b[31m{s}\x1b[0m")
    } else {
        s
    }
}

/// Humanize a counter for the status line: `8123`, `45.1k`, `2.3M`.
fn fmt_count(n: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    match n {
        0..=9_999 => n.to_string(),
        10_000..=999_999 => format!("{:.1}k", n as f64 / 1e3),
        _ => format!("{:.1}M", n as f64 / 1e6),
    }
}
