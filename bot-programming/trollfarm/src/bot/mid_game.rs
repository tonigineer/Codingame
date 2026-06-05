use crate::bot::Bot;
use crate::bot::params;
use crate::game::{Action, Game, Side, Tree, TreeType, Troll};
use crate::utils::Position;
use std::collections::HashMap;

/// Shack points an item is worth when banked (mirrors `Inventory::score`).
const WOOD_PTS: i32 = 4;
const FRUIT_PTS: i32 = 1;
/// A plant grows one size per cooldown up to this, and only fruits at max size
/// (referee `Constants.PLANT_MAX_SIZE`). Wood from felling equals the size, so
/// the economy troll never chops a tree still below this.
const MAX_SIZE: i32 = 4;

/// What a troll is here to do, which decides how it picks actions.
///
/// Our slow/weak starter (lowest id) is the home economy farmer: it grows and
/// then fells a banana grove next to our shack, scoring every option in points
/// per turn. Every trained troll is a roaming harasser using the legacy
/// item-per-turn tree score plus denial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Economy,
    Harasser,
}

/// A candidate move for a single troll, ranked by `score`.
///
/// If `tree` is set, the action targets that tree (claimed so two trolls don't
/// work the same one). A tree-targeted `Move` is turned into a `Chop` once the
/// troll stands on it; terminal actions (`Harvest`/`Chop`/…) are left untouched.
#[derive(Debug)]
struct Candidate {
    troll_id: i32,
    action: Action,
    score: i32,
    tree: Option<Position>,
}

/// Distance from `pos` in a BFS map, or `i32::MAX` if unreachable.
fn dist_in(map: &HashMap<Position, (i32, Position)>, pos: Position) -> i32 {
    map.get(&pos).map_or(i32::MAX, |(d, _)| *d)
}

/// Whether a grid byte is a standing tree (so a planted cell is still occupied).
fn is_tree(b: u8) -> bool {
    matches!(b, b'A' | b'B' | b'L' | b'P')
}

impl Bot {
    /// Mid/late game: rank every troll's options and greedily assign the best.
    ///
    /// 1. Drop grove cells whose tree has been felled (replantable again).
    /// 2. Per troll, generate scored candidates by role.
    /// 3. Sort all candidates by score descending.
    /// 4. Greedily assign: each troll acts once, each tree is claimed once.
    pub fn mid_game(&mut self, game: &Game) {
        let trolls: Vec<&Troll> = game.trolls(Side::Me);

        // Skip if still in early game and moving.
        if trolls.len() <= 1
            && self
                .actions
                .iter()
                .any(|a| !matches!(a, Action::Train(_, _, _, _)))
        {
            return;
        }

        self.planted_cells.retain(|p| is_tree(game.grid[*p]));

        let mut actions = self.collect_actions(&trolls, game);
        actions.sort_by_key(|a| -a.score);

        self.assign_actions(actions, &trolls, game);
    }

    /// Build the scored candidate list, by role, for all trolls.
    fn collect_actions(&self, trolls: &[&Troll], game: &Game) -> Vec<Candidate> {
        let mut actions = Vec::new();
        for troll in trolls {
            match Bot::role(troll, game) {
                Role::Economy => Bot::economy_candidates(troll, game, &mut actions),
                Role::Harasser => self.harasser_candidates(troll, game, &mut actions),
            }
        }
        actions
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

    // ----------------------------------------------------------------------
    // Harasser: legacy item-per-turn tree score + return, skipping our grove.
    // ----------------------------------------------------------------------

    fn harasser_candidates(&self, troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        if troll.has_cargo() {
            out.push(Bot::return_action(troll, game));
        }
        for tree in &game.trees {
            // Never raid our own home grove — that is the economy troll's job.
            if self.planted_cells.contains(&tree.position) {
                continue;
            }
            if Bot::tree_occupied_by_others(tree, troll, game) {
                continue;
            }
            if Bot::tree_would_be_gone_on_arrival(tree, troll, game) {
                continue;
            }
            out.push(Bot::score_tree(troll, tree, game));
        }
    }

    // ----------------------------------------------------------------------
    // Economy: one points-per-turn ranking over the whole PCD loop.
    // ----------------------------------------------------------------------

    /// All candidates for the home economy troll: bank cargo, work any tree
    /// (chop / co-chop a dying enemy tree / harvest), fetch a seed, or plant.
    /// The highest score wins, so the grow→fell loop and its size limit emerge
    /// from geometry instead of explicit phases.
    fn economy_candidates(troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        if troll.has_cargo() {
            out.push(Bot::economy_drop(troll, game));
        }

        for tree in &game.trees {
            if Bot::tree_occupied_by_others(tree, troll, game) {
                continue;
            }
            Bot::economy_tree(troll, tree, game, out);
        }

        // Fetch a banana seed from the shack to expand (only when not already
        // carrying one — a held seed should be planted, not stockpiled).
        if troll.carry_banana == 0
            && troll.free_capacity() > 0
            && game.inventory(Side::Me).banana.amount > 0
            && let Some(c) = Bot::pick_candidate(troll, game)
        {
            out.push(c);
        }

        // Plant a carried seed on the nearest free home cell.
        if troll.carry_banana > 0
            && let Some(c) = Bot::plant_candidate(troll, game)
        {
            out.push(c);
        }
    }

    /// Bank carried cargo: realized points per turn to reach the shack.
    ///
    /// Carried **bananas are excluded** — to the economy troll a banana is seed
    /// currency worth far more planted than the 1 point of banking it, so it is
    /// never dropped while a plant is possible. Only wood and other fruit count.
    fn economy_drop(troll: &Troll, game: &Game) -> Candidate {
        let speed = troll.movement_speed.max(1);
        let to_shack = dist_in(&game.shack_dist_map, troll.position) / speed;

        let carried_pts = troll.carry_wood * WOOD_PTS
            + (troll.carry_plum + troll.carry_lemon + troll.carry_apple) * FRUIT_PTS;

        #[allow(clippy::cast_precision_loss)]
        let score = carried_pts as f32 / (to_shack + 1) as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Drop(troll.id)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };
        Bot::candidate(troll.id, action, score, None)
    }

