//! Tunable hyperparameters for the early- and mid-game heuristics.
//!
//! By default these are compile-time constants ([`DEFAULT`]) with zero runtime
//! cost — the exact values that ship to the bot. Building with the `tuning`
//! feature instead makes [`get`] read overrides from environment variables
//! once on first use (falling back to the default for any unset or
//! unparseable var), so an external search script can probe parameter settings
//! without recompiling.
//!
//! Env var names are the field name upper-cased with a `TF_` prefix, e.g.
//! `early_max_turns` → `TF_EARLY_MAX_TURNS`, `lemon_bonus` → `TF_LEMON_BONUS`.

/// All tunable knobs in one place. Read it via [`get`]; never construct ad-hoc.
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct Params {
    // ---- early game (src/bot/early_game.rs) ----
    /// Early-game gathering window (turns) before falling back to feasibility.
    pub early_max_turns: i32,
    /// Gather tier thresholds: target stock per resource, highest tier first.
    pub gather_best: i32,
    pub gather_good: i32,
    pub gather_least: i32,
    /// Fixed pick+drop overhead added to each gather trip's turn cost.
    pub cost_pick_drop: i32,
    /// Minimum stats required of the trained second troll.
    pub min_movement_speed: i32,
    pub min_carry_capacity: i32,
    pub min_chop_power: i32,
    /// Relaxed minimum stats, used only to break a training deadlock.
    pub relax_movement_speed: i32,
    pub relax_carry_capacity: i32,
    pub relax_chop_power: i32,
    /// How long we wait for a deficit fruit before training a weaker troll.
    pub stuck_horizon: i32,

    // ---- mid game (src/bot/mid_game.rs) ----
    /// Tie-break bonuses added to a tree's wood-per-turn score, by fruit type.
    pub lemon_bonus: f32,
    pub banana_bonus: f32,
    /// Harasser's denial weight: per-turn value of felling a tree near the
    /// opponent shack to stump their economy. Independent of carry capacity, so
    /// a full harasser still chops it (wood wasted on purpose).
    pub denial_bonus: f32,
    /// Denial weight for the home economy troll — it should ignore enemy trees,
    /// so this is normally zero.
    pub denial_weight_economy: f32,
    /// BFS radius around the opponent shack counted as "denial range".
    pub opp_denial_radius: i32,
    /// Float-to-int scale for the sortable candidate score.
    pub score_scale: f32,

    // ---- returning home (src/bot/mid_game.rs) ----
    /// Per-role pull toward banking cargo at the shack, on the same
    /// wood-per-turn scale as chopping. The home economy troll values banking;
    /// a roaming harasser's wood is incidental, so it rarely returns.
    pub return_weight_economy: f32,
    pub return_weight_harasser: f32,
    /// Extra multiplier on the return pull when the troll is full — it can
    /// collect nothing more, so loitering is pure waste.
    pub return_full_boost: f32,

    // ---- economy grove (src/bot/strat_economy.rs) ----
    /// Value (points) of adding one more tree to the home banana grove. Scored
    /// per turn over the pick→walk→plant cost, so as near-shack cells fill and
    /// the walk grows, expansion naturally yields to chopping the grown grove.
    pub grove_value: f32,
    /// Hard cap on the home grove: the economy troll stops expanding (no seed
    /// fetch, no new planting) once this many trees stand on our side of the map.
    /// Keeps the grove small enough for one troll to actually service instead of
    /// letting it sprawl to ~20 trees it can never harvest+bank in time.
    pub grove_cap: i32,
    /// Per-turn value coefficients for the home economy troll's three actions,
    /// each its own knob so picking, harvesting and chopping rebalance
    /// independently. The candidate score is `weight * gain / trip_turns`:
    /// (before `score_scale`): `econ_pick_weight` per seed-fetch trip,
    /// `econ_harvest_weight` per fruit harvested, `econ_chop_weight` per wood
    /// unit felled.
    pub econ_pick_weight: f32,
    pub econ_harvest_weight: f32,
    pub econ_chop_weight: f32,
    /// Early-game urgency boost for picking (fetching a seed to expand the
    /// grove): pick weight is multiplied by `1 + econ_pick_early_boost` at
    /// turn 0, fading linearly to `1.0` by `econ_pick_boost_turns`. Builds the
    /// grove early, when a planted tree has time to compound.
    pub econ_pick_early_boost: f32,
    pub econ_pick_boost_turns: f32,
    /// End-game planting margin (turns). `plant_candidate` refuses to plant a
    /// seed unless `turns_remaining >= 4*initial_cooldown(seed) + plant_decay_turns`
    /// — i.e. the tree has time to grow to size 4 and first-fruit (≈4 cooldown
    /// cycles), plus this margin to actually harvest/chop it. Otherwise the
    /// planting scores nothing and the troll banks/chops the cargo instead.
    pub plant_decay_turns: i32,
    /// Grove liquidation horizon (turns before the end). Inside the last this
    /// many turns, `chop_candidate` lifts the below-cap banana protection so
    /// the standing grove gets felled and banked (a size-4 banana = 4 wood =
    /// 16 pts) instead of being harvested for 1-pt fruits until trees are left
    /// standing at turn 300. The same window stops grove expansion, suppresses
    /// our-side harvests (they'd perpetually interrupt a fell on fast-refruit
    /// trees) and applies [`Params::endgame_liquidate_boost`].
    pub endgame_liquidate_turns: i32,
    /// Zero-trees commitment, OFF at the default `1.0`. When set above 1, an
    /// our-side tree whose fell still fits in the remaining turns has its
    /// harvest candidates suppressed and its chop score multiplied by this, so
    /// the home half ends the game fully cleared. Measured -4 to -5 margin
    /// (held-out, all ref opponents) at 2.5-8.0: a standing water apple earns
    /// more as a harvest engine to turn 300 than its wood (20 health = 10
    /// chop turns for 16 pts). Kept as an opt-in knob; bananas and other
    /// cheap trees already get felled by the normal liquidation scoring.
    pub endgame_liquidate_boost: f32,

    // ---- harassment (src/bot/strat_harassment.rs) ----
    /// Flat ranking score for a free harasser planting a held seed on a home
    /// cell. High, so once the opponent has nothing left to deny, growing the
    /// home grove outranks roaming off to chop.
    pub harass_seed_plant_score: f32,
    /// Ranking score for a harasser fetching a seed from the shack to plant.
    /// Low: a setup step, taken only when nothing better is on offer.
    pub harass_seed_fetch_score: f32,
    /// Ranking score for camping the nearest opponent troll's planting tile
    /// while the opponent still has resources worth denying.
    pub harass_camp_score: f32,
    /// Per-turn value of felling a tree near the opponent shack (denial),
    /// folded into a harasser's chop score. Local to the harasser and distinct
    /// from the economy-side [`Params::denial_bonus`] /
    /// [`Params::denial_weight_economy`].
    pub harass_denial_weight: f32,
    /// Per-tree-type multipliers on a harasser's chop score, a mild lean toward
    /// the training fruits (lemon→carry, plum→speed, apple→harvest) over banana.
    /// Keep the spread SMALL. Empirically (vs OldJohn, an economy/wood opponent),
    /// the harasser's real value is denying *wood* — chopping whatever tree sits
    /// nearest the enemy before they fell it — so being picky about fruit type
    /// backfires: lemon=plum=2.0/banana=0.25 made it skip near-enemy bananas and
    /// the opp out-chopped us for +200 (216→196 us, 223→438 them). So leave
    /// banana ≈ 1; the proximity/denial term, not the type, should drive targets.
    /// [`Params::harass_bottleneck_weight`] can sharpen toward training fruit.
    pub harass_chop_scale_lemon: f32,
    pub harass_chop_scale_plum: f32,
    pub harass_chop_scale_apple: f32,
    pub harass_chop_scale_banana: f32,
    /// Multiplier on a harasser's chop score for trees on OUR half (closer to
    /// our shack than the opponent's). `1.0` (default) = treat them like any
    /// other tree — chop them for wood. Lowering it makes the harasser focus on
    /// enemy-side denial, but a broad A/B (5 opponents, seed 2) showed that
    /// **tanks** vs no-economy wood-rushers (gold-X/3/70: ~+55 → ~+7) because
    /// there's nothing to deny and troll 2 is far better off gathering wood;
    /// only vs a strong economy were 0.0 and 1.0 a wash. So denial is not the
    /// harasser's job — wood is. Kept as a knob for the rare all-economy field.
    pub harass_home_scale: f32,
    /// Dynamic denial sharpening. A harasser's denial term for a tree is scaled
    /// by `1 + harass_bottleneck_weight * deficit`, where `deficit` is the
    /// opponent's normalized shortfall of that tree's resource toward their next
    /// troll (cost per resource is `n + stat²`, `n` = opp troll count,
    /// `stat` = [`Params::harass_train_min_stat`]). Banana scores 0 (untrainable),
    /// and a resource they already have enough of scores 0, so denial
    /// concentrates on the tree whose fruit gates the opponent's next troll.
    pub harass_bottleneck_weight: f32,
    /// Assumed minimum attribute level the opponent trains to, used to size the
    /// next-troll resource cost. A stat of 1 is wasteful (`n + 1`), so default 2.
    pub harass_train_min_stat: i32,
    /// Weight on a harasser's return-home score. A harasser's wood is
    /// incidental, so the pull back to bank is deliberately weak.
    pub harass_return_weight: f32,
    /// Harassment fade. The harasser's denial/camp intensity is
    /// `clamp(1 - turn/harass_turn_decay) * clamp(1 - opp_score/harass_opp_cap)`,
    /// in `[0,1]`. Once it reaches 0 — late game, or the opponent is outscoring
    /// us, so denial isn't working — the harasser flips to home economy
    /// instead of wasting tempo on the far side.
    pub harass_turn_decay: f32,
    pub harass_opp_cap: f32,
}

