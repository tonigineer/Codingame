use crate::entities::{Inventory, Tree, Troll};
use crate::game::{Action, Game, Side};
use crate::position::Position;

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
    carry_iron: i32,
    carry_wood: i32,
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
            carry_iron: t.carry_iron,
            carry_wood: t.carry_wood,
        }
    }

    fn diff(&self, actual: &Troll) -> Vec<String> {
        let mut diffs = Vec::new();
        let checks: &[(&str, i32, i32)] = &[
            ("carry_plum", self.carry_plum, actual.carry_plum),
            ("carry_lemon", self.carry_lemon, actual.carry_lemon),
            ("carry_apple", self.carry_apple, actual.carry_apple),
            ("carry_banana", self.carry_banana, actual.carry_banana),
            ("carry_iron", self.carry_iron, actual.carry_iron),
            ("carry_wood", self.carry_wood, actual.carry_wood),
        ];
        if self.position != actual.position {
            diffs.push(format!(
                "position: predicted {:?} got {:?}",
                self.position, actual.position
            ));
        }
        for (name, pred, act) in checks {
            if pred != act {
                diffs.push(format!("{name}: predicted {pred} got {act}"));
            }
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
    iron: i32,
    wood: i32,
}

impl SnapshotInventory {
    fn from(r: &Inventory) -> Self {
        Self {
            plum: r.plum.amount,
            lemon: r.lemon.amount,
            apple: r.apple.amount,
            banana: r.banana.amount,
            iron: r.iron.amount,
            wood: r.wood.amount,
        }
    }

    fn diff(&self, r: &Inventory) -> Vec<String> {
        let mut diffs = Vec::new();
        let checks: &[(&str, i32, i32)] = &[
            ("plum", self.plum, r.plum.amount),
            ("lemon", self.lemon, r.lemon.amount),
            ("apple", self.apple, r.apple.amount),
            ("banana", self.banana, r.banana.amount),
            ("iron", self.iron, r.iron.amount),
            ("wood", self.wood, r.wood.amount),
        ];
        for (name, pred, act) in checks {
            if pred != act {
                diffs.push(format!("{name}: predicted {pred} got {act}"));
            }
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
    health: i32,
}

impl SnapshotTree {
    fn from(t: &Tree) -> Self {
        Self {
            position: t.position,
            fruits: t.fruits,
            size: t.size,
            cooldown: t.cooldown,
            health: t.health,
        }
    }

    fn diff(&self, actual: &Tree) -> Vec<String> {
        let mut diffs = Vec::new();
        let checks: &[(&str, i32, i32)] = &[
            ("fruits", self.fruits, actual.fruits),
            ("size", self.size, actual.size),
            ("cooldown", self.cooldown, actual.cooldown),
            ("health", self.health, actual.health),
        ];
        for (name, pred, act) in checks {
            if pred != act {
                diffs.push(format!("{name}: predicted {pred} got {act}"));
            }
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

        for diff in snapshot.inventory.diff(self.inventory(Side::Me)) {
            ok = false;
            eprintln!("[SIM MISMATCH] inventory: {diff}");
        }

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

        for pred_tree in &snapshot.trees {
            let opp_on_tree = self
                .trolls
                .iter()
                .any(|t| t.side == Side::Opp && t.position == pred_tree.position);

            if opp_on_tree {
                continue;
            }

            match self
                .trees
                .iter()
                .find(|t| t.position == pred_tree.position)
            {
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
                    // Tree might have been chopped by opponent
                    if !opp_on_tree {
                        ok = false;
                        eprintln!(
                            "[SIM MISMATCH] tree@({},{}) missing in actual state",
                            pred_tree.position.x, pred_tree.position.y
                        );
                    }
                }
            }
        }

        if ok {
            eprintln!("[SIM] prediction matched!");
        }
    }
}
