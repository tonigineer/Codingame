//! Harasser strategy: a roaming, chop-only troll that pressures the opponent.
//!
//! Every troll trained after the starter takes the harasser role (see
//! [`crate::bot::Candidate`] and the role split in `late_game.rs`). Its job, in
//! priority order, is:
//!
//! 1. **Deny** — while the opponent still has resources, camp the tile their
//!    troll wants to plant on ([`Bot::last_resort`]);
//! 2. **Self-sustain** — once the opponent is tapped out, fall back to a
//!    pick → plant → chop loop at home ([`Bot::seed_workflow`]);
//! 3. **Chop** — otherwise rank every reachable tree by wood-per-turn, with a
//!    denial bonus for trees near the enemy shack ([`Bot::score_chop`]), and
//!    bank cargo when carrying wood ([`Bot::score_return`]).
//!
//! All magic numbers live in [`crate::bot::params`] (`TF_HARASS_*` env vars) so
//! they can be swept by the tuning harness without recompiling.

use crate::bot::core::tree_occupied_by_others;
use crate::bot::params;
use crate::bot::{Bot, Candidate};
use crate::game::{Action, Game, Side, Tree, TreeType, Troll};

/// Whether the opponent has no usable resources left: nothing to train a
/// new troll with and nothing to grow an economy from. Banked wood is
/// excluded — it is finished score, not a means we could deny.
fn opp_resources_empty(game: &Game) -> bool {
    let inv = game.inventory(Side::Opp);
    inv.plum.amount == 0
        && inv.lemon.amount == 0
        && inv.apple.amount == 0
        && inv.banana.amount == 0
}

/// How badly the opponent still needs this tree's fruit for their next troll,
/// in `[0, 1]` — the dynamic side of denial.
///
/// Training a stat costs `n + stat²` of one resource, where `n` is the
/// opponent's current troll count. Assuming they aim for at least
/// [`params::Params::harass_train_min_stat`] (a stat of 1 is wasteful), the
/// target stock per resource is `n + stat²`; the deficit is what they still
/// lack toward it, normalized by that target. Banana is **not** a training
/// resource (score/seed only), so it always returns 0 — felling it cannot delay
/// a troll — and a resource they already have enough of returns 0 too. The
/// result therefore peaks on the fruit that gates the opponent's next troll.
fn opp_train_deficit(game: &Game, typ: TreeType) -> f32 {
    if matches!(typ, TreeType::Banana) {
        return 0.0;
    }
    let p = params::get();
    let n = game.troll_count(Side::Opp);
    let stat = p.harass_train_min_stat.max(1);
    let target = n + stat * stat;
    let deficit = (target - game.inventory(Side::Opp).get(typ.as_resource_type())).max(0);
    #[allow(clippy::cast_precision_loss)]
    {
        deficit as f32 / target.max(1) as f32
    }
}

/// Whether an opponent troll would chop this tree before my troll arrives.
///
/// Compares two whole-turn estimates for the tree's current occupant on the
/// opponent's side: the turns I need to *travel* there
/// (`ceil(distance / movement_speed)`) against the turns they need to *fell* it
/// (`ceil(health / chop_power)`). If they finish no later than I arrive the
/// tree is a wasted target, so [`Bot::harasser_candidates`] filters it out.
/// Returns `false` when no opponent with chop power stands on the tree.
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


/// Harassment intensity in `[0,1]`: how much the harasser should still chase
/// denial rather than farm at home.
///
/// Full early, fading with the turn count and with the opponent's banked score
/// — if they are outscoring us our denial plainly isn't working, so the troll
/// is better off coming home to farm. Reaches `0` at `harass_turn_decay` turns
/// or once the opponent's score hits `harass_opp_cap`, whichever comes first;
/// at `0` the harasser flips to the home economy playbook.
fn harass_factor(game: &Game) -> f32 {
    let p = params::get();
    let turn_w = (1.0 - f32::from(game.turn) / p.harass_turn_decay.max(1.0)).clamp(0.0, 1.0);
    #[allow(clippy::cast_precision_loss)]
    let opp = game.inventory(Side::Opp).score() as f32;
    let opp_w = (1.0 - opp / p.harass_opp_cap.max(1.0)).clamp(0.0, 1.0);
    turn_w * opp_w
}