/// The shipped defaults — the values used when the `tuning` feature is off.
pub const DEFAULT: Params = Params {
    early_max_turns: 24,
    gather_best: 10,
    gather_good: 5,
    gather_least: 2,
    cost_pick_drop: 2,
    min_movement_speed: 1,
    min_carry_capacity: 2,
    min_chop_power: 2,
    relax_movement_speed: 1,
    relax_carry_capacity: 1,
    relax_chop_power: 1,
    stuck_horizon: 20,
    lemon_bonus: 0.01,
    banana_bonus: 0.005,
    denial_bonus: 8.0,
    denial_weight_economy: 0.0,
    opp_denial_radius: 6,
    score_scale: 1000.0,
    return_weight_economy: 1.0,
    return_weight_harasser: 0.3,
    return_full_boost: 4.0,
    grove_value: 4.0,
    grove_cap: 6,
    econ_pick_weight: 2.0,
    // Tried 5.0 (2026-06-10) to redirect fruit-turns into wood after losses
    // with 54-69 banked fruit: DISPROVED in the IDE bench — harvests are also
    // the SEED SUPPLY, so nerfing them starves the plant→fell wood engine
    // (one map collapsed 179→129 with wood 41→31). Keep harvest hot; fruit
    // banked is a byproduct of seed gathering, not a detour.
    econ_harvest_weight: 8.0,
    econ_chop_weight: 5.0,
    econ_pick_early_boost: 4.0,
    econ_pick_boost_turns: 120.0,
    plant_decay_turns: 10,
    endgame_liquidate_turns: 40,
    endgame_liquidate_boost: 1.0,
    harass_seed_plant_score: 1000.0,
    harass_seed_fetch_score: 0.0,
    harass_camp_score: 0.0,
    harass_denial_weight: 2.0,
    harass_chop_scale_lemon: 1.75,
    harass_chop_scale_plum: 1.50,
    harass_chop_scale_apple: 1.25,
    harass_chop_scale_banana: 1.00,
    harass_home_scale: 1.0,
    // Net-negative vs every clean (non-tuning) ref opponent in sweeps (gold-X /
    // gold-3 / gold-70: WR falls monotonically as this rises), so off by
    // default. Kept as an opt-in knob — untested against a clean strong-economy
    // bot, the regime where denying a training bottleneck might actually pay.
    // 0.0 → 2.0 (2026-06-10): vs REAL 4-troll opponents the extra trolls are
    // financed by a handful of plum/lemon/apple trees at their shack (one plum
    // tree funded 12 plums by t80 in a Tictac75 loss; 3rd troll t~70, 4th
    // t~120 — our harasser can be there by t~35). The old "net-negative"
    // verdict ([[denial sweeps]]) was measured vs referee bots that never
    // train past 2 — nothing to deny. Re-judged on the real-opponent IDE
    // bench together with the proximity normalization fix in score_chop.
    harass_bottleneck_weight: 2.0,
    harass_train_min_stat: 2,
    harass_return_weight: 1.0,
    harass_turn_decay: 120.0,
    harass_opp_cap: 150.0,
};

