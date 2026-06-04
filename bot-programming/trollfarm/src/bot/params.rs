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
    /// Bonus for chopping lemon/plum trees near the opponent shack when ahead.
    pub denial_bonus: f32,
    /// BFS radius around the opponent shack counted as "denial range".
    pub opp_denial_radius: i32,
    /// Float-to-int scale for the sortable candidate score.
    pub score_scale: f32,
}

/// The shipped defaults — the values used when the `tuning` feature is off.
pub const DEFAULT: Params = Params {
    early_max_turns: 20,
    gather_best: 10,
    gather_good: 5,
    gather_least: 2,
    cost_pick_drop: 2,
    min_movement_speed: 2,
    min_carry_capacity: 2,
    min_chop_power: 1,
    relax_movement_speed: 1,
    relax_carry_capacity: 1,
    relax_chop_power: 1,
    stuck_horizon: 20,
    lemon_bonus: 0.01,
    banana_bonus: 0.005,
    denial_bonus: 2.0,
    opp_denial_radius: 6,
    score_scale: 1000.0,
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
        opp_denial_radius: env_i32("TF_OPP_DENIAL_RADIUS", DEFAULT.opp_denial_radius),
        score_scale: env_f32("TF_SCORE_SCALE", DEFAULT.score_scale),
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