impl Bot {
    /// Push every scored option for one harasser troll onto `out`.
    ///
    /// In order: the [`Bot::last_resort`] action (deny the opponent, or fall
    /// back to the home seed loop when they are tapped out); a
    /// [`Bot::score_return`] candidate when the troll is carrying wood; and a
    /// [`Bot::score_chop`] candidate for every reachable tree that is neither
    /// already worked by another of my trolls nor about to be felled by the
    /// opponent first. The caller ([`Bot::assign_actions`]) picks the
    /// highest-scoring one.
    ///
    /// # Examples
    ///
    /// ```
    /// use trollfarm::bot::Bot;
    /// use trollfarm::game::{Action, Game, Side};
    ///
    /// // Opponent inventory is all zero and no trees remain on the map. A
    /// // harasser (id 100) carrying a banana seed therefore falls through to
    /// // the self-sufficient seed workflow: move toward a free home cell to
    /// // plant it. With nothing else to do, that is its only candidate.
    /// let input = "\
    /// 3 3
    /// 0..
    /// ...
    /// ..1
    /// 0 0 0 0 0 0
    /// 0 0 0 0 0 0
    /// 0
    /// 1
    /// 100 0 1 1 1 2 1 1 0 0 0 1 0 0";
    /// let game = Game::create_mock(input);
    /// let troll = game.trolls(Side::Me)[0];
    ///
    /// let mut out = Vec::new();
    /// Bot::harasser_candidates(troll, &game, &mut out);
    ///
    /// assert_eq!(out.len(), 1);
    /// assert_eq!(out[0].score, 1000); // params::DEFAULT.harass_seed_plant_score
    /// assert!(matches!(out[0].action, Action::Move(100, _)));
    /// ```
    pub fn harasser_candidates(troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        let hf = harass_factor(game);

        // Harassment exhausted (late game, or the opponent is outscoring us):
        // the troll stops wasting tempo on the far side and farms at home like
        // the economy troll — banking, chopping the home grove, picking/planting.
        if hf <= 0.0 {
            Bot::economy_candidates(troll, game, out);
            return;
        }

        // If there are no trees left, go harass the enemy near its shack; once
        // the enemy is tapped out, switch to planting our own fruit.
        if let Some(candidate) = Bot::last_resort(troll, game, hf) {
            out.push(candidate);
        };

        // Determine score for delivering current cargo.
        // Never drop only fruits, plant them.
        if troll.has_cargo() && troll.carry_wood > 0 {
            out.push(Bot::score_return(troll, game));
        }

        for tree in game.trees.iter().filter(|t| {
            !tree_occupied_by_others(t, troll, game) && !tree_would_be_gone_on_arrival(t, troll, game)
        }) {
            out.push(Bot::score_chop(troll, tree, game, hf));
        }
    }

    /// The fallback action when there is no worthwhile tree to chop.
    ///
    /// While the opponent still holds resources, deny them by moving onto the
    /// nearest opponent troll's tile (its planting spot), scored
    /// [`crate::bot::params::Params::harass_camp_score`]. Once the opponent is
    /// out of resources there is nothing left to deny, so the harasser switches
    /// to the self-sufficient [`Bot::seed_workflow`] instead. Returns `None`
    /// when neither applies (e.g. no opponent visible, or already on target).
    fn last_resort(troll: &Troll, game: &Game, hf: f32) -> Option<Candidate> {
        // Once the opponent has no resources left there is nothing to deny:
        // they cannot train and have nothing to rebuild, so camping their
        // planting spot is wasted tempo. Switch to a pick → plant → chop loop.
        if opp_resources_empty(game) {
            return Bot::seed_workflow(troll, game);
        }

        let p = params::get();

        // Otherwise move onto the nearest opponent's tile (their planting spot);
        // with no opponent visible, there is nothing to deny.
        let target = game
            .trolls(Side::Opp)
            .iter()
            .min_by_key(|t| Bot::dist(game, &troll.dist_map, t.position))
            .map(|t| t.position);

        match target {
            Some(pos) if pos != troll.position => Some(Candidate {
                troll_id: troll.id,
                action: Action::Move(troll.id, pos),
                #[allow(clippy::cast_possible_truncation)]
                score: (p.harass_camp_score * hf) as i32,
                tree: None,
            }),
            _ => None,
        }
    }

    /// A self-sufficient pick → plant → chop step for a harasser with no enemy
    /// economy left to disrupt. Returns the single most valuable next action:
    ///
    /// 1. plant a held seed on the nearest free home cell, scored
    ///    [`crate::bot::params::Params::harass_seed_plant_score`];
    /// 2. otherwise fetch a seed from our shack — the highest-priority fruit
    ///    type stocked (banana → apple → lemon → plum) — scored
    ///    [`crate::bot::params::Params::harass_seed_fetch_score`];
    /// 3. otherwise return `None` and leave it to the chop candidates.
    ///
    /// Steps are scored on the same scale as [`Bot::score_chop`] so the chop
    /// candidates `harasser_candidates` also pushes rank fairly against this
    /// fallback.
    fn seed_workflow(troll: &Troll, game: &Game) -> Option<Candidate> {
        let p = params::get();

        // Seed priority: banana first (renewable, fast-regrowing), then apple,
        // lemon, plum — used both for what to plant and what to fetch.
        const SEED_PRIORITY: [TreeType; 4] = [
            TreeType::Banana,
            TreeType::Apple,
            TreeType::Lemon,
            TreeType::Plum,
        ];
        let carried = SEED_PRIORITY.iter().copied().find(|&t| troll.carries(t) > 0);

        // 1. Plant a carried seed on the nearest free home cell (the
        //    highest-priority type we hold).
        if let Some(seed) = carried
            && let Some((cell, _)) = Bot::nearest_free_cell(game, troll)
        {
            let action = if troll.position == cell {
                Action::Plant(troll.id, seed)
            } else {
                Action::Move(troll.id, cell)
            };

            return Some(Candidate {
                troll_id: troll.id,
                action,
                #[allow(clippy::cast_possible_truncation)]
                score: p.harass_seed_plant_score as i32,
                tree: None,
            });
        }

        // 2. Fetch a seed from the shack to plant — the highest-priority fruit
        //    type currently stocked there.
        let stocked = SEED_PRIORITY
            .iter()
            .copied()
            .find(|&t| game.inventory(Side::Me).get_by_tree(t) > 0);

        if carried.is_none()
            && troll.free_capacity() > 0
            && let Some(seed) = stocked
        {
            let action = if game.is_adjacent_to_shack(troll) {
                Action::Pick(troll.id, seed)
            } else {
                Action::Move(troll.id, game.shack(Side::Me))
            };
            return Some(Candidate {
                troll_id: troll.id,
                action,
                #[allow(clippy::cast_possible_truncation)]
                score: p.harass_seed_fetch_score as i32,
                tree: None,
            });
        }

        // 3. Nothing to plant or fetch — let the regular chop logic take over.
        None
    }

