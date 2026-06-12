use crate::bot::params;
use crate::bot::{Bot, Candidate};
use crate::game::{Action, Game, ResourceType, Side, TrainCost, Tree, TreeType, Troll};
use crate::utils::{CARDINALS, Position};

#[derive(Debug, Clone, Copy)]
enum GatherTier {
    Best,
    Good,
    Least,
}

impl GatherTier {
    const PRIORITY: [GatherTier; 3] = [GatherTier::Best, GatherTier::Good, GatherTier::Least];

    /// Target stock for this tier, from the active [`params`].
    fn threshold(self, p: &params::Params) -> i32 {
        match self {
            GatherTier::Best => p.gather_best,
            GatherTier::Good => p.gather_good,
            GatherTier::Least => p.gather_least,
        }
    }
}

#[derive(Debug)]
struct GatherCandidate {
    resource: ResourceType,
    target: Position,
    /// Distance from the troll's *current* position to the gather spot — the
    /// travel cost to begin this trip. Selecting the nearest target gives
    /// stateless "commitment": the troll keeps heading where it already is,
    /// because its own position is the only memory we need.
    travel: i32,
    // Due to starting on the shack or coming from another drop
    // the current tour could be different than another one.
    cost_now: i32,  // cost for current trip
    cost_next: i32, // cost for another trip
    amount: i32,
}

impl GatherCandidate {
    fn turns_to(&self, goal: i32) -> Option<i32> {
        let need = goal - self.amount;
        (need > 0).then(|| self.cost_now + (need - 1) * self.cost_next)
    }

    fn achievable(&self, goal: i32, remaining: i32) -> bool {
        let Some(turns) = self.turns_to(goal) else {
            return false;
        };

        let possible = turns <= remaining;
        if possible {
            eprintln!(
                "Going for: {goal} {:?} [Remaining turns {turns}]",
                self.resource
            );
        }

        possible
    }
}

impl Bot {
    /// Early-game strategy (turns 1–15): gather resources, then train
    /// the best possible second troll.
    pub(super) fn second_troll(&mut self, game: &Game) {
        let troll = game
            .trolls
            .iter()
            .find(|t| t.side == Side::Me)
            .expect("Where is my first troll?");

        let shack = game.shack(Side::Me);
        let p = params::get();

        if troll.free_capacity() == 0 {
            self.actions.push(Bot::return_to_shack(troll, game, shack));
            return;
        }

        // The minimum-stat troll takes precedence over the fixed early-game window.
        let can_afford_min = game.can_train(
            Side::Me,
            p.min_movement_speed,
            p.min_carry_capacity,
            0,
            p.min_chop_power,
        );

        let candidates = Bot::build_candidates(game);

        // Don't abandon a resource we're already standing next to: if the troll
        // can gather a still-wanted candidate this turn without moving (adjacent
        // to iron / on a fruited tree), bank that zero-travel gain instead of
        // walking off to a "better" but distant target.
        if let Some(action) = Self::gather_in_reach(troll, game, &candidates) {
            self.actions.push(action);
            return;
        }

        // While any floor resource is still short, pursue the deficit
        // unconditionally (the minimum takes precedence over feasibility within
        // the turn window). Once all floors are met, fall back to the normal
        // tier-based gathering bounded by the remaining early-game turns.
        let chosen = if can_afford_min {
            let remaining = p.early_max_turns - i32::from(game.turn);
            Self::pick_best_candidate(&candidates, remaining)
        } else {
            Self::pick_deficit_candidate(&candidates, game)
        };

        if let Some(candidate) = chosen {
            if let Some(action) = Bot::gather_action(troll, game, candidate) {
                self.actions.push(action);
            }
            // A `None` action means the troll is already on the target tree
            // waiting for it to fruit — committed either way.
            return;
        }

        // Train the best troll meeting the full stat floors, if affordable.
        if let Some(action) = Self::best_trainable(game) {
            self.actions.push(action);
            return;
        }

        // Deadlock breaker: the floor is unaffordable and the missing floor
        // resource won't be harvestable within the early-game window (e.g. the
        // only plum trees are size 1, ~30 turns from fruiting). Train the best
        // troll we can afford now — a slightly weaker troll out immediately beats
        // a perfect one dozens of turns late.
        if Self::floor_deficit_stuck(game)
            && let Some(action) = Self::best_affordable(game)
        {
            self.actions.push(action);
        }
    }