    /// Push chop / co-chop / harvest candidates for one tree (economy scale).
    fn economy_tree(troll: &Troll, tree: &Tree, game: &Game, out: &mut Vec<Candidate>) {
        let speed = troll.movement_speed.max(1);
        let travel_raw = dist_in(&troll.dist_map, tree.position);
        if travel_raw == i32::MAX {
            return; // unreachable
        }
        let travel = travel_raw / speed;
        let ret = dist_in(&game.shack_dist_map, tree.position) / speed;
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
                    .then(|| Bot::nearest_free_cell(game, &troll.dist_map))
                    .flatten();
                #[allow(clippy::cast_precision_loss)]
                let (value, cost) = match cell {
                    Some((_, cell_d)) => (params::get().grove_value, travel + 1 + cell_d / speed + 1),
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

    /// Fetch a banana seed from the shack, scored over the full pick→plant trip
    /// (so it falls off as the nearest free cell gets farther).
    fn pick_candidate(troll: &Troll, game: &Game) -> Option<Candidate> {
        let p = params::get();
        let speed = troll.movement_speed.max(1);
        let (_, cell_dist) = Bot::nearest_free_cell(game, &game.shack_dist_map)?;
        let to_shack = dist_in(&game.shack_dist_map, troll.position) / speed;

        // pick + walk-to-shack + walk-to-cell + plant
        let cost = to_shack + 1 + cell_dist / speed + 1;
        #[allow(clippy::cast_precision_loss)]
        let score = p.grove_value / cost.max(1) as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Pick(troll.id, TreeType::Banana)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };
        Some(Bot::candidate(troll.id, action, score, None))
    }

    /// Plant the carried seed on the nearest free home cell. The travel term is
    /// where the grove's natural size limit lives: once near cells are full the
    /// walk grows and a grown tree's chop score overtakes planting.
    fn plant_candidate(troll: &Troll, game: &Game) -> Option<Candidate> {
        let p = params::get();
        let speed = troll.movement_speed.max(1);
        let (cell, cell_dist) = Bot::nearest_free_cell(game, &troll.dist_map)?;

        let cost = cell_dist / speed + 1;
        #[allow(clippy::cast_precision_loss)]
        let score = p.grove_value / cost.max(1) as f32;

        let action = if troll.position == cell {
            Action::Plant(troll.id, TreeType::Banana)
        } else {
            Action::Move(troll.id, cell)
        };
        Some(Bot::candidate(troll.id, action, score, None))
    }

    /// Nearest empty grass cell on our side of the map (closer to our shack than
    /// the opponent's), excluding the shack ring, by the given distance map.
    /// Ties prefer a spot next to water (faster regrowth). Returns `(cell, dist)`
    /// where `dist` is read from `map`.
    fn nearest_free_cell(
        game: &Game,
        map: &HashMap<Position, (i32, Position)>,
    ) -> Option<(Position, i32)> {
        let shack = game.shack(Side::Me);
        map.iter()
            .filter_map(|(&pos, &(d, _))| {
                if game.grid[pos] != b'.' || pos.manhattan(shack) <= 1 {
                    return None;
                }
                let opp = dist_in(&game.opp_shack_dist_map, pos);
                let ours = dist_in(&game.shack_dist_map, pos);
                (ours < opp).then_some((pos, d))
            })
            .min_by_key(|&(pos, d)| (d, i32::from(!game.is_near_water(pos))))
    }

    /// A chop-targeted candidate: emit `Chop` when already on the tree, else
    /// `Move` toward it (claimed via `tree` so two trolls don't share it).
    fn tree_action(troll: &Troll, pos: Position, score: f32) -> Candidate {
        let action = if troll.position == pos {
            Action::Chop(troll.id)
        } else {
            Action::Move(troll.id, pos)
        };
        Bot::candidate(troll.id, action, score, Some(pos))
    }

    /// A harvest-targeted candidate: `Harvest` when on the tree, else `Move`.
    fn harvest_action(troll: &Troll, pos: Position, score: f32) -> Candidate {
        let action = if troll.position == pos {
            Action::Harvest(troll.id)
        } else {
            Action::Move(troll.id, pos)
        };
        Bot::candidate(troll.id, action, score, Some(pos))
    }

    /// Wrap a scored action, applying the float→int score scale.
    fn candidate(troll_id: i32, action: Action, score: f32, tree: Option<Position>) -> Candidate {
        let p = params::get();
        Candidate {
            troll_id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.score_scale) as i32,
            tree,
        }
    }

