use crate::game::Action;
use crate::game::{Game,Side};

/// Turn after which no new trolls will be trained.
const STOP_TRAINING_AFTER_TURN: u16 = 180;

/// Maximum value for any single troll stat (movement speed, carry capacity,
/// harvest power, chop power).
const MAX_STAT: i32 = 5;
/// Minimum value for any single troll stat.
const MIN_STAT: i32 = 1;

pub trait Training {
    /// Provides a default troll-training strategy.
    ///
    /// The strategy iterates over every legal combination of the four troll stats
    /// (`movement_speed`, `carry_capacity`, `harvest_power`, `chop_power`, each in
    /// `1..=5`) and picks the affordable, non-dominated build with the highest
    /// total stat score.
    ///
    /// Training is skipped entirely after turn [`STOP_TRAINING_AFTER_TURN`] (180).
    /// On turn 1, dominated builds are still considered so that the very first
    /// troll can always be recruited.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::game::{Action, Game};
    /// use crate::player::Player;
    ///
    /// let game = make_test_game();   // turn 1, enough resources for a (1,1,1,1) troll
    /// let player = game.current_player();
    ///
    /// // On the first turn a training action is always possible.
    /// let action = Training::training(&game, &player);
    /// assert!(action.is_some());
    ///
    /// if let Some(Action::Train(ms, cc, hp, cp)) = action {
    ///     // The chosen build maximises total stats within the budget.
    ///     assert!(ms >= 1 && cc >= 1 && hp >= 1 && cp >= 1);
    /// }
    ///
    /// // After turn 180 no training is attempted.
    /// let late_game = make_test_game_at_turn(181);
    /// assert_eq!(Training::training(&late_game, &late_game.current_player()), None);
    /// ```
    fn training(game: &Game, side: Side) -> Option<Action> {
        if game.turn > STOP_TRAINING_AFTER_TURN {
            return None;
        }

        let trolls = game.trolls_for(side);

        let best_cc = trolls.iter().map(|t| t.carry_capacity).max().unwrap();
        let best_hp = trolls.iter().map(|t| t.harvest_power).max().unwrap();
        let best_cp = trolls.iter().map(|t| t.chop_power).max().unwrap();
        let best_ms = trolls.iter().map(|t| t.movement_speed).max().unwrap();

        let mut best: Option<(Action, i32)> = None;

        for ms in MIN_STAT..=MAX_STAT {
            for cc in MIN_STAT..=MAX_STAT {
                for hp in MIN_STAT..=MAX_STAT {
                    for cp in MIN_STAT..=MAX_STAT {
                        let dominated =
                            cc <= best_cc && hp <= best_hp && cp <= best_cp && ms <= best_ms;
                        if dominated && game.turn > 1 {
                            continue;
                        }
                        if !game.can_train(side, ms, cc, hp, cp) {
                            continue;
                        }
                        let score = cc + hp + cp + ms;
                        if best.is_none() || score > best.as_ref().unwrap().1 {
                            best = Some((Action::Train(ms, cc, hp, cp), score));
                        }
                    }
                }
            }
        }

        best.map(|(action, _)| action)
    }
}
