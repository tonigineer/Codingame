use crate::bot::{Bot, Candidate};
use crate::bot::params;
use crate::game::{Troll, Action, Game, ResourceType, Side, TrainCost, Tree, TreeType};
use crate::utils::{CARDINALS, Position};

use std::collections::HashMap;

impl Bot {
    // Only chops trees, at the beginning tries to harrass the opponent
    // near its shack.
    pub fn harasser_candidates(troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        if let Some(candidate) = Bot::last_resort(troll, game) {
            out.push(candidate);
        }

        // Determine score for delivering current cargo
        if troll.has_cargo() {
            out.push(Bot::score_return(troll, game));
        }

        for tree in game.trees.iter().filter(|t| {
            !Bot::tree_occupied_by_others(t, troll, game)
                && !Bot::tree_would_be_gone_on_arrival(t, troll, game)
        }) {
            out.push(Bot::score_chop(troll, tree, game));
        }
    }

    fn last_resort(troll: &Troll, game: &Game) -> Option<Candidate> {
        let here = troll.position;


        // 3. Move onto the nearest opponent's tile (their planting spot); with no
        //    opponent visible, drift to the neighbour closest to their shack.
        let target = game
            .trolls(Side::Opp)
            .iter()
            .min_by_key(|t| Bot::dist(game, &troll.dist_map, t.position))
            .map(|t| t.position);

        match target {
            Some(pos) => {
                if pos != troll.position {
                    Some(Candidate {
                        troll_id: troll.id,
                        action: Action::Move(troll.id, pos),
                        score: 0,
                        tree: None,
                    })
                } else {
                    return None;
                }
            }
            None => None,
        }
    }

    fn score_chop(troll: &Troll, tree: &Tree, game: &Game) -> Candidate {
        // Score for gathering
        let travel = Bot::dist(&game, &troll.dist_map, tree.position) / troll.movement_speed;
        let chop = tree.health / troll.chop_power;
        let ret = Bot::dist(&game, &game.shack_dist_map, troll.position) / troll.movement_speed;

        #[allow(clippy::cast_precision_loss)]
        let collectible = tree.size.min(troll.free_capacity()) as f32;
        #[allow(clippy::cast_precision_loss)]
        let mut score = collectible / (travel + chop + ret).max(1) as f32;

        // Denial: felling a tree near the enemy shack stumps their short-travel
        // economy, independent of our carry capacity, so a full harasser still
        // chops it. Only harassers chase denial; the economy troll weights it 0.
        // let denial_weight = p.denial_bonus;
        // let de
        // if denial_weight > 0.0 {
        let denial_weight = 2.0;
        let opp_dist = game
            .opp_shack_dist_map
            .get(&tree.position)
            .map_or(i32::MAX, |(d, _)| *d);

        if opp_dist <= game.shack(Side::Me).manhattan(game.shack(Side::Opp)) as i32 {
            let proximity = (game.shack(Side::Me).manhattan(game.shack(Side::Opp)) as i32
                - opp_dist
                + 1) as f32
                / (ret + 1) as f32;
            score += denial_weight * proximity / (travel + chop).max(1) as f32;
        }

        let score_scale = match tree.typ {
            TreeType::Lemon => 1.25,
            TreeType::Banana => 1.10,
            TreeType::Plum => 1.05,
            _ => 1.0,
        };

        let scale = 1000.0;

        Candidate {
            troll_id: troll.id,
            action: Action::Move(troll.id, tree.position),
            #[allow(clippy::cast_possible_truncation)]
            score: (score * scale * score_scale) as i32,
            tree: Some(tree.position),
        }
    }

    fn score_return(troll: &Troll, game: &Game) -> Candidate {
        // let p = params::get();
        // let weight = p.return_weight_harasser;
        //
        let ret_turns =
            Bot::dist(&game, &game.shack_dist_map, troll.position) / troll.movement_speed.max(1);

        // let full_boost = if troll.free_capacity() == 0 {
        //     p.return_full_boost
        // } else {
        //     1.0
        // };

        #[allow(clippy::cast_precision_loss)]
        let per_turn = troll.total_carried() as f32 / ret_turns.max(1) as f32;

        let shacks_dist = game.shack(Side::Me).manhattan(game.shack(Side::Opp)) as i32;

        let score = troll.total_carried() as f32 * (shacks_dist as f32 / ret_turns.max(1) as f32);
        let scale = 0.05;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Drop(troll.id)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * scale * 1000.0) as i32,
            tree: None,
        }
    }

    // --- Helper ------------------------------------------------

    /// Whether another of my trolls is already standing on this tree.
    pub fn tree_occupied_by_others(tree: &Tree, troll: &Troll, game: &Game) -> bool {
        game.trolls
            .iter()
            .any(|t| t.side == Side::Me && t.id != troll.id && t.position == tree.position)
    }

    /// Whether an opponent troll would chop this tree before my troll arrives.
    fn tree_would_be_gone_on_arrival(tree: &Tree, troll: &Troll, game: &Game) -> bool {
        if let Some(opp_troll) = game
            .trolls
            .iter()
            .find(|t| t.side == Side::Opp && t.position == tree.position && t.chop_power > 0)
        {
            let dist = troll
                .dist_map
                .get(&tree.position)
                .map_or(i32::MAX, |(d, _)| *d);

            // Stats and distances are non-negative in practice.
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let travel_turns = (dist as u32).div_ceil(troll.movement_speed.max(1) as u32) as i32;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let opp_chop_turns =
                (tree.health as u32).div_ceil(opp_troll.chop_power.max(1) as u32) as i32;

            return opp_chop_turns <= travel_turns;
        }

        false
    }
}
