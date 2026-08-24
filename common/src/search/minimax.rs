use crate::search::Strategy;
use crate::{Game, GameError};
use ahash::AHashMap;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

/// How often (in nodes) the search checks the wall clock and redraws the
/// status bar; a power of two so the check compiles to a mask.
const CLOCK_CHECK_INTERVAL: u64 = 4096;

/// Root moves scoring within this of the best are treated as ties and
/// sampled between, so equal positions don't replay the same game every time.
const TIE_EPS: f32 = 1e-6;

const LABEL: &str = "\x1b[1;36mminimax\x1b[0m";
const SEP: &str = " \x1b[2m|\x1b[0m ";

#[derive(PartialEq)]
pub enum TranspositionType {
    Exact,
    UpperBound,
    LowerBound,
}

/// Negamax with alpha-beta pruning and a transposition table.
///
/// With a [`time_budget`](Minimax::time_budget) the search iteratively deepens
/// up to `max_depth`, keeping the last fully completed depth when the clock
/// runs out; without one it searches straight to `max_depth`.
///
/// All scores are from the perspective of the player to move at the node
/// being evaluated, so the game's `evaluate` must be zero-sum.
pub struct Minimax<G: Game> {
    /// Hard cap on search depth. Also the deepest iteration when iteratively
    /// deepening under a time budget.
    pub max_depth: usize,
    /// Optional wall-clock budget per move; `None` searches straight to
    /// `max_depth`. Set via [`Minimax::with_time_budget`].
    pub time_budget: Option<Duration>,
    /// TT value: score, the remaining depth it was searched to (draft), and
    /// what kind of bound the score is. Keyed on the position hash, so draft
    /// is what lets iterative deepening reuse only entries searched deep
    /// enough for the current iteration.
    pub transpositions: AHashMap<u64, (f32, usize, TranspositionType)>,
    pub move_score: f32,
    /// Live search statistics on stderr while computing (off by default);
    /// toggle directly or via [`Minimax::with_status_bar`].
    pub status_bar: bool,
    /// One reusable move buffer per ply, so the hot loop neither clones the
    /// game nor allocates.
    move_buffers: Vec<Vec<G::Move>>,
    /// Horizon of the iteration currently running (`<= max_depth`).
    depth_limit: usize,
    /// Deepest iteration that ran to completion this move, for reporting.
    completed_depth: usize,
    /// When the budget expires; `None` when there is no budget.
    deadline: Option<Instant>,
    /// Set once the deadline passes so the in-flight iteration unwinds fast;
    /// its partial result is then discarded.
    aborted: bool,
    n_nodes: u64,
    n_pruned: u64,
    n_cached_transposition: u64,
    n_eval_terminal_state: u64,
    n_eval_game_state: u64,
    compute_time_ns: u128,
    search_start: Instant,
    root_done: usize,
    root_total: usize,
}

impl<G: Game> Minimax<G> {
    pub fn new(max_depth: usize) -> Self {
        Minimax {
            max_depth,
            time_budget: None,
            transpositions: AHashMap::new(),
            move_score: 0.0,
            status_bar: false,
            move_buffers: Vec::new(),
            depth_limit: max_depth,
            completed_depth: 0,
            deadline: None,
            aborted: false,
            n_nodes: 0,
            n_pruned: 0,
            n_cached_transposition: 0,
            n_eval_terminal_state: 0,
            n_eval_game_state: 0,
            compute_time_ns: 0,
            search_start: Instant::now(),
            root_done: 0,
            root_total: 0,
        }
    }

    #[must_use]
    pub fn with_status_bar(mut self) -> Self {
        self.status_bar = true;
        self
    }

    /// Iteratively deepen up to `max_depth`, stopping once `budget` elapses
    /// and returning the best move from the last fully completed depth.
    #[must_use]
    pub fn with_time_budget(mut self, budget: Duration) -> Self {
        self.time_budget = Some(budget);
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
        self.completed_depth = 0;
        self.aborted = false;
        self.search_start = Instant::now();
        self.deadline = self.time_budget.map(|b| self.search_start + b);

        let moves: Vec<G::Move> = game.get_possible_moves().collect();
        self.root_done = 0;
        self.root_total = moves.len();

        // Seed with any legal move so we always have something to return, even
        // if the very first iteration is cut off before it completes.
        let mut best = *moves
            .choose(&mut rand::thread_rng())
            .ok_or(GameError::NoMovesAvailable)?;

        // No budget: one search straight to max_depth. With a budget: deepen
        // one ply at a time, keeping the last iteration that finished in full
        // and discarding whichever one the clock interrupts.
        let start_depth = if self.deadline.is_some() {
            1
        } else {
            self.max_depth
        };
        for limit in start_depth..=self.max_depth {
            self.depth_limit = limit;
            self.draw_status(0);
            match self.search_root(game, &moves) {
                Some((score, mv)) => {
                    best = mv;
                    self.move_score = score;
                    self.completed_depth = limit;
                }
                // Aborted mid-iteration: its scores are unreliable, so keep the
                // previous depth's move and stop deepening.
                None => break,
            }
        }

        self.compute_time_ns = self.search_start.elapsed().as_nanos();
        self.clear_status();
        Ok(best)
    }

