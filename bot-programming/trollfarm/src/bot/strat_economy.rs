use crate::bot::core::tree_occupied_by_others;
use crate::bot::params;
use crate::bot::{Bot, Candidate};
use crate::game::{Action, Game, Side, Tree, TreeType, Troll};
use crate::utils::Position;

const WOOD_PTS: i32 = 4;
const FRUIT_PTS: i32 = 1;
const MAX_SIZE: i32 = 4;
/// How many of the cells nearest the shack `plant_candidate` weighs up when
/// choosing where to plant (instead of blindly taking the single nearest).
const PLANT_CANDIDATES: usize = 6;

impl Bot {
    pub fn economy_candidates(troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        if troll.has_cargo() {
            out.push(Bot::economy_drop(troll, game));
        }

        // for tree in &game.trees {
        //     if tree_occupied_by_others(tree, troll, game) {
        //         continue;
        //     }
        //     Bot::economy_tree(troll, tree, game, out);
        // }

        // Seed priority: banana first (fast-regrowing), then apple, lemon, plum.
        // Drives both which carried seed to plant and which stocked seed to
        // fetch — the highest-priority type available wins.
        const SEED_PRIORITY: [TreeType; 4] = [
            TreeType::Banana,
            TreeType::Apple,
            TreeType::Lemon,
            TreeType::Plum,
        ];
        let carried = SEED_PRIORITY
            .iter()
            .copied()
            .find(|&t| troll.carries(t) > 0);

        // Fetch a seed from the shack to expand — the highest-priority type
        // stocked there — but only when not already carrying one (a held seed
        // should be planted, not stockpiled).
        if troll.free_capacity() > 0
            && let Some(seed) = SEED_PRIORITY
                .iter()
                .copied()
                .find(|&t| game.inventory(Side::Me).get_by_tree(t) > 0)
            && let Some(c) = Bot::pick_candidate(troll, game, seed)
        {
            out.push(c);
        }

        // Plant the carried seed (highest-priority type held) on the nearest
        // free home cell.
        if let Some(seed) = carried
            && let Some(c) = Bot::plant_candidate(troll, game, seed)
        {
            out.push(c);
        }
    }