    /// Score moving to (and felling) `tree`, as wood-per-turn plus a denial
    /// bonus for trees near the opponent shack.
    ///
    /// The base score is the collectible wood divided by the round-trip turn
    /// cost (travel + chop + return). When the tree sits within the
    /// shack-to-shack distance of the opponent, a
    /// [`crate::bot::params::Params::harass_denial_weight`] term is added for
    /// stumping their short-travel economy (wood we may waste on purpose). The
    /// result is scaled by [`crate::bot::params::Params::score_scale`] and a
    /// per-fruit `harass_chop_scale_*` multiplier before truncating to the
    /// integer ranking key.
    fn score_chop(troll: &Troll, tree: &Tree, game: &Game, hf: f32) -> Candidate {
        let p = params::get();

        // Score for gathering.
        let travel = Bot::dist(&game, &troll.dist_map, tree.position) / troll.movement_speed;
        let chop = tree.health / troll.chop_power;
        let ret = Bot::dist(&game, &game.shack_dist_map, troll.position) / troll.movement_speed;

        #[allow(clippy::cast_precision_loss)]
        let collectible = tree.size.min(troll.free_capacity()) as f32;
        #[allow(clippy::cast_precision_loss)]
        let mut score = collectible / (travel + chop + ret).max(1) as f32;

        // Denial: felling a tree near the enemy shack stumps their short-travel
        // economy, independent of our carry capacity, so a full harasser still
        // chops it. Faded by `hf` so it tapers off with the turn count / when the
        // opponent is ahead. Only harassers chase denial; the economy troll = 0.
        let denial_weight = p.harass_denial_weight * tree.size as f32 / 4.0 * hf;
        let opp_dist = game
            .opp_shack_dist_map
            .get(&tree.position)
            .map_or(i32::MAX, |(d, _)| *d);

        if opp_dist <= game.shack(Side::Me).manhattan(game.shack(Side::Opp)) as i32 {
            #[rustfmt::skip]
            let proximity = (game.shack(Side::Me).manhattan(game.shack(Side::Opp)) as i32 - opp_dist + 1) as f32 / (ret + 1) as f32;
            // Sharpen denial onto the fruit that actually gates their next troll:
            // banana and resources they already have score no boost.
            let bottleneck = 1.0 + p.harass_bottleneck_weight * opp_train_deficit(game, tree.typ);
            score += denial_weight * proximity / (travel + chop).max(1) as f32 * bottleneck;
        }

        // Static tie-break toward the fruit whose denial hurts training most
        // (lemon > plum > apple > banana, the non-training fruit).
        let type_scale = match tree.typ {
            TreeType::Lemon => p.harass_chop_scale_lemon,
            TreeType::Plum => p.harass_chop_scale_plum,
            TreeType::Apple => p.harass_chop_scale_apple,
            TreeType::Banana => p.harass_chop_scale_banana,
        };

        Candidate {
            troll_id: troll.id,
            action: Action::Move(troll.id, tree.position),
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.score_scale * type_scale) as i32,
            tree: Some(tree.position),
        }
    }

    /// Score banking the troll's current cargo at our shack.
    ///
    /// Values carried units per return-turn, weighted up by the shack-to-shack
    /// distance (cargo gathered far away is worth bringing home), then scaled by
    /// [`crate::bot::params::Params::harass_return_weight`] (small — a
    /// harasser's wood is incidental) and
    /// [`crate::bot::params::Params::score_scale`]. Emits a `Drop` when already
    /// beside the shack, otherwise a `Move` toward it.
    fn score_return(troll: &Troll, game: &Game) -> Candidate {
        let p = params::get();

        let ret_turns =
            Bot::dist(&game, &game.shack_dist_map, troll.position) / troll.movement_speed.max(1);

       let score = troll.carry_wood as f32 / ret_turns.max(1) as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Drop(troll.id)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: (score * p.harass_return_weight * p.score_scale) as i32,
            tree: None,
        }
    }
}
