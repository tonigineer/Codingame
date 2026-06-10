//! Mid/late-game decision core: rank every troll's options as scored
//! [`Candidate`]s and greedily assign the best one per troll.
//!
//! [`Bot::late_game`] is the entry point. Candidates are produced per troll by
//! role (economy vs. harasser, see [`Role`]), sorted by score, then handed to
//! [`Bot::assign_actions`], which commits at most one action per troll and one
//! worker per tree.

use crate::bot::Bot;
use crate::game::{Action, Game, Side, Troll};
use crate::utils::Position;

/// What a troll is for, which decides how its candidates are generated.
///
/// The lowest-id troll (our slow/weak starter) farms the home grove; every
/// troll trained later roams the map as a harasser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Economy,
    Harasser,
}

/// One scored option for a single troll.
///
/// `score` is the (pre-scaled, integer) ranking key; higher wins. `tree`, when
/// set, marks the cell this action works so [`Bot::assign_actions`] lets only
/// one troll claim it and can rewrite an on-tree `Move` into a `Chop`.
#[derive(Debug)]
pub struct Candidate {
    pub troll_id: i32,
    pub action: Action,
    pub score: i32,
    pub tree: Option<Position>,
}

/// Role of a troll: the lowest-id troll (our slow/weak starter) farms at
/// home; every trained troll is a roaming harasser.
fn role(troll: &Troll, game: &Game) -> Role {
    let first = game.trolls(Side::Me).into_iter().map(|t| t.id).min();
    if Some(troll.id) == first {
        Role::Economy
    } else {
        Role::Harasser
    }
}

impl Bot {
    /// Decide this turn's actions for every troll once past the opening.
    ///
    /// Collects each troll's scored candidates, sorts them best-first, and
    /// greedily assigns them. Bails out early while still a single troll in
    /// the opening, both when it is mid-move (anything other than a pending
    /// `Train`) and when the opening deliberately produced no action (e.g.
    /// waiting on a tree to fruit) — otherwise the economy playbook would
    /// interject for one turn and undo the opening's plan (the t1 pick → t2
    /// drop churn). The one exception is a pending `Train`: the shack trains
    /// while the troll still acts, so the economy may fill that turn.
    pub fn late_game(&mut self, game: &Game) {
        let trolls: Vec<&Troll> = game.trolls(Side::Me);

        // Skip if still in early game and moving or deliberately waiting.
        if trolls.len() <= 1
            && (self.actions.is_empty()
                || self
                    .actions
                    .iter()
                    .any(|a| !matches!(a, Action::Train(_, _, _, _))))
        {
            return;
        }

        let mut actions = self.collect_actions(&trolls, game);
        actions.sort_by_key(|a| -a.score);

        self.assign_actions(actions, &trolls);
    }

    /// Build the scored candidate list, by role, for all trolls.
    fn collect_actions(&self, trolls: &[&Troll], game: &Game) -> Vec<Candidate> {
        let mut actions = Vec::new();
        for troll in trolls {
            // On the 3rd-troll mission BOTH trolls get mission candidates
            // first — each gathers only what it can (the harasser has no
            // harvest power, so fruits fall to the economy troll, iron to the
            // harasser). Mission scores dominate; the role candidates below
            // remain as fallback so nobody idles.
            if self.third_troll_mission {
                Bot::train_gather_candidates(troll, game, &mut actions);
            }
            match role(troll, game) {
                Role::Economy => Bot::economy_candidates(troll, game, &mut actions),
                Role::Harasser => Bot::harasser_candidates(troll, game, &mut actions),
            }
        }
        actions
    }

    /// Greedily commit the highest-scoring candidates.
    ///
    /// Walks `actions` in the order given (the caller pre-sorts them best-first)
    /// and accepts each one only if its troll has not already acted and its
    /// `tree`, if any, has not already been claimed. A tree-targeted `Move`
    /// whose troll already stands on the tree is rewritten into a `Chop`;
    /// terminal actions are committed as-is. Planting also records the cell in
    /// [`Bot::planted_cells`] so the harasser leaves the home grove alone.
    ///
    /// # Examples
    ///
    /// ```
    /// use trollfarm::bot::{Bot, Candidate};
    /// use trollfarm::game::{Action, Game, Side};
    /// use trollfarm::utils::Position;
    ///
    /// // One of our trolls (id 100) standing on (1,1). Both shacks (0 and 1)
    /// // must be present for the mock to parse.
    /// let input = "\
    /// 3 3
    /// 0..
    /// ...
    /// ..1
    /// 0 0 0 0 0 0
    /// 0 0 0 0 0 0
    /// 0
    /// 1
    /// 100 0 1 1 1 2 1 1 0 0 0 0 0 0";
    /// let game = Game::create_mock(input);
    /// let trolls = game.trolls(Side::Me);
    ///
    /// // Two candidates for the same troll. The first targets the tree the
    /// // troll already stands on with a Move, so it is rewritten to Chop; the
    /// // second is dropped because each troll acts at most once.
    /// let candidates = vec![
    ///     Candidate {
    ///         troll_id: 100,
    ///         action: Action::Move(100, Position::new(1, 1)),
    ///         score: 10,
    ///         tree: Some(Position::new(1, 1)),
    ///     },
    ///     Candidate { troll_id: 100, action: Action::Drop(100), score: 5, tree: None },
    /// ];
    ///
    /// let mut bot = Bot::new();
    /// bot.assign_actions(candidates, &trolls);
    ///
    /// assert_eq!(bot.actions.len(), 1);
    /// assert!(matches!(bot.actions[0], Action::Chop(100)));
    /// ```
    pub fn assign_actions(&mut self, candidates: Vec<Candidate>, trolls: &[&Troll]) {
        let mut busy_trolls: Vec<i32> = Vec::with_capacity(trolls.len());
        let mut claimed_trees: Vec<Position> = Vec::with_capacity(trolls.len());

        for c in candidates.iter().filter(|c| c.troll_id == 0).take(3) {
            eprintln!("{:?}", c);
        }
        for c in candidates.iter().filter(|c| c.troll_id == 2).take(3) {
            eprintln!("{:?}", c);
        }
        for mut candidate in candidates {
            if busy_trolls.contains(&candidate.troll_id) {
                continue;
            }

            if let Some(pos) = candidate.tree {
                if claimed_trees.contains(&pos) {
                    continue;
                }
                let troll = trolls.iter().find(|t| t.id == candidate.troll_id).unwrap();
                if troll.position == pos && matches!(candidate.action, Action::Move(_, _)) {
                    candidate.action = Action::Chop(troll.id);
                }
                claimed_trees.push(pos);
            }

            self.actions.push(candidate.action);
            busy_trolls.push(candidate.troll_id);
        }
    }
}