    fn economy_drop(troll: &Troll, game: &Game) -> Candidate {
        let p = params::get();

        let speed = troll.movement_speed.max(1);
        let to_shack =
            (Bot::dist(game, &game.shack_dist_map, troll.position) as u32).div_ceil(speed as u32);

        let carried_pts = troll.carry_wood * WOOD_PTS
            + (troll.carry_plum + troll.carry_lemon + troll.carry_apple) * FRUIT_PTS;

        #[allow(clippy::cast_precision_loss)]
        let score = carried_pts as f32 / (to_shack.max(1)) as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Drop(troll.id)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.score_scale) as i32,
            tree: None,
        }
    }

    /// Push chop / co-chop / harvest candidates for one tree (economy scale).
    fn economy_tree(troll: &Troll, tree: &Tree, game: &Game, out: &mut Vec<Candidate>) {
        let speed = troll.movement_speed.max(1);
        let travel_raw = Bot::dist(&game, &troll.dist_map, tree.position);
        if travel_raw == i32::MAX {
            return; // unreachable
        }
        let travel = travel_raw / speed;
        let ret = Bot::dist(&game, &game.shack_dist_map, tree.position) / speed;
        let free = troll.free_capacity();
        if free == 0 {
            return; // can collect nothing; banking (economy_drop) covers this
        }

        // Is an opponent felling this tree right now?
        let opp: Vec<&Troll> = game
            .trolls
            .iter()
            .filter(|t| t.side == Side::Opp && t.position == tree.position && t.chop_power > 0)
            .collect();

        if opp.is_empty() {
            // Chop for wood — but only a tree that has reached max size AND
            // borne fruit. Wood equals size, so a growing tree is worth far more
            // felled later; and requiring a fruit first guarantees the grove a
            // reseed window (harvest competes with chop) before it is liquidated.
            if troll.chop_power > 0 && tree.size >= MAX_SIZE && tree.fruits > 0 {
                let chop = tree.health / troll.chop_power;
                let wood = tree.size.min(free);
                #[allow(clippy::cast_precision_loss)]
                let score = (wood * WOOD_PTS) as f32 / (travel + chop + ret).max(1) as f32;
                out.push(Bot::tree_action(troll, tree.position, score));
            }
            // Harvest fruit. A harvested banana is a *renewable* seed (the tree
            // survives and re-fruits): score the whole reinvest cycle (harvest →
            // walk to a free cell → plant) at `grove_value`, so it keeps seeds
            // flowing yet decays as the grove fills and yields to chopping. A
            // non-banana, or a full grove, is just banked fruit points.
            if tree.fruits > 0 && troll.harvest_power > 0 {
                let gain = tree.fruits.min(free).min(troll.harvest_power);
                let cell = (tree.typ == TreeType::Banana)
                    .then(|| Bot::nearest_free_cell(game, troll))
                    .flatten();
                #[allow(clippy::cast_precision_loss)]
                let (value, cost) = match cell {
                    Some((_, cell_d)) => {
                        (params::get().grove_value, travel + 1 + cell_d / speed + 1)
                    }
                    None => ((gain * FRUIT_PTS) as f32, travel + 1 + ret),
                };
                #[allow(clippy::cast_precision_loss)]
                let score = value / cost.max(1) as f32;
                out.push(Bot::harvest_action(troll, tree.position, score));
            }
        } else if troll.chop_power > 0 {
            // Co-chop: only choppers landing the killing turn share the wood, so
            // it is worth arriving iff we get there by the time the opp fells it.
            let opp_power: i32 = opp.iter().map(|t| t.chop_power).sum();
            let opp_kill = tree.health.div_euclid(opp_power.max(1)).max(0)
                + i32::from(tree.health % opp_power.max(1) != 0);
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let our_arrive = (travel_raw as u32).div_ceil(speed as u32) as i32;
            if our_arrive <= opp_kill {
                #[allow(clippy::cast_precision_loss)]
                let share = (tree.size as f32 / (opp.len() + 1) as f32).min(free as f32);
                let cost = (opp_kill.max(our_arrive) + ret + 1).max(1);
                #[allow(clippy::cast_precision_loss)]
                let score = (share * WOOD_PTS as f32) / cost as f32;
                out.push(Bot::tree_action(troll, tree.position, score));
            }
        }
    }

    /// Fetch `seed` from the shack to plant, scored over the full pick→plant
    /// trip (so it falls off as the nearest free cell gets farther / the grove
    /// fills). Caller picks `seed` by priority; this only builds the candidate.
    fn pick_candidate(troll: &Troll, game: &Game, seed: TreeType) -> Option<Candidate> {
        let p = params::get();
        let speed = troll.movement_speed.max(1) as f32;
        let (_, cell_dist) = Bot::nearest_free_cell(game, troll)?;
        let to_shack = Bot::dist(game, &game.shack_dist_map, troll.position) as f32 / speed;

        const TURNS_PICK: f32 = 1.0;
        const TURNS_PLANT: f32 = 1.0;
        let grove_size = game
            .trees
            .iter()
            .filter(|t| t.position.manhattan(game.shack(Side::Me)) <= 3)
            .collect::<Vec<_>>()
            .len();

        let cost: f32 = TURNS_PICK + TURNS_PLANT + (cell_dist as f32 + to_shack) as f32 / speed;
        #[allow(clippy::cast_precision_loss)]
        #[rustfmt::skip]
        let score = (1.0 / grove_size as f32 + 1.0 / cost.max(1.0) as f32)
            * p.grove_value / game.shack(Side::Me).manhattan(game.shack(Side::Opp)) as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Pick(troll.id, seed)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        assert!(score > 0.001);
        Some(Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.score_scale) as i32,
            tree: None,
        })

        // Some(Bot::candidate(troll.id, action, score, None))
    }

    /// Plant the carried `seed` on the best of the few cells nearest our shack.
    ///
    /// Rather than blindly taking the single closest free cell, this weighs up
    /// the [`PLANT_CANDIDATES`] nearest ones and keeps the highest-scoring
    /// planting spot: `grove_value`, boosted by the seed's water regrowth
    /// speed-up when the cell sits beside water, over the walk to reach it. So a
    /// slightly farther water cell — where the tree re-fruits far faster — can
    /// beat the nearest dry cell. The travel term still caps the grove's natural
    /// size: once near cells fill, the walk grows and chopping overtakes planting.
    fn plant_candidate(troll: &Troll, game: &Game, seed: TreeType) -> Option<Candidate> {
        let p = params::get();
        let speed = troll.movement_speed.max(1) as f32;

        // Re-fruit period for this seed on dry ground vs. beside water; a
        // water-adjacent planting yields `dry / wet` times more often.
        #[allow(clippy::cast_precision_loss)]
        let dry = Tree::initial_cooldown(seed) as f32;
        #[allow(clippy::cast_precision_loss)]
        let wet = Tree::initial_cooldown_water(seed) as f32;

        let (cell, score) = Bot::nearest_free_cells(game, troll, PLANT_CANDIDATES)
            .into_iter()
            .map(|(cell, cell_dist)| {
                let regrowth = if game.is_near_water(cell) {
                    dry / wet.max(1.0)
                } else {
                    1.0
                };
                #[allow(clippy::cast_precision_loss)]
                let cost =
                    (cell_dist as f32 + Bot::dist(game, &troll.dist_map, cell) as f32) / speed;
                let score = p.grove_value * regrowth / cost.max(1.0);
                (cell, score)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))?;

        let action = if troll.position == cell {
            Action::Plant(troll.id, seed)
        } else {
            Action::Move(troll.id, cell)
        };

        assert!(score > 0.001);
        Some(Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.score_scale) as i32,
            tree: None,
        })
    }

    /// A chop-targeted candidate: emit `Chop` when already on the tree, else
    /// `Move` toward it (claimed via `tree` so two trolls don't share it). The
    /// raw per-turn `score` is multiplied by `score_scale` to the integer
    /// ranking key, so sub-1.0 scores don't truncate to 0.
    fn tree_action(troll: &Troll, pos: Position, score: f32) -> Candidate {
        let p = params::get();
        let action = if troll.position == pos {
            Action::Chop(troll.id)
        } else {
            Action::Move(troll.id, pos)
        };

        assert!(score > 0.001);
        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.score_scale) as i32,
            tree: Some(pos),
        }
    }

    /// A harvest-targeted candidate: `Harvest` when on the tree, else `Move`.
    /// As in [`Bot::tree_action`], the raw per-turn `score` is multiplied by
    /// `score_scale` so sub-1.0 scores don't truncate to 0.
    fn harvest_action(troll: &Troll, pos: Position, score: f32) -> Candidate {
        let action = if troll.position == pos {
            Action::Harvest(troll.id)
        } else {
            Action::Move(troll.id, pos)
        };

        assert!(score > 0.001);
        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * params::get().score_scale) as i32,
            tree: Some(pos),
        }
    }
}
