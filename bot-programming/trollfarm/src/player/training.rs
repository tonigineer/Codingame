use crate::game::{Action, Game, Side};

use super::Player;

/// Minimum stats for trolls 2–4: [movement_speed, carry_capacity, harvest_power, chop_power].
///
/// The starter troll (index 0, carry_capacity = 1) is not listed here;
/// it is created automatically at the start of the game.
#[rustfmt::skip]
const TROLL_BUILDS: &[[i32; 4]] = &[
    [1, 1, 1, 1], // 2nd troll: balanced harvester
    [4, 5, 4, 5], // 3rd troll: stronger all-rounder
    [4, 5, 1, 5], // 4th troll: dedicated chopper
];

/// Upper bound (inclusive) for each individual stat.
const MAX_STAT: i32 = 5;

impl Player {
    /// Attempts to train the next troll, if affordable and the shack is free.
    ///
    /// The build order is defined by [`TROLL_BUILDS`]: each entry sets the
    /// minimum stats for the next troll to recruit. The method searches all
    /// stat combinations at or above those minimums and picks the one with
    /// the **highest total stat sum** that `game.can_train` allows, spending
    /// as much of the available budget as possible.
    ///
    /// Returns `None` when:
    /// - All planned trolls have already been trained.
    /// - A troll is standing on the shack (blocking spawns).
    /// - No affordable build meets the minimum requirements.
    ///
    /// # Panics
    ///
    /// Panics if `game.trolls_for(side)` returns an empty list (there must
    /// always be at least the starter troll).
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(action) = ai.training(&game, side) {
    ///     // action is e.g. Action::Train(4, 5, 4, 5)
    ///     submit(action);
    /// }
    /// ```
    pub fn training(&mut self, game: &Game, side: Side) -> Option<Action> {
        let trolls = game.trolls_for(side);
        let num_trolls = trolls.len();

        // All planned trolls already trained?
        if num_trolls > TROLL_BUILDS.len() {
            return None;
        }

        // Shack occupied — can't spawn a new troll.
        if trolls.iter().any(|t| t.position == game.shack(side)) {
            return None;
        }

        let mins = TROLL_BUILDS[num_trolls - 1];

        // Search all stat combos at or above the minimums; keep the one
        // with the highest total, i.e. the most expensive affordable build.
        let mut best: Option<(Action, i32)> = None;

        for ms in mins[0]..=MAX_STAT {
            for cc in mins[1]..=MAX_STAT {
                for hp in mins[2]..=MAX_STAT {
                    for cp in mins[3]..=MAX_STAT {
                        if !game.can_train(side, ms, cc, hp, cp) {
                            continue;
                        }
                        let score = ms + cc + hp + cp;
                        match best {
                            Some((_, prev)) if score <= prev => {}
                            _ => best = Some((Action::Train(ms, cc, hp, cp), score)),
                        }
                    }
                }
            }
        }

        if let Some((action, _)) = &best {
            eprintln!(
                "[TRAINING] Troll #{} with {:?} (mins {:?})",
                num_trolls + 1,
                action,
                mins
            );
        }

        best.map(|(action, _)| action)
    }
}