    // ----------------------------------------------------------------------
    // Shared helpers + the legacy harasser scorers (unchanged behaviour).
    // ----------------------------------------------------------------------

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

    /// Score a tree as wood-per-turn for the given (harasser) troll, plus a
    /// capacity-independent denial term near the opponent shack.
    fn score_tree(troll: &Troll, tree: &Tree, game: &Game) -> Candidate {
        let p = params::get();
        let dist = |pos: &Position, map: &HashMap<Position, (i32, Position)>| {
            map.get(pos).map_or(i32::MAX, |(d, _)| *d)
        };

        let travel = dist(&tree.position, &troll.dist_map) / troll.movement_speed;
        let chop = tree.health / troll.chop_power;
        let ret = dist(&tree.position, &game.shack_dist_map) / troll.movement_speed;

        #[allow(clippy::cast_precision_loss)]
        let collectible = tree.size.min(troll.free_capacity()) as f32;
        #[allow(clippy::cast_precision_loss)]
        let mut score = collectible / (travel + chop + ret) as f32;

        score += match tree.typ {
            TreeType::Lemon => p.lemon_bonus,
            TreeType::Banana => p.banana_bonus,
            _ => 0.0,
        };

        // Denial: felling a tree near the enemy shack stumps their short-travel
        // economy, independent of our carry capacity, so a full harasser still
        // chops it. Only harassers chase denial; the economy troll weights it 0.
        let denial_weight = match Bot::role(troll, game) {
            Role::Harasser => p.denial_bonus,
            Role::Economy => p.denial_weight_economy,
        };
        if denial_weight > 0.0 {
            let opp_dist = game
                .opp_shack_dist_map
                .get(&tree.position)
                .map_or(i32::MAX, |(d, _)| *d);
            if opp_dist <= p.opp_denial_radius {
                #[allow(clippy::cast_precision_loss)]
                {
                    let proximity = (p.opp_denial_radius - opp_dist + 1) as f32
                        / (p.opp_denial_radius + 1) as f32;
                    score += denial_weight * proximity / (travel + chop).max(1) as f32;
                }
            }
        }

        Bot::candidate(
            troll.id,
            Action::Move(troll.id, tree.position),
            score,
            Some(tree.position),
        )
    }

    /// Scored action to bank a harasser's cargo (items-per-turn, role weighted).
    fn return_action(troll: &Troll, game: &Game) -> Candidate {
        let p = params::get();

        let weight = match Bot::role(troll, game) {
            Role::Economy => p.return_weight_economy,
            Role::Harasser => p.return_weight_harasser,
        };

        let ret_turns = (dist_in(&game.shack_dist_map, troll.position)
            / troll.movement_speed.max(1))
        .max(1);

        let full_boost = if troll.free_capacity() == 0 {
            p.return_full_boost
        } else {
            1.0
        };

        #[allow(clippy::cast_precision_loss)]
        let per_turn = troll.total_carried() as f32 / ret_turns as f32;
        let score = per_turn * weight * full_boost;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Drop(troll.id)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        Bot::candidate(troll.id, action, score, None)
    }

    /// Greedily assign the highest-scoring actions: each troll acts once, each
    /// tree is claimed by at most one troll. A tree-targeted `Move` becomes a
    /// `Chop` once the troll stands on it; terminal actions are left as-is.
    /// Records freshly planted cells so the harasser leaves the grove alone.
    fn assign_actions(&mut self, actions: Vec<Candidate>, trolls: &[&Troll], game: &Game) {
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
                let troll = trolls.iter().find(|t| t.id == act.troll_id).unwrap();
                if troll.position == pos && matches!(act.action, Action::Move(_, _)) {
                    act.action = Action::Chop(troll.id);
                }
                claimed_trees.push(pos);
            }

            if let Action::Plant(id, _) = act.action
                && let Some(t) = game.trolls.iter().find(|t| t.id == id)
            {
                self.planted_cells.insert(t.position);
            }

            self.actions.push(act.action);
            busy_trolls.push(act.troll_id);
        }
    }
}
