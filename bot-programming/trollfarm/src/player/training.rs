use crate::game::{Action, Game, Side};

/// Turn after which no new trolls will be trained.
const STOP_TRAINING_AFTER_TURN: u16 = 180;

/// Maximum value for any single troll stat.
const MAX_STAT: i32 = 5;
/// Minimum value for any single troll stat.
const MIN_STAT: i32 = 1;

use super::Player;

impl Player {
    pub fn training(&mut self, game: &Game, side: Side) -> Option<Action> {
        // --- Do not train after a certain number of turns
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
                        if dominated {
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
