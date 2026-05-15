use crate::game::Game;
use crate::position::Position;
use crate::entities::{Inventory, Tree, Troll};
use crate::game::{Action, Side};

// ------------------------------------------------------------------------
// Snapshot types
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SnapshotTroll {
    id: i32,
    position: Position,
    carry_plum: i32,
    carry_lemon: i32,
    carry_apple: i32,
    carry_banana: i32,
}

impl SnapshotTroll {
    fn from(t: &Troll) -> Self {
        Self {
            id: t.id,
            position: t.position,
            carry_plum: t.carry_plum,
            carry_lemon: t.carry_lemon,
            carry_apple: t.carry_apple,
            carry_banana: t.carry_banana,
        }
    }

    fn diff(&self, actual: &Troll) -> Vec<String> {
        let mut diffs = Vec::new();
        if self.position != actual.position {
            diffs.push(format!(
                "position: predicted {:?} got {:?}",
                self.position, actual.position
            ));
        }
        if self.carry_plum != actual.carry_plum {
            diffs.push(format!(
                "carry_plum: predicted {} got {}",
                self.carry_plum, actual.carry_plum
            ));
        }
        if self.carry_lemon != actual.carry_lemon {
            diffs.push(format!(
                "carry_lemon: predicted {} got {}",
                self.carry_lemon, actual.carry_lemon
            ));
        }
        if self.carry_apple != actual.carry_apple {
            diffs.push(format!(
                "carry_apple: predicted {} got {}",
                self.carry_apple, actual.carry_apple
            ));
        }
        if self.carry_banana != actual.carry_banana {
            diffs.push(format!(
                "carry_banana: predicted {} got {}",
                self.carry_banana, actual.carry_banana
            ));
        }
        diffs
    }
}

#[derive(Debug, Clone)]
struct SnapshotInventory {
    plum: i32,
    lemon: i32,
    apple: i32,
    banana: i32,
}

impl SnapshotInventory {
    fn from(r: &Inventory) -> Self {
        Self {
            plum: r.plum.amount(),
            lemon: r.lemon.amount(),
            apple: r.apple.amount(),
            banana: r.banana.amount(),
        }
    }

    fn diff(&self, r: &Inventory) -> Vec<String> {
        let mut diffs = Vec::new();
        if self.plum != r.plum.amount() {
            diffs.push(format!(
                "plum: predicted {} got {}",
                self.plum,
                r.plum.amount()
            ));
        }
        if self.lemon != r.lemon.amount() {
            diffs.push(format!(
                "lemon: predicted {} got {}",
                self.lemon,
                r.lemon.amount()
            ));
        }
        if self.apple != r.apple.amount() {
            diffs.push(format!(
                "apple: predicted {} got {}",
                self.apple,
                r.apple.amount()
            ));
        }
        if self.banana != r.banana.amount() {
            diffs.push(format!(
                "banana: predicted {} got {}",
                self.banana,
                r.banana.amount()
            ));
        }
        diffs
    }
}

#[derive(Debug, Clone)]
struct SnapshotTree {
    position: Position,
    fruits: i32,
    size: i32,
    cooldown: i32,
}

impl SnapshotTree {
    fn from(t: &Tree) -> Self {
        Self {
            position: t.position,
            fruits: t.fruits,
            size: t.size,
            cooldown: t.cooldown,
        }
    }

    fn diff(&self, actual: &Tree) -> Vec<String> {
        let mut diffs = Vec::new();
        if self.fruits != actual.fruits {
            diffs.push(format!(
                "fruits: predicted {} got {}",
                self.fruits, actual.fruits
            ));
        }
        if self.size != actual.size {
            diffs.push(format!("size: predicted {} got {}", self.size, actual.size));
        }
        if self.cooldown != actual.cooldown {
            diffs.push(format!(
                "cooldown: predicted {} got {}",
                self.cooldown, actual.cooldown
            ));
        }
        diffs
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    inventory: SnapshotInventory,
    trolls: Vec<SnapshotTroll>,
    trees: Vec<SnapshotTree>,
}

// ------------------------------------------------------------------------
// Trait
// ------------------------------------------------------------------------

pub trait Predictable {
    fn snapshot(&self, my_actions: &[Action], opp_actions: &[Action]) -> Snapshot;
    fn compare(&self, snapshot: &Snapshot);
}

impl Predictable for Game {
    fn snapshot(&self, my_actions: &[Action], opp_actions: &[Action]) -> Snapshot {
        let mut sim = self.clone();
        sim.play(my_actions, opp_actions);

        Snapshot {
            inventory: SnapshotInventory::from(sim.inventory(Side::Me)),
            trolls: sim
                .trolls
                .iter()
                .filter(|t| t.side == Side::Me)
                .map(SnapshotTroll::from)
                .collect(),
            trees: sim.trees.iter().map(SnapshotTree::from).collect(),
        }
    }

    fn compare(&self, snapshot: &Snapshot) {
        let mut ok = true;

        // Compare inventory
        for diff in snapshot.inventory.diff(self.inventory(Side::Me)) {
            ok = false;
            eprintln!("[SIM MISMATCH] inventory: {diff}");
        }

        // Compare my trolls
        for pred_troll in &snapshot.trolls {
            match self.trolls.iter().find(|t| t.id == pred_troll.id) {
                Some(actual) => {
                    for diff in pred_troll.diff(actual) {
                        ok = false;
                        eprintln!("[SIM MISMATCH] troll {}: {diff}", pred_troll.id);
                    }
                }
                None => {
                    ok = false;
                    eprintln!(
                        "[SIM MISMATCH] troll {} missing in actual state",
                        pred_troll.id
                    );
                }
            }
        }

        // Compare trees (skip those with opponent trolls — can't predict their harvest)
        for pred_tree in &snapshot.trees {
            let opp_harvesting = self
                .trolls
                .iter()
                .any(|t| t.side == Side::Opp && t.position == pred_tree.position);

            if opp_harvesting {
                continue;
            }

            match self.trees.iter().find(|t| t.position == pred_tree.position) {
                Some(actual) => {
                    for diff in pred_tree.diff(actual) {
                        ok = false;
                        eprintln!(
                            "[SIM MISMATCH] tree@({},{}): {diff}",
                            pred_tree.position.x, pred_tree.position.y
                        );
                    }
                }
                None => {
                    ok = false;
                    eprintln!(
                        "[SIM MISMATCH] tree@({},{}) missing in actual state",
                        pred_tree.position.x, pred_tree.position.y
                    );
                }
            }
        }

        if ok {
            eprintln!("[SIM] prediction matched!");
        }
    }
}
