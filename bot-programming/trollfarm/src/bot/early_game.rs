use crate::bot::Bot;
use crate::game::{Action, Game, ResourceType, Side};
use crate::utils::{CARDINALS, Position};

const MAX_TURNS: i32 = 15;

#[derive(Debug, Clone, Copy)]
enum GatherTier {
    Best = 10,
    Good = 5,
    Least = 2,
}

impl GatherTier {
    const PRIORITY: [GatherTier; 3] = [GatherTier::Best, GatherTier::Good, GatherTier::Least];
}

#[derive(Debug)]
struct GatherCandidate {
    resource: ResourceType,
    target: Position,
    cost: i32,
    amount: i32,
}

impl GatherCandidate {
    fn turns_to(&self, goal: i32) -> Option<i32> {
        let need = goal - self.amount;
        (need > 0).then(|| need * self.cost + 1)
    }

    fn achievable(&self, goal: i32, remaining: i32) -> bool {
        let Some(turns) = self.turns_to(goal) else {
            return false;
        };
        let possible = turns <= remaining;
        if possible {
            eprintln!("Going for: {goal} {:?} [Remaining {turns}]", self.resource);
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
        let remaining = MAX_TURNS - i32::from(game.turn);

        if troll.free_capacity() == 0 {
            self.actions.push(Bot::return_to_shack(troll, game, shack));
            return;
        }

        let candidates = Bot::build_candidates(game);

        if let Some(candidate) = Self::pick_best_candidate(&candidates, remaining) {
            self.actions
                .push(Bot::gather_action(troll, game, candidate));
            return;
        }

        if let Some(action) = Self::best_trainable(game) {
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
        let dist = |p: &Position| game.shack_dist_map.get(p).map_or(i32::MAX, |(d, _)| *d);

        let inv = game.inventory(Side::Me);

        let closest_tree = |rt: ResourceType| {
            game.trees
                .iter()
                .filter(|t| t.get_resource_type() == rt)
                .min_by_key(|t| dist(&t.position))
                .map(|t| t.position)
        };

        let mut candidates = Vec::new();

        let all_adj_mine: Vec<Position> = game
            .mines()
            .flat_map(|m| CARDINALS.iter().map(move |&c| m + c))
            .filter(|pos| game.grid.contains(*pos) && b".ABPL".contains(&game.grid[*pos]))
            .collect();

        eprintln!("{all_adj_mine:?}");

        if let Some((pos, d)) = all_adj_mine
            .iter()
            .map(|p| (*p, dist(p)))
            .min_by_key(|(_, d)| *d)
        {
            candidates.push(GatherCandidate {
                resource: ResourceType::Iron,
                target: pos,
                cost: (d - 1) * 2 + 2,
                amount: inv.iron.amount,
            });
        }

        if let Some(pos) = closest_tree(ResourceType::Lemon) {
            candidates.push(GatherCandidate {
                resource: ResourceType::Lemon,
                target: pos,
                cost: dist(&pos) * 2 + 2,
                amount: inv.lemon.amount,
            });
        }

        if let Some(pos) = closest_tree(ResourceType::Plum) {
            candidates.push(GatherCandidate {
                resource: ResourceType::Plum,
                target: pos,
                cost: dist(&pos) * 2 + 2,
                amount: inv.plum.amount,
            });
        }

        eprintln!("{candidates:?}");
        candidates
    }

    fn pick_best_candidate(
        candidates: &[GatherCandidate],
        remaining: i32,
    ) -> Option<&GatherCandidate> {
        for tier in GatherTier::PRIORITY {
            for candidate in candidates {
                if candidate.achievable(tier as i32, remaining) {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn gather_action(
        troll: &crate::game::Troll,
        game: &Game,
        candidate: &GatherCandidate,
    ) -> Action {
        let at_target = match candidate.resource {
            ResourceType::Iron => game.is_adjacent_to_iron(troll),
            _ => troll.position == candidate.target,
        };

        if !at_target {
            Action::Move(troll.id, candidate.target)
        } else if candidate.resource == ResourceType::Iron {
            Action::Mine(troll.id)
        } else {
            Action::Harvest(troll.id)
        }
    }

    fn best_trainable(game: &Game) -> Option<Action> {
        let mut best: Option<(Action, i32)> = None;

        for ms in 0..=4 {
            for cc in 0..=4 {
                for hp in 0..=0 {
                    for cp in 0..=4 {
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
}
