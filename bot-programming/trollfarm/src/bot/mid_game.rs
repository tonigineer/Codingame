use crate::bot::Bot;
use crate::game::{Action, Game, Side, Tree, TreeType, Troll};
use crate::utils::Position;
use std::collections::HashMap;

/// Bonus added to a tree's wood-per-turn score based on its fruit type.
/// Encourages picking lemon and banana trees when scores are otherwise close.
const LEMON_BONUS: f32 = 0.01;
const BANANA_BONUS: f32 = 0.005;

/// Scale factor to convert the floating-point score into a sortable integer.
const SCORE_SCALE: f32 = 1000.0;

/// A candidate move for a single troll, ranked by `score`.
///
/// If `tree` is set, the action targets that tree (and the tree gets claimed
/// so two trolls don't chop the same one).
#[derive(Debug)]
struct Candidate {
    troll_id: i32,
    action: Action,
    score: i32,
    tree: Option<Position>,
}

impl Bot {
    /// Mid-game strategy: assign each troll to the most valuable tree
    /// (by wood-per-turn), or send full trolls back to the shack.
    ///
    /// # Strategy
    ///
    /// 1. Generate every possible (troll, tree) action with a score.
    /// 2. Full trolls also get a high-priority "return to shack" action.
    /// 3. Sort all actions by score descending.
    /// 4. Greedily assign: each troll acts once, each tree is claimed once.
    pub fn mid_game(&mut self, game: &Game) {
        let trolls: Vec<&Troll> = game.trolls(Side::Me);

        // Skip if still in early game and moving
        if trolls.len() <= 1
            && self
                .actions
                .iter()
                .any(|a| !matches!(a, Action::Train(_, _, _, _)))
        {
            return;
        }

        let mut actions = Bot::collect_actions(&trolls, game);
        actions.sort_by_key(|a| -a.score);

        self.assign_actions(actions, &trolls);
    }

    /// Build the full list of scored candidate actions for all trolls.
    fn collect_actions(trolls: &[&Troll], game: &Game) -> Vec<Candidate> {
        let mut actions = Vec::new();

        for troll in trolls {
            // Full troll: head back to drop off wood.
            if troll.free_capacity() == 0 {
                actions.push(Bot::return_action(troll, game));
            }

            // Score each unclaimed tree as a target.
            for tree in &game.trees {
                if Bot::tree_occupied_by_others(tree, troll, game) {
                    continue;
                }

                if Bot::tree_would_be_gone_on_arrival(tree, troll, game) {
                    continue;
                }

                actions.push(Bot::score_tree(troll, tree, game));
            }
        }

        actions
    }

    /// Whether another of my trolls is already standing on this tree.
    fn tree_occupied_by_others(tree: &Tree, troll: &Troll, game: &Game) -> bool {
        game.trolls
            .iter()
            .any(|t| t.side == Side::Me && t.id != troll.id && t.position == tree.position)
    }
    /// Whether an opponent troll would chop this tree before my troll arrives.
    fn tree_would_be_gone_on_arrival(tree: &Tree, troll: &Troll, game: &Game) -> bool {
        if let Some(opp_troll) = game
            .trolls
            .iter()
            .find(|t| t.side == Side::Opp && t.position == tree.position)
        {
            let dist = troll
                .dist_map
                .get(&tree.position)
                .map_or(i32::MAX, |(d, _)| *d);

            // Stats and distances are non-negative in practice.
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let travel_turns = (dist as u32).div_ceil(troll.movement_speed as u32) as i32;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let opp_chop_turns = (tree.health as u32).div_ceil(opp_troll.chop_power as u32) as i32;

            return opp_chop_turns <= travel_turns;
        }

        false
    }

    /// Score a tree as wood-per-turn for the given troll.
    ///
    /// ```text
    /// score = min(treeSize, carryRemaining) / (travel + chop + return)
    /// ```
    ///
    /// where travel/return are measured via the troll's distance maps divided
    /// by its movement speed, and chop reflects its chop power. Fruit trees
    /// receive a small bonus so they win ties.
    fn score_tree(troll: &Troll, tree: &Tree, game: &Game) -> Candidate {
        let dist = |p: &Position, map: &HashMap<Position, (i32, Position)>| {
            map.get(p).map_or(i32::MAX, |(d, _)| *d)
        };

        let travel = dist(&tree.position, &troll.dist_map) / troll.movement_speed;
        let chop = tree.health / troll.chop_power;
        let ret = dist(&tree.position, &game.shack_dist_map) / troll.movement_speed;

        #[allow(clippy::cast_precision_loss)]
        let collectible = tree.size.min(troll.free_capacity()) as f32;
        #[allow(clippy::cast_precision_loss)]
        let mut score = collectible / (travel + chop + ret) as f32;

        score += match tree.typ {
            TreeType::Lemon => LEMON_BONUS,
            TreeType::Banana => BANANA_BONUS,
            _ => 0.0,
        };

        Candidate {
            troll_id: troll.id,
            action: Action::Move(troll.id, tree.position),
            #[allow(clippy::cast_possible_truncation)]
            score: (score * SCORE_SCALE) as i32,
            tree: Some(tree.position),
        }
    }

    /// High-priority action to bring a full troll back to the shack and drop.
    fn return_action(troll: &Troll, game: &Game) -> Candidate {
        let (action, score) = if game.is_adjacent_to_shack(troll) {
            (Action::Drop(troll.id), 1000)
        } else {
            (Action::Move(troll.id, game.shack(Side::Me)), 900)
        };

        Candidate {
            troll_id: troll.id,
            action,
            score,
            tree: None,
        }
    }

    /// Greedily assign the highest-scoring actions: each troll acts once,
    /// each tree is claimed by at most one troll. Converts a Move onto a
    /// tree the troll already stands on into a Chop.
    fn assign_actions(&mut self, actions: Vec<Candidate>, trolls: &[&Troll]) {
        let mut busy_trolls: Vec<i32> = Vec::with_capacity(trolls.len());
        let mut claimed_trees: Vec<Position> = Vec::with_capacity(trolls.len());

        for mut act in actions {
            if busy_trolls.contains(&act.troll_id) {
                continue;
            }

            if let Some(pos) = act.tree {
                if claimed_trees.contains(&pos) {
                    continue;
                }

                // If already at the tree, chop instead of moving.
                let troll = trolls.iter().find(|t| t.id == act.troll_id).unwrap();
                if troll.position == pos {
                    act.action = Action::Chop(troll.id);
                }

                claimed_trees.push(pos);
            }

            self.actions.push(act.action);
            busy_trolls.push(act.troll_id);
        }
    }
}
