use crate::entities::{ResourceType, Tree, TreeType};
use crate::game::{Game, Side};

pub struct Priority {
    pub apple: i32,
    pub banana: i32,
    pub lemon: i32,
    pub plum: i32,
    pub iron: i32,
    pub wood: i32,
}

impl Priority {
    pub fn new() -> Self {
        Self {
            apple: 0,
            banana: 0,
            lemon: 0,
            plum: 0,
            iron: 0,
            wood: 0,
        }
    }

    pub fn update(&mut self, game: &Game) {
        let inv = game.inventory(Side::Me);

        let min_fruit_stock = 16;
        let min_iron_stock = 10;

        self.apple = (min_fruit_stock - inv.get_by_tree(&TreeType::Apple)).max(0);
        self.banana = (min_fruit_stock - inv.get_by_tree(&TreeType::Banana)).max(0);
        self.lemon = (min_fruit_stock - inv.get_by_tree(&TreeType::Lemon)).max(0);
        self.plum = (min_fruit_stock - inv.get_by_tree(&TreeType::Plum)).max(0);
        self.iron = (min_iron_stock - inv.get(ResourceType::Iron)).max(0);
        self.wood = (180 / game.turns_remaining().max(1)).min(1);
    }

    pub fn weight_for_tree(&self, tree: &Tree) -> i32 {
        self.weight_for_type(tree.typ)
    }

    pub fn weight_for_type(&self, typ: TreeType) -> i32 {
        match typ {
            TreeType::Apple => self.apple,
            TreeType::Banana => self.banana,
            TreeType::Lemon => self.lemon,
            TreeType::Plum => self.plum,
        }
    }
}
