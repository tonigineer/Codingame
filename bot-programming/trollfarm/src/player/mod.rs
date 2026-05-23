mod harvesting;
mod planting;
mod priority;
mod simulation;
mod training;

use crate::game::{Action, Game, Side};
use crate::player::priority::Priority;
use crate::position::Position;
use crate::prediction::Snapshot;
use crate::utils::*;

use std::collections::{HashMap, HashSet};

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

pub struct Player {
    pub side: Side,
    pub actions: Vec<Action>,
    pub predicted: Option<Snapshot>,
    pub prev_positions: HashMap<i32, Position>,
    pub plans: Vec<Plan>,
    pub priority: Priority,
    trolls_busy: HashSet<i32>,
    claimed_positions: HashSet<Position>,
    claimed_resources: HashSet<Position>,
}

impl Player {
    #[must_use]
    pub fn new(side: Side) -> Self {
        Self {
            side,
            actions: Vec::new(),
            predicted: None,
            prev_positions: HashMap::new(),
            plans: Vec::new(),
            priority: Priority::new(),
            trolls_busy: HashSet::new(),
            claimed_positions: HashSet::new(),
            claimed_resources: HashSet::new(),
        }
    }

    /// Resets all per-turn working collections so the AI can begin planning
    /// the next turn with a clean slate.
    ///
    /// This must be called at the start of each turn, before any new
    /// decisions are made.
    ///
    /// # Example
    ///
    /// ```
    /// ai.get_ready_for_next_turn();
    /// // ai.actions, ai.trolls_busy, ai.positions_claimed,
    /// // and ai.claimed_entities are now empty.
    /// ```
    fn reset_turn_state(&mut self) {
        self.actions.clear();
        self.trolls_busy.clear();
        self.claimed_positions.clear();
        self.claimed_resources.clear();
    }

    /// Removes plans whose targets are no longer valid on the game board.
    ///
    /// - **Harvest** plans are kept only if the target cell is a tree type
    ///   (`A`, `B`, `P`, or `L`) and the tree still has fruits.
    /// - **Chop** plans are kept only if the target tree is still alive
    ///   (health > 0).
    /// - All other plan types are kept unconditionally.
    ///
    /// After this call, `self.plans` contains only the surviving plans.
    ///
    /// # Example
    ///
    /// ```
    /// // Suppose self.plans contains a Harvest plan targeting a tree
    /// // that has been fully picked (fruits == 0) and a Chop plan
    /// // targeting a tree with health == 3.
    /// ai.validate_existing_plans(&game);
    /// // The stale Harvest plan is removed; the Chop plan remains.
    /// ```
    fn prune_stale_plans(&mut self, game: &Game) {
        eprint!("[PRUNE-STALE-PLANS] Before: #{:?}", self.plans.len());
        self.plans.retain(|p| match &p.action {
            Action::Harvest(_) => {
                b"ABPL".contains(&game.grid[p.to])
                    && game.tree_at(p.to).map_or(false, |t| t.fruits > 0)
            }
            Action::Chop(_) => game.tree_at(p.to).map_or(false, |t| t.health > 0),
            _ => true,
        });
        eprintln!("After: #{:?}", self.plans.len());
    }

    fn act_on_plans(&mut self, game: &Game) {
        for troll in game.trolls_for(self.side).iter() {
            if let Some(plan) = self.plans.iter().find(|p| p.troll_id == troll.id) {
                if plan.to == troll.position {
                    eprintln!("[ACT_ON_PLAN] ID: {} - ACTION: {:?}", troll.id, plan.action);
                    self.actions.push(plan.action);
                    self.trolls_busy.insert(troll.id);
                    self.claimed_positions.insert(troll.position);

                    match plan.action {
                        Action::Harvest(_) => self.claimed_resources.insert(troll.position),
                        _ => false,
                    };
                }
            }
        }
    }

    // ====== THINK — orchestrates all steps ================================
    pub fn think(&mut self, game: &Game) {
        // let trolls = game.trolls_for(self.side);

        // --- 0. Prepare turn
        self.prune_stale_plans(game);
        self.reset_turn_state();
        self.priority.update(game);

        // --- 1. Training new trolls
        if let Some(action) = self.training(game, self.side) {
            self.actions.push(action);
            self.claimed_positions.insert(game.shack(Side::Me));
        }

        // --- 2. Act when destinaion of plan is reached
        self.act_on_plans(game);

        // // --- 2. Planting (first troll only)
        // if let Some(plan) = self.planting(game) {
        //     self.plans.retain(|p| p.troll_id != plan.troll_id);
        //     self.plans.push(plan);
        //     eprintln!("PLANTING: {:?}", plan);
        // }


        // // --- 5. Harvest
        // for troll in trolls.iter().filter(|t| !self.trolls_busy.contains(&t.id)) {
        //     if let Some(plan) = self.harvesting(game, &troll) {
        //         latest_plans.push(plan);
        //     }
        // }

        // --- 6. Move trolls toward their plan targets
        let no_blocked = HashSet::new();
        for troll in trolls.iter() {
            if self.trolls_busy.contains(&troll.id) {
                continue;
            }

            if let Some(plan) = latest_plans.iter().find(|p| p.troll_id == troll.id) {
                // if plan.to == troll.position {
                //     eprintln!("SHOULD NOT HAPPEN");
                //     continue; // already there, action was emitted above
                // }

                let dist_map = bfs_distance_map(troll.position, &game.grid, &no_blocked);
                if let Some(path) = reconstruct_path(troll.position, plan.to, &dist_map) {
                    if !path.is_empty() {
                        let steps = path.len().min(troll.movement_speed as usize);
                        let target = path[steps - 1];
                        self.actions.push(Action::Move(troll.id, target));
                        self.trolls_busy.insert(troll.id);
                    }
                }

                // Keep the plan alive for next turn
                self.plans.push(*plan);
            }
        }

        self.prev_positions = trolls.iter().map(|t| (t.id, t.position)).collect();
    }
}
