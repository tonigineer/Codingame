use crate::bot::core::tree_occupied_by_others;
use crate::bot::params;
use crate::bot::{Bot, Candidate};
use crate::game::{Action, Game, Side, Tree, TreeType, Troll};
use crate::utils::Position;

const WOOD_PTS: i32 = 4;
const FRUIT_PTS: i32 = 1;
/// How many of the cells nearest the shack `plant_candidate` weighs up when
/// choosing where to plant (instead of blindly taking the single nearest).
const PLANT_CANDIDATES: usize = 6;

impl Bot {
    pub fn economy_candidates(troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        // Seed priority: banana first (fast-regrowing), then apple, lemon, plum.
        // Drives both which carried seed to plant and which stocked seed to
        // fetch — the highest-priority type available wins. Replanting carried
        // fruit (apple included — a water apple re-fruits every 2 turns)
        // compounds far better than banking it for 1 pt: a banana-only
        // restriction halved total production in a 5-seed probe. The price is
        // tanky trees at the end (a size-4 apple = 20 health), which the
        // liquidation window pays down instead.
        const SEED_PRIORITY: [TreeType; 4] = [
            TreeType::Banana,
            TreeType::Apple,
            TreeType::Lemon,
            TreeType::Plum,
        ];

        // Grove-size cap: once the grove is full, stop expanding it (no seed
        // fetch, no new planting) so one troll can actually service it. Harvest,
        // chop and banking still run, and any held seed gets dropped (banked).
        // (Expansion is NOT stopped by the liquidation window — the per-seed
        // maturity gates already taper planting off, and blocking it outright
        // measured negative.)
        let p = params::get();
        let grove = Bot::grove_size(game);
        let cap = p.grove_cap;
        let can_expand = grove < cap;
        eprintln!(
            "[ECON] turn={} troll={} grove={}/{} expand={} our_trees={}",
            game.turn,
            troll.id,
            grove,
            cap,
            can_expand,
            game.trees
                .iter()
                .filter(|t| Bot::on_our_side(t, game))
                .count()
        );

        if troll.has_cargo() {
            out.push(Bot::economy_drop(troll, game));
        }

        // Per-tree options: fell a grown, fruited tree for wood and/or harvest
        // its fruit. Both can be offered for the same tree; the assigner commits
        // at most one troll per tree.
        for tree in &game.trees {
            if tree_occupied_by_others(tree, troll, game) {
                continue;
            }
            if let Some(c) = Bot::chop_candidate(troll, tree, game, grove, cap) {
                out.push(c);
            }
            if let Some(c) = Bot::harvest_candidate(troll, tree, game) {
                out.push(c);
            }
        }

        // NB engine selection is deliberately SOFT: `plant_candidate` already
        // prefers water cells via the regrowth factor (apple: 9 dry → 2 wet).
        // A hard "apples only on water" gate measured -9 to -12 margin — even
        // a dry apple re-fruits ~15 times over its life, far above the 1 pt
        // from banking the fruit.
        let carried = SEED_PRIORITY
            .iter()
            .copied()
            .find(|&t| troll.carries(t) > 0);

        // Fetch a seed from the shack to expand — the highest-priority type
        // stocked there — but only when not already carrying one (a held seed
        // should be planted, not stockpiled).
        if can_expand
            && troll.free_capacity() > 0
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
        if can_expand
            && let Some(seed) = carried
            && let Some(c) = Bot::plant_candidate(troll, game, seed)
        {
            out.push(c);
        }
    }