/// The active parameters. Without the `tuning` feature this is just a
/// reference to the compile-time [`DEFAULT`] (inlined, free).
#[cfg(not(feature = "tuning"))]
#[inline]
#[must_use]
pub fn get() -> &'static Params {
    &DEFAULT
}

/// The active parameters, with environment-variable overrides applied once on
/// first call (the `tuning` feature).
#[cfg(feature = "tuning")]
#[must_use]
pub fn get() -> &'static Params {
    use std::sync::OnceLock;
    static PARAMS: OnceLock<Params> = OnceLock::new();
    PARAMS.get_or_init(load_from_env)
}

#[cfg(feature = "tuning")]
fn load_from_env() -> Params {
    let p = Params {
        early_max_turns: env_i32("TF_EARLY_MAX_TURNS", DEFAULT.early_max_turns),
        gather_best: env_i32("TF_GATHER_BEST", DEFAULT.gather_best),
        gather_good: env_i32("TF_GATHER_GOOD", DEFAULT.gather_good),
        gather_least: env_i32("TF_GATHER_LEAST", DEFAULT.gather_least),
        cost_pick_drop: env_i32("TF_COST_PICK_DROP", DEFAULT.cost_pick_drop),
        min_movement_speed: env_i32("TF_MIN_MOVEMENT_SPEED", DEFAULT.min_movement_speed),
        min_carry_capacity: env_i32("TF_MIN_CARRY_CAPACITY", DEFAULT.min_carry_capacity),
        min_chop_power: env_i32("TF_MIN_CHOP_POWER", DEFAULT.min_chop_power),
        relax_movement_speed: env_i32("TF_RELAX_MOVEMENT_SPEED", DEFAULT.relax_movement_speed),
        relax_carry_capacity: env_i32("TF_RELAX_CARRY_CAPACITY", DEFAULT.relax_carry_capacity),
        relax_chop_power: env_i32("TF_RELAX_CHOP_POWER", DEFAULT.relax_chop_power),
        stuck_horizon: env_i32("TF_STUCK_HORIZON", DEFAULT.stuck_horizon),
        lemon_bonus: env_f32("TF_LEMON_BONUS", DEFAULT.lemon_bonus),
        banana_bonus: env_f32("TF_BANANA_BONUS", DEFAULT.banana_bonus),
        denial_bonus: env_f32("TF_DENIAL_BONUS", DEFAULT.denial_bonus),
        denial_weight_economy: env_f32("TF_DENIAL_WEIGHT_ECONOMY", DEFAULT.denial_weight_economy),
        opp_denial_radius: env_i32("TF_OPP_DENIAL_RADIUS", DEFAULT.opp_denial_radius),
        score_scale: env_f32("TF_SCORE_SCALE", DEFAULT.score_scale),
        return_weight_economy: env_f32("TF_RETURN_WEIGHT_ECONOMY", DEFAULT.return_weight_economy),
        return_weight_harasser: env_f32(
            "TF_RETURN_WEIGHT_HARASSER",
            DEFAULT.return_weight_harasser,
        ),
        return_full_boost: env_f32("TF_RETURN_FULL_BOOST", DEFAULT.return_full_boost),
        grove_value: env_f32("TF_GROVE_VALUE", DEFAULT.grove_value),
        grove_cap: env_i32("TF_GROVE_CAP", DEFAULT.grove_cap),
        econ_pick_weight: env_f32("TF_ECON_PICK_WEIGHT", DEFAULT.econ_pick_weight),
        econ_harvest_weight: env_f32("TF_ECON_HARVEST_WEIGHT", DEFAULT.econ_harvest_weight),
        econ_chop_weight: env_f32("TF_ECON_CHOP_WEIGHT", DEFAULT.econ_chop_weight),
        econ_pick_early_boost: env_f32("TF_ECON_PICK_EARLY_BOOST", DEFAULT.econ_pick_early_boost),
        econ_pick_boost_turns: env_f32("TF_ECON_PICK_BOOST_TURNS", DEFAULT.econ_pick_boost_turns),
        plant_decay_turns: env_i32("TF_PLANT_DECAY_TURNS", DEFAULT.plant_decay_turns),
        endgame_liquidate_turns: env_i32(
            "TF_ENDGAME_LIQUIDATE_TURNS",
            DEFAULT.endgame_liquidate_turns,
        ),
        endgame_liquidate_boost: env_f32(
            "TF_ENDGAME_LIQUIDATE_BOOST",
            DEFAULT.endgame_liquidate_boost,
        ),
        harass_seed_plant_score: env_f32(
            "TF_HARASS_SEED_PLANT_SCORE",
            DEFAULT.harass_seed_plant_score,
        ),
        harass_seed_fetch_score: env_f32(
            "TF_HARASS_SEED_FETCH_SCORE",
            DEFAULT.harass_seed_fetch_score,
        ),
        harass_camp_score: env_f32("TF_HARASS_CAMP_SCORE", DEFAULT.harass_camp_score),
        harass_denial_weight: env_f32("TF_HARASS_DENIAL_WEIGHT", DEFAULT.harass_denial_weight),
        harass_chop_scale_lemon: env_f32(
            "TF_HARASS_CHOP_SCALE_LEMON",
            DEFAULT.harass_chop_scale_lemon,
        ),
        harass_chop_scale_plum: env_f32(
            "TF_HARASS_CHOP_SCALE_PLUM",
            DEFAULT.harass_chop_scale_plum,
        ),
        harass_chop_scale_apple: env_f32(
            "TF_HARASS_CHOP_SCALE_APPLE",
            DEFAULT.harass_chop_scale_apple,
        ),
        harass_chop_scale_banana: env_f32(
            "TF_HARASS_CHOP_SCALE_BANANA",
            DEFAULT.harass_chop_scale_banana,
        ),
        harass_home_scale: env_f32("TF_HARASS_HOME_SCALE", DEFAULT.harass_home_scale),
        harass_bottleneck_weight: env_f32(
            "TF_HARASS_BOTTLENECK_WEIGHT",
            DEFAULT.harass_bottleneck_weight,
        ),
        harass_train_min_stat: env_i32("TF_HARASS_TRAIN_MIN_STAT", DEFAULT.harass_train_min_stat),
        harass_return_weight: env_f32("TF_HARASS_RETURN_WEIGHT", DEFAULT.harass_return_weight),
        harass_turn_decay: env_f32("TF_HARASS_TURN_DECAY", DEFAULT.harass_turn_decay),
        harass_opp_cap: env_f32("TF_HARASS_OPP_CAP", DEFAULT.harass_opp_cap),
    };
    eprintln!("[PARAMS] {p:?}");
    p
}

#[cfg(feature = "tuning")]
fn env_i32(key: &str, default: i32) -> i32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

#[cfg(feature = "tuning")]
fn env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}
