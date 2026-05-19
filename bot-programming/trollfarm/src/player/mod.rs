mod movement;
mod planting;
mod priority;
mod simulation;
mod targeting;
mod training;

use crate::game::{Action, Game, Side};
use crate::position::Position;
use crate::prediction::Snapshot;

use std::collections::{HashMap, HashSet};

use priority::Priority;

// ========================================================================
// Plan
// ========================================================================

#[derive(Debug, Copy, Clone)]
pub struct Plan {
    pub troll_id: i32,
    pub to: Position,
    pub action: Action,
}

// ========================================================================
// Player
// ========================================================================

/// Player entity to play the game.
///
/// Example:
///
///
pub struct Player {
    pub side: Side,
    pub actions: Vec<Action>,
    pub predicted: Option<Snapshot>,
    pub prev_positions: HashMap<i32, Position>,
    pub priority: Priority,
    pub plans: Vec<Plan>,
    trolls_busy: HashSet<i32>,
    positions_claimed: HashSet<Position>,
    claimed_entities: HashSet<Position>,
}

impl Player {
    #[must_use]
    pub fn new(side: Side) -> Self {
        Self {
            side,
            actions: Vec::new(),
            predicted: None,
            prev_positions: HashMap::new(),
            priority: Priority::new(),
            plans: Vec::new(),
            trolls_busy: HashSet::new(),
            positions_claimed: HashSet::new(),
            claimed_entities: HashSet::new(),
        }
    }

    fn get_ready_for_next_turn(&mut self) {
        self.actions.clear();
        self.trolls_busy.clear();
        self.positions_claimed.clear();
        self.claimed_entities.clear();
    }

    // ====== THINK — orchestrates all steps ================================
    pub fn think(&mut self, game: &Game) {
        // --- 0. Getting ready
        self.priority.update(game);
        self.get_ready_for_next_turn();

        let shack = game.shack(self.side);
        let trolls = game.trolls_for(self.side);

        // --- 1. Training new trolls
        if let Some(action) = self.training(game, self.side) {
            self.actions.push(action);
        }

        // --- 2. Planting new trees (placeholder)
        if let Some(plan) = self.planting(game) {
            // Remve plan if troll already has a plan (helps not dropping
            // fruit which was planned for planting.
            self.plans.pop_if(|p| p.troll_id == plan.troll_id);
            self.plans.push(plan);
        }

        // --- 3. Validate existing plan
        let latest_plans: Vec<Plan> = self
            .plans
            .drain(..)
            .filter(|p| match p.action {
                // Tree for harvest does not exist anymore
                Action::Harvest(_) => {
                    b"ABPL".contains(&game.grid[p.to])
                        && game.tree_at(p.to).map(|t| t.fruits > 0).unwrap_or(false)
                }
                // Tree for chopping does not exist anymore
                Action::Chop(_) => game.tree_at(p.to).map(|t| t.health > 0).unwrap_or(false),
                _ => true,
            })
            .collect();

        // --- 4. Execute arrived plans
        for troll in trolls.iter() {
            if let Some(plan) = latest_plans.iter().find(|p| p.troll_id == troll.id) {
                if plan.to == troll.position {
                    self.actions.push(plan.action);
                    self.trolls_busy.insert(troll.id);
                    self.positions_claimed.insert(troll.position);
                    self.claimed_entities.insert(troll.position);
                }
            }
        }

        // --- 5. Opportunistic harvesting/mining for idle trolls
        self.opportunistic_actions(&game, &trolls);

        // Mark in-progress plan destinations
        for plan in &latest_plans {
            if !self.trolls_busy.contains(&plan.troll_id) {
                self.claimed_entities.insert(plan.to);
            }
        }

        // --- 6. Movement and new plan assignment
        let move_intents = self.assign_moves(game, &trolls, &shack, &latest_plans);

        // --- 7. Resolve collisions and emit MOVE actions
        self.resolve_collisions(&game, &move_intents);
        self.prev_positions = trolls.iter().map(|t| (t.id, t.position)).collect();
    }
}