    fn return_to_shack(troll: &crate::game::Troll, game: &Game, shack: Position) -> Action {
        if game.is_adjacent_to_shack(troll) {
            Action::Drop(troll.id)
        } else {
            Action::Move(troll.id, shack)
        }
    }

    fn build_candidates(game: &Game) -> Vec<GatherCandidate> {
        let cost_pick_drop = params::get().cost_pick_drop;

        let dist = |p: &Position| game.shack_dist_map.get(p).map_or(i32::MAX, |(d, _)| *d);

        let troll = game.trolls(Side::Me)[0];
        let dist_troll = |p: &Position| troll.dist_map.get(p).map_or(i32::MAX, |(d, _)| *d);

        let inv = game.inventory(Side::Me);

        // A target is either fruited now or a full-size tree about to re-fruit;
        // for the latter the wait is priced into the trip below, so the troll
        // leaves early and arrives as the fruit appears instead of idling at
        // the shack (where the economy fallback used to burn the turns on a
        // pick→drop churn). Ranked by the effective trip start: max(distance,
        // cooldown), so a near-but-slow tree can't shadow a ready one.
        let closest_tree = |rt: ResourceType| {
            game.trees
                .iter()
                .filter(|t| t.get_resource_type() == rt && (t.fruits > 0 || t.size == 4))
                .min_by_key(|t| dist(&t.position).max(if t.fruits > 0 { 0 } else { t.cooldown }))
                .map(|t| t.position)
        };

        let mut candidates = Vec::new();

        let all_adj_mine: Vec<Position> = game
            .mines()
            .flat_map(|m| CARDINALS.iter().map(move |&c| m + c))
            .filter(|pos| game.grid.contains(*pos) && b".ABPL".contains(&game.grid[*pos]))
            .collect();

        if let Some(pos) = closest_tree(ResourceType::Lemon) {
            let cost_travel = dist_troll(&pos);
            if let Some(tree) = game.tree_at(pos) {
                // Travelling and the tree's re-fruit cooldown overlap: the trip
                // effectively starts at whichever ends later.
                let wait = if tree.fruits > 0 { 0 } else { tree.cooldown };
                candidates.push(GatherCandidate {
                    resource: ResourceType::Lemon,
                    target: pos,
                    travel: cost_travel,
                    cost_now: cost_travel.max(wait) + dist(&pos) + cost_pick_drop,
                    cost_next: (dist(&pos) * 2 - 2) + cost_pick_drop,
                    amount: inv.lemon.amount,
                });
            }
        }

        if let Some(pos) = closest_tree(ResourceType::Plum) {
            let cost_travel = dist_troll(&pos);
            if let Some(tree) = game.tree_at(pos) {
                let wait = if tree.fruits > 0 { 0 } else { tree.cooldown };
                candidates.push(GatherCandidate {
                    resource: ResourceType::Plum,
                    target: pos,
                    travel: cost_travel,
                    cost_now: cost_travel.max(wait) + dist(&pos) + cost_pick_drop,
                    cost_next: (dist(&pos) * 2 - 2) + cost_pick_drop,
                    amount: inv.plum.amount,
                });
            }
        }

        if let Some((pos, d)) = all_adj_mine
            .iter()
            .map(|p| (*p, dist(p)))
            .min_by_key(|(_, d)| *d)
        {
            let cost_travel = dist_troll(&pos);
            candidates.push(GatherCandidate {
                resource: ResourceType::Iron,
                target: pos,
                travel: cost_travel,
                cost_now: cost_travel + d + cost_pick_drop,
                cost_next: (d * 2 - 2) + cost_pick_drop,
                amount: inv.iron.amount,
            });
        }

        // Prioritise covering the minimum-stat troll's cost: while any floor
        // resource (plum→speed, lemon→carry, iron→chop) is still short, only chase
        // the ones below their minimum. Once every minimum is met, all candidates
        // are fair game again.
        let cost = Self::floor_cost(game);
        if candidates
            .iter()
            .any(|c| Self::below_floor(&cost, c.resource, c.amount))
        {
            candidates.retain(|c| Self::below_floor(&cost, c.resource, c.amount));
        }

        candidates
    }