    /// Search every root move at the current `depth_limit` and sample uniformly
    /// among the equally-best. Returns `None` if the clock aborted the pass.
    ///
    /// Unlike the narrowing alpha used deeper, every root move gets a full
    /// window: raising alpha across the root loop lets alpha-beta return a mere
    /// bound for later moves, which would corrupt the tie set. Deeper cutoffs
    /// still narrow within each child.
    fn search_root(&mut self, game: &mut G, moves: &[G::Move]) -> Option<(f32, G::Move)> {
        let alpha = f32::MIN;
        let beta = f32::MAX;

        self.root_done = 0;
        let mut best_score = f32::MIN;
        let mut scored: Vec<(f32, G::Move)> = Vec::with_capacity(moves.len());
        for &mv in moves {
            game.apply_move(mv);
            let score = -self.negamax(game, 1, -beta, -alpha);
            game.undo_move(mv);

            if self.aborted {
                return None;
            }

            best_score = best_score.max(score);
            self.move_score = best_score;
            scored.push((score, mv));

            self.root_done += 1;
            self.draw_status(0);
        }

        scored
            .iter()
            .filter(|(score, _)| best_score - score <= TIE_EPS)
            .map(|(_, mv)| *mv)
            .collect::<Vec<_>>()
            .choose(&mut rand::thread_rng())
            .copied()
            .map(|mv| (best_score, mv))
    }

    fn negamax(&mut self, game: &mut G, depth: usize, mut alpha: f32, mut beta: f32) -> f32 {
        self.n_nodes += 1;
        if self.n_nodes.is_multiple_of(CLOCK_CHECK_INTERVAL) {
            if self.deadline.is_some_and(|dl| Instant::now() >= dl) {
                self.aborted = true;
            }
            if self.status_bar && self.n_nodes.is_multiple_of(65_536) {
                self.draw_status(depth);
            }
        }
        // Bail straight back to the root, which discards this iteration; any
        // value works since it is never used.
        if self.aborted {
            return 0.0;
        }

        if let Some(score) = self.terminal_score(game, depth) {
            self.n_eval_terminal_state += 1;
            return score;
        }

        if depth > self.depth_limit {
            self.n_eval_game_state += 1;
            return game.evaluate();
        }

        // Plies left below this node at the current horizon. Keyed on the
        // position (not the ply), so this is what makes a deeper iteration
        // reuse only entries themselves searched deep enough.
        let draft = self.depth_limit - depth;

        let game_state_hash = game.get_game_state_hash();
        if let Some((score_seen, draft_seen, transposition_type)) =
            self.transpositions.get(&game_state_hash)
        {
            if *draft_seen >= draft {
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

            if self.aborted {
                break;
            }

            best_score = best_score.max(score);
            alpha = alpha.max(best_score);
            if alpha >= beta {
                self.n_pruned += 1;
                break;
            }
        }

        self.move_buffers[depth] = moves;

        // A cut-off search never finished this node, so don't cache its
        // half-formed score for a later iteration to trust.
        if self.aborted {
            return 0.0;
        }

        let transposition_type = if best_score <= alpha_original {
            TranspositionType::UpperBound
        } else if best_score >= beta {
            TranspositionType::LowerBound
        } else {
            TranspositionType::Exact
        };

        self.transpositions
            .insert(game_state_hash, (best_score, draft, transposition_type));

        best_score
    }

    /// Redraw the status line in place on stderr. `ply` is where the search
    /// happens to be right now (0 = back at the root between moves).
    fn draw_status(&self, ply: usize) {
        if !self.status_bar {
            return;
        }

        // Leaf evaluations happen one ply past the horizon — clamp so the
        // line never reads "ply 16/15". The horizon is the current iteration's
        // limit, which climbs as iterative deepening progresses.
        let ply = ply.min(self.depth_limit);
        let elapsed = self.search_start.elapsed().as_secs_f64();

        eprint!(
            "\r\x1b[2K {LABEL}  {} {ply:>2}/{:<2}{SEP}{} {:>2}/{:<2}{SEP}{} {}{SEP}{}",
            dim("ply"),
            self.depth_limit,
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
            self.completed_depth,
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
