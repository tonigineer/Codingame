use crate::game::{Action, Side};
use crate::utils::Position;
use std::collections::HashSet;

pub struct Bot {
    #[allow(dead_code)]
    pub side: Side,
    pub actions: Vec<Action>,
    /// Cells where the economy troll has planted a grove tree. Persisted across
    /// turns (the `Bot` outlives a turn) so the roaming harasser never fells our
    /// home grove — only the economy troll works it. Pruned each turn to cells
    /// that still hold a plant (a felled cell becomes replantable again).
    pub planted_cells: HashSet<Position>,
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            side: Side::Me,
            actions: Vec::new(),
            planted_cells: HashSet::new(),
        }
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}