    /// Per-resource amounts the minimum-stat second troll costs (its training
    /// floor): `plum` for speed, `lemon` for carry, `iron` for chop.
    fn floor_cost(game: &Game) -> TrainCost {
        let p = params::get();
        game.train_cost(
            Side::Me,
            p.min_movement_speed,
            p.min_carry_capacity,
            0,
            p.min_chop_power,
        )
    }

    /// Whether `amount` of `resource` is still below the floor troll's cost.
    /// Only the floor attributes count; apple/harvest has no floor and is never
    /// gathered in the early game.
    fn below_floor(cost: &TrainCost, resource: ResourceType, amount: i32) -> bool {
        match resource {
            ResourceType::Plum => amount < cost.plum,
            ResourceType::Lemon => amount < cost.lemon,
            ResourceType::Iron => amount < cost.iron,
            _ => false,
        }
    }

    /// Pick the cheapest still-deficit resource to gather next.
    ///
    /// Used while a floor resource is below the minimum-stat troll's cost:
    /// feasibility within the early-game window is deliberately ignored, because
    /// reaching the minimum takes precedence over the turn budget — this is what
    /// the tier-based [`Bot::pick_best_candidate`] could not express (it would
    /// drop a resource already past the smallest tier yet still under the floor).
    /// Returns `None` when no genuinely-deficit resource is gatherable this turn.
    fn pick_deficit_candidate<'a>(
        candidates: &'a [GatherCandidate],
        game: &Game,
    ) -> Option<&'a GatherCandidate> {
        let cost = Self::floor_cost(game);
        candidates
            .iter()
            .filter(|c| Self::below_floor(&cost, c.resource, c.amount))
            .min_by_key(|c| c.cost_now)
    }

    /// MINE/HARVEST for the first still-wanted candidate the troll can act on
    /// *in place* (adjacent to iron, or standing on a fruited tree). Bounded by
    /// the Best goal so it doesn't keep gathering a resource we already have
    /// enough of. `None` when nothing is in reach. Independent of the tier
    /// achievability gate on purpose: grabbing a unit we're already next to is
    /// worth it even when finishing the full goal wouldn't fit the turn window.
    fn gather_in_reach(
        troll: &crate::game::Troll,
        game: &Game,
        candidates: &[GatherCandidate],
    ) -> Option<Action> {
        let p = params::get();
        candidates
            .iter()
            .filter(|c| c.amount < GatherTier::Best.threshold(p))
            .filter_map(|c| Bot::gather_action(troll, game, c))
            .find(|a| !matches!(a, Action::Move(_, _)))
    }

    /// Pick the next gather target: among the candidates achievable at the best
    /// reachable tier, the one nearest the troll. Selecting by distance-to-troll
    /// (rather than insertion order) gives stateless commitment — once the troll
    /// is en route, the target it is closest to stays the pick, so a freshly
    /// fruited far tree can't preempt a trip already underway.
    fn pick_best_candidate(
        candidates: &[GatherCandidate],
        remaining: i32,
    ) -> Option<&GatherCandidate> {
        let p = params::get();
        for tier in GatherTier::PRIORITY {
            if let Some(best) = candidates
                .iter()
                .filter(|c| c.achievable(tier.threshold(p), remaining))
                .min_by_key(|c| c.travel)
            {
                return Some(best);
            }
        }
        None
    }

    /// `None` means "stay put": the troll stands on a still-cooling tree it
    /// came for early — waiting beats a failing `HARVEST` or walking off.
    fn gather_action(
        troll: &crate::game::Troll,
        game: &Game,
        candidate: &GatherCandidate,
    ) -> Option<Action> {
        let at_target = match candidate.resource {
            ResourceType::Iron => game.is_adjacent_to_iron(troll),
            _ => troll.position == candidate.target,
        };

        if !at_target {
            Some(Action::Move(troll.id, candidate.target))
        } else if candidate.resource == ResourceType::Iron {
            Some(Action::Mine(troll.id))
        } else if game.tree_at(candidate.target).is_some_and(|t| t.fruits > 0) {
            Some(Action::Harvest(troll.id))
        } else {
            None
        }
    }

    /// Best troll meeting the full stat floors that we can afford right now.
    /// Floors (`min_*`) live in [`params`]; the shipped defaults require a troll
    /// that can carry a real load, move at a decent pace, and chop wood.
    fn best_trainable(game: &Game) -> Option<Action> {
        let p = params::get();
        Self::best_trainable_with(
            game,
            p.min_movement_speed,
            p.min_carry_capacity,
            p.min_chop_power,
        )
    }

    /// Best troll we can afford right now under the relaxed (deadlock) floors
    /// (`relax_*` in [`params`]): a still-functional troll that can move, carry,
    /// and chop.
    fn best_affordable(game: &Game) -> Option<Action> {
        let p = params::get();
        Self::best_trainable_with(
            game,
            p.relax_movement_speed,
            p.relax_carry_capacity,
            p.relax_chop_power,
        )
    }

    /// Highest-stat-total troll affordable now, with each attribute at or above
    /// the given lower bounds (harvest power is always 0 — we don't gather apple).
    fn best_trainable_with(
        game: &Game,
        min_speed: i32,
        min_carry: i32,
        min_chop: i32,
    ) -> Option<Action> {
        let mut best: Option<(Action, i32)> = None;

        for ms in min_speed..=4 {
            for cc in min_carry..=4 {
                for hp in 0..=0 {
                    for cp in min_chop..=4 {
                        if !game.can_train(Side::Me, ms, cc, hp, cp) {
                            continue;
                        }
                        let score = ms + cc + hp + cp;
                        if best.as_ref().is_none_or(|(_, prev)| score > *prev) {
                            best = Some((Action::Train(ms, cc, hp, cp), score));
                        }
                    }
                }
            }
        }

        best.map(|(action, _)| action)
    }

    /// Whether a stat-floor *fruit* resource (plum→speed, lemon→carry) is short
    /// **and** can't be harvested within `params.stuck_horizon` turns — i.e. no
    /// tree of that type will bear fruit in time. Iron is excluded: it's mined,
    /// so it's never stuck. When true we should stop waiting and train a weaker
    /// troll now rather than strand ourselves at one troll for ~30+ turns.
    fn floor_deficit_stuck(game: &Game) -> bool {
        let cost = Self::floor_cost(game);
        let inv = game.inventory(Side::Me);

        [
            (inv.plum.amount < cost.plum, TreeType::Plum),
            (inv.lemon.amount < cost.lemon, TreeType::Lemon),
        ]
        .iter()
        .any(|&(short, typ)| short && Self::soonest_fruit(game, typ) > params::get().stuck_horizon)
    }

    /// Fewest turns until some tree of `typ` bears a harvestable fruit, or
    /// `i32::MAX` if there is none.
    fn soonest_fruit(game: &Game, typ: TreeType) -> i32 {
        game.trees
            .iter()
            .filter(|t| t.typ == typ)
            .map(|t| Self::time_to_fruit(t, game))
            .min()
            .unwrap_or(i32::MAX)
    }

    /// Candidates for the conditional-3rd-troll mission (latch in `core.rs`):
    /// gather the still-missing training resources for a 2/2/0/2 troll and
    /// bank them — training spends the BANK, and plum/lemon/iron are types the
    /// normal economy never stocks. Scores sit far above the regular harasser
    /// candidates (which the caller pushes as fallback), so mission work wins
    /// whenever any is available and the troll never idles when none is.
    #[allow(clippy::too_many_lines)]
    pub(super) fn train_gather_candidates(troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        let p = params::get();
        let scale = p.score_scale * 30.0;
        let Some((ms, cc, hp, cp)) = Bot::third_troll_plan(game) else {
            return; // mission is being unlatched this turn
        };
        let cost = game.train_cost(Side::Me, ms, cc, hp, cp);
        let inv = game.inventory(Side::Me);

        let short = |c: i32, have: i32| (c - have).max(0);
        let shorts = [
            (ResourceType::Plum, short(cost.plum, inv.plum.amount)),
            (ResourceType::Lemon, short(cost.lemon, inv.lemon.amount)),
            (ResourceType::Apple, short(cost.apple, inv.apple.amount)),
            (ResourceType::Iron, short(cost.iron, inv.iron.amount)),
        ];
        let total_short: i32 = shorts.iter().map(|&(_, s)| s).sum();
        let carried_useful: i32 = shorts
            .iter()
            .map(|&(rt, s)| troll.carries_resource(rt).min(s))
            .sum();

        let bank = |score: f32| {
            let action = if game.is_adjacent_to_shack(troll) {
                Action::Drop(troll.id)
            } else {
                Action::Move(troll.id, game.shack(Side::Me))
            };
            Candidate {
                troll_id: troll.id,
                action,
                #[allow(clippy::cast_possible_truncation)]
                score: score as i32,
                tree: None,
            }
        };
        #[rustfmt::skip]
        let ret = Bot::dist(game, &game.shack_dist_map, troll.position) / troll.movement_speed.max(1);

        // 1. Bank when the hands are full, or when what we carry closes the
        //    remaining bank gap — the Train itself fires in `play()` as soon
        //    as the bank covers the cost.
        #[allow(clippy::cast_precision_loss)]
        if (carried_useful > 0 && carried_useful >= total_short)
            || (troll.free_capacity() == 0 && troll.has_cargo())
        {
            out.push(bank(scale / (1 + ret.max(0)) as f32));
            return;
        }

        // 2. Gather each still-missing resource from its nearest source —
        //    but only what THIS troll can collect: the harasser is trained
        //    with harvest power 0 (it spammed 47 failed harvests before this
        //    gate), so fruits fall to the economy troll, iron to whoever can
        //    mine. The caller runs this for both trolls during the mission.
        for (rt, s) in shorts {
            let to_gather = s - troll.carries_resource(rt);
            if to_gather <= 0 {
                continue;
            }
            if rt != ResourceType::Iron && troll.harvest_power == 0 {
                continue;
            }
            if rt == ResourceType::Iron {
                let target = game
                    .mines()
                    .flat_map(|m| CARDINALS.iter().map(move |&c| m + c))
                    .filter(|pos| game.grid.contains(*pos) && b".ABPL".contains(&game.grid[*pos]))
                    .min_by_key(|pos| Bot::dist(game, &troll.dist_map, *pos));
                if let Some(pos) = target {
                    let travel = Bot::dist(game, &troll.dist_map, pos);
                    if travel == i32::MAX {
                        continue;
                    }
                    let action = if game.is_adjacent_to_iron(troll) {
                        Action::Mine(troll.id)
                    } else {
                        Action::Move(troll.id, pos)
                    };
                    #[allow(clippy::cast_precision_loss)]
                    out.push(Candidate {
                        troll_id: troll.id,
                        action,
                        #[allow(clippy::cast_possible_truncation)]
                        score: (scale / (1 + travel) as f32) as i32,
                        tree: None,
                    });
                }
                continue;
            }
            // Fruit: nearest tree of the type, fruited now or full-size and
            // re-fruiting by arrival (same waiting-aware costing as the
            // opening). A fruitless tree we already stand on is skipped — the
            // fallback harasser candidates fill that turn.
            let target = game
                .trees
                .iter()
                .filter(|t| {
                    t.get_resource_type() == rt
                        && (t.fruits > 0 || t.size == 4)
                        && !(t.position == troll.position && t.fruits == 0)
                })
                .min_by_key(|t| {
                    let d = Bot::dist(game, &troll.dist_map, t.position);
                    d.max(if t.fruits > 0 { 0 } else { t.cooldown })
                });
            if let Some(tree) = target {
                let travel = Bot::dist(game, &troll.dist_map, tree.position);
                if travel == i32::MAX {
                    continue;
                }
                let wait = if tree.fruits > 0 { 0 } else { tree.cooldown };
                // Opportunistic only: the economy troll picks fruit near its
                // working area — abandoning the grove for a cross-map plum
                // costs more than the troll it buys (the deadline handles
                // maps where nothing is ever in reach).
                if travel.max(wait) > p.third_troll_reach {
                    continue;
                }
                let action = if troll.position == tree.position {
                    Action::Harvest(troll.id)
                } else {
                    Action::Move(troll.id, tree.position)
                };
                #[allow(clippy::cast_precision_loss)]
                out.push(Candidate {
                    troll_id: troll.id,
                    action,
                    #[allow(clippy::cast_possible_truncation)]
                    score: (scale / (1 + travel.max(wait)) as f32) as i32,
                    tree: Some(tree.position),
                });
            }
        }
    }

    /// Turns until `tree` next bears fruit. A tree only fruits at size 4; below
    /// that, each cooldown cycle just grows it one size (faster next to water).
    fn time_to_fruit(tree: &Tree, game: &Game) -> i32 {
        if tree.fruits > 0 {
            return 0;
        }
        let period = if game.is_near_water(tree.position) {
            tree.cooldown_time_water()
        } else {
            tree.cooldown_time()
        };
        let grows_needed = (4 - tree.size).max(0);
        tree.cooldown + grows_needed * period
    }
}
