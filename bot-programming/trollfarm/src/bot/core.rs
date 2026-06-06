use crate::game::{Game,Action, Side};
use crate::utils::Position;
use std::collections::HashSet;

pub struct Bot {
    #[allow(dead_code)]
    pub side: Side,
    pub actions: Vec<Action>,
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            side: Side::Me,
            actions: Vec::new(),
        }
    }

    pub fn reset_turn(&mut self, game: &mut Game) {
        self.actions.clear();

        let blocked = HashSet::new();
        game.shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Me), &game.grid, &blocked);
        game.opp_shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Opp), &game.grid, &blocked);

        for troll in &mut game.trolls {
            troll.dist_map = crate::utils::bfs_distance_map(troll.position, &game.grid, &blocked);
        }
    }

    pub fn debug(game: &Game) {
        // DEBUG: dump both shack inventories each turn (+ our per-troll carry).
        // eval.py parses these `[INV]` lines from the replay log for trajectory,
        // composition, wasted-fruit and game-length plots.
        let me = game.inventory(Side::Me);
        let opp = game.inventory(Side::Opp);
        let carried: Vec<String> = game
            .trolls(Side::Me)
            .iter()
            .map(|t| {
                format!(
                    "t{}[P{} L{} A{} B{} I{} W{}]",
                    t.id,
                    t.carry_plum,
                    t.carry_lemon,
                    t.carry_apple,
                    t.carry_banana,
                    t.carry_iron,
                    t.carry_wood
                )
            })
            .collect();
        eprintln!(
            "[INV] turn={} me_shack[P{} L{} A{} B{} I{} W{}] \
                 opp_shack[P{} L{} A{} B{} I{} W{}] {}",
            game.turn,
            me.plum.amount,
            me.lemon.amount,
            me.apple.amount,
            me.banana.amount,
            me.iron.amount,
            me.wood.amount,
            opp.plum.amount,
            opp.lemon.amount,
            opp.apple.amount,
            opp.banana.amount,
            opp.iron.amount,
            opp.wood.amount,
            carried.join(" ")
        );
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}