    /// Size of our home **banana garden**: banana trees on our side of the map
    /// (closer to our shack than the opponent's). Only bananas count — the
    /// garden is the renewable banana engine troll 1 cultivates. Natural
    /// apple/lemon/plum trees on our side are still harvested/chopped, but they
    /// are NOT the garden and must not count against the cap; otherwise they
    /// saturate it and block us from ever planting bananas (the "grove never
    /// grows, we just chop apples" bug).
    #[must_use]
    pub fn grove_size(game: &Game) -> i32 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        {
            game.trees
                .iter()
                .filter(|t| Bot::is_grove_tree(t, game))
                .count() as i32
        }
    }

    /// Whether `tree` stands on our half: reachable from our shack and at
    /// least as close to it as to the opponent's.
    fn on_our_side(tree: &Tree, game: &Game) -> bool {
        let me = Bot::dist(game, &game.shack_dist_map, tree.position);
        let opp = Bot::dist(game, &game.opp_shack_dist_map, tree.position);
        me != i32::MAX && me <= opp
    }

    /// Whether `tree` is part of our banana garden: a banana on our side
    /// (reachable from our shack and at least as close to it as the opponent's).
    fn is_grove_tree(tree: &Tree, game: &Game) -> bool {
        matches!(tree.typ, TreeType::Banana) && Bot::on_our_side(tree, game)
    }

    fn economy_drop(troll: &Troll, game: &Game) -> Candidate {
        let p = params::get();

        let speed = troll.movement_speed.max(1);
        #[allow(clippy::cast_sign_loss)]
        let to_shack =
            (Bot::dist(game, &game.shack_dist_map, troll.position) as u32).div_ceil(speed as u32);

        // Carried bananas are seeds, not score, while planting is still
        // viable — but once it's too late for a new tree to pay off (the same
        // maturity horizon that gates `plant_candidate`), a held banana is
        // worth its fruit point only if banked, so count it toward dropping.
        let banana_pts = if game.turns_remaining()
            < Tree::initial_cooldown(TreeType::Banana) * 4 + p.plant_decay_turns
        {
            troll.carry_banana
        } else {
            0
        };
        let carried_pts = troll.carry_wood * WOOD_PTS
            + (troll.carry_plum + troll.carry_lemon + troll.carry_apple + banana_pts) * FRUIT_PTS;

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

    /// Fell `tree` for wood, scored `econ_chop_weight` per wood unit over the
    /// round-trip turns (travel + chop + return). `None` when the troll can't
    /// chop, it's full, or the tree is unreachable. Deliberately NOT gated on
    /// ripeness or end-game bankability: a fell the opponent would otherwise
    /// get is denial, and gating it measured -2.4 to -5.7 margin (held-out,
    /// paired) against every reference opponent.
    fn chop_candidate(
        troll: &Troll,
        tree: &Tree,
        game: &Game,
        grove: i32,
        cap: i32,
    ) -> Option<Candidate> {
        let p = params::get();
        let free = troll.free_capacity();
        if troll.chop_power == 0 || free == 0 {
            return None;
        }

        // Protect the banana engine: only fell a banana once the garden is at
        // (or above) its cap — i.e. there's a spare one to turn into wood, which
        // troll 1 then replants. Below the cap, leave bananas to be harvested
        // (renewable fruit + seeds) so the engine keeps running. Natural
        // (non-banana) trees can always be felled for their one-time wood.
        // EXCEPT in the liquidation window at the end of the game: a standing
        // tree at turn 300 is ~16 unrealized points, so the protection lifts
        // and the grove gets felled and banked.
        if matches!(tree.typ, TreeType::Banana)
            && grove < cap
            && game.turns_remaining() > p.endgame_liquidate_turns
        {
            return None;
        }

        let speed = troll.movement_speed.max(1);
        let travel_raw = Bot::dist(game, &troll.dist_map, tree.position);
        if travel_raw == i32::MAX {
            return None; // unreachable
        }
        let travel = travel_raw / speed;
        let ret = Bot::dist(game, &game.shack_dist_map, tree.position) / speed;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        let chop = (tree.health as u32).div_ceil(troll.chop_power.max(1) as u32) as i32;

        let wood = tree.size.min(free);
        let cost = (travel + chop + ret).max(1);
        #[allow(clippy::cast_precision_loss)]
        let mut score = p.econ_chop_weight * wood as f32 / cost as f32;

        // Optional zero-trees commitment (`endgame_liquidate_boost` > 1, OFF
        // by default — see params): clearing our half outranks roaming. Per-
        // tree gate: only a fell that still fits (chop + walk home + drop)
        // gets the boost — half-chopping a tree we can't finish forfeits both
        // fruit and wood.
        if p.endgame_liquidate_boost > 1.0
            && Bot::on_our_side(tree, game)
            && game.turns_remaining() <= p.endgame_liquidate_turns
            && travel + chop + ret < game.turns_remaining()
        {
            score *= p.endgame_liquidate_boost;
        }

        Some(Bot::tree_action(troll, tree.position, score))
    }

    /// Harvest `tree`'s fruit, scored `econ_harvest_weight` per fruit gained
    /// over the round-trip turns (travel + harvest + return). A harvested fruit
    /// banks a point now and — for the seed types — becomes a carried seed that
    /// `plant_candidate` can then reinvest into the grove. `None` when the tree
    /// has no fruit, the troll can't harvest, it's full, or it's unreachable.
    fn harvest_candidate(troll: &Troll, tree: &Tree, game: &Game) -> Option<Candidate> {
        let p = params::get();
        let free = troll.free_capacity();
        if tree.fruits == 0 || troll.harvest_power == 0 || free == 0 {
            return None;
        }

        // Optional zero-trees commitment (`endgame_liquidate_boost` > 1, OFF
        // by default — see params): an our-side tree whose fell still fits in
        // the game is felled, not harvested — a water apple re-fruits every 2
        // turns, so its harvest candidate would perpetually interrupt
        // (outscore) its own 10-turn fell and the tree stands at turn 300. A
        // tree too tanky to clear in the remaining turns keeps being
        // harvested as usual.
        if p.endgame_liquidate_boost > 1.0
            && Bot::on_our_side(tree, game)
            && game.turns_remaining() <= p.endgame_liquidate_turns
            && troll.chop_power > 0
        {
            let speed = troll.movement_speed.max(1);
            let travel = Bot::dist(game, &troll.dist_map, tree.position) / speed;
            let ret = Bot::dist(game, &game.shack_dist_map, tree.position) / speed;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let chop = (tree.health as u32).div_ceil(troll.chop_power as u32) as i32;
            // saturating: an unreachable tree's distance is i32::MAX
            if travel
                .saturating_add(chop)
                .saturating_add(ret)
                .saturating_add(1)
                <= game.turns_remaining()
            {
                return None;
            }
        }

        let speed = troll.movement_speed.max(1);
        let travel_raw = Bot::dist(game, &troll.dist_map, tree.position);
        if travel_raw == i32::MAX {
            return None; // unreachable
        }
        let travel = travel_raw / speed;
        let ret = Bot::dist(game, &game.shack_dist_map, tree.position) / speed;

        let gain = tree.fruits.min(free).min(troll.harvest_power);
        let cost = (travel + 1 + ret).max(1);
        #[allow(clippy::cast_precision_loss)]
        let score = p.econ_harvest_weight * gain as f32 / cost as f32;
        Some(Bot::harvest_action(troll, tree.position, score))
    }

    /// Fetch `seed` from the shack to expand the grove, scored `econ_pick_weight`
    /// over the full trip turns (walk to shack, pick, walk to the nearest free
    /// cell, plant). The cost rises as the grove fills and the nearest free cell
    /// recedes, so picking naturally tapers off. Caller picks `seed` by priority.
    fn pick_candidate(troll: &Troll, game: &Game, seed: TreeType) -> Option<Candidate> {
        let p = params::get();

        // Only fetch a seed we could still plant in time. Without this, once it's
        // too late to plant (end-game), the troll fetches a seed it can never use
        // and pick→drops it every turn forever (the apple-churn). Mirrors the same
        // maturity gate in `plant_candidate`.
        let maturity = Tree::initial_cooldown(seed) * 4 + p.plant_decay_turns;
        if game.turns_remaining() < maturity {
            return None;
        }

        let move_speed = troll.movement_speed.max(1);
        let (_, cell_dist) = Bot::nearest_free_cell(game, troll)?;
        let to_shack = Bot::dist(game, &game.shack_dist_map, troll.position) / move_speed;

        // walk-to-shack + pick + walk-to-cell + plant
        let cost = (to_shack + cell_dist / move_speed + 2).max(1);
        // Expanding the grove pays off most early (a planted tree has time to
        // compound), so boost picking up front and let it fade with the turn.
        let early = 1.0
            + p.econ_pick_early_boost
                * (1.0 - f32::from(game.turn) / p.econ_pick_boost_turns.max(1.0)).clamp(0.0, 1.0);
        #[allow(clippy::cast_precision_loss)]
        let score = p.econ_pick_weight * early / cost as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Pick(troll.id, seed)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        Some(Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.score_scale) as i32,
            tree: None,
        })
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

        // Don't plant a seed too late to pay off. A fresh size-1 tree needs ~4
        // cooldown cycles to grow to size 4 and bear its first fruit (dry-ground
        // worst case, matching early_game's `time_to_fruit`), plus a
        // `plant_decay_turns` margin to then harvest/chop it. With less game
        // left the tree yields nothing — bank or chop the cargo instead. (This
        // is why late apple plants — apple's 9-turn cooldown → 36+ to mature —
        // were pure waste.)
        let maturity = Tree::initial_cooldown(seed) * 4 + p.plant_decay_turns;
        if game.turns_remaining() < maturity {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        let move_speed = troll.movement_speed.max(1) as f32;

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
                    (cell_dist as f32 + Bot::dist(game, &troll.dist_map, cell) as f32) / move_speed;
                let score = p.grove_value * regrowth / cost.max(1.0);
                (cell, score)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))?;

        let action = if troll.position == cell {
            Action::Plant(troll.id, seed)
        } else {
            Action::Move(troll.id, cell)
        };

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

        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * params::get().score_scale) as i32,
            tree: Some(pos),
        }
    }
}
