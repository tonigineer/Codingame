use crate::entities::{ResourceType};
use crate::game::{Game, Side};

#[rustfmt::skip]
const TROLL_BUILDS: &[[i32; 4]] = &[
    // 2nd troll: balanced harvester
    [1, 1, 1, 1],
    // 3rd troll: stronger all-rounder
    [4, 5, 4, 5],
    // 4th troll: dedicated chopper (no harvest power needed)
    [4, 5, 1, 5],
];

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

    /// Return the costs for the next troll target attributes
    fn cost_next_troll(&self, game: &Game) -> (i32, i32, i32, i32) {
        let num_trolls = game.trolls_for(Side::Me).len();
        let next_troll_attr = TROLL_BUILDS[num_trolls - 1];

        let plums = num_trolls as i32 + next_troll_attr[0].pow(2);
        let lemons = num_trolls as i32 + next_troll_attr[1].pow(2);
        let apples = num_trolls as i32 + next_troll_attr[2].pow(2);
        let iron = num_trolls as i32 + next_troll_attr[3].pow(2);

        (plums, lemons, apples, iron)
    }

    pub fn update(&mut self, game: &Game) {
        (self.plum, self.lemon, self.apple, self.iron) = self.cost_next_troll(game);

        // TODO: logic for banana and wood needs
        // let inv = game.inventory(Side::Me);
        self.banana =  0;
        self.wood = 0;
    }

    /// Return the amount that is currently need for this type of resource
    pub fn need_for_resource(&self, resource_type: ResourceType) -> i32 {
            match resource_type {
                ResourceType::Apple => self.apple,
                ResourceType::Banana => self.banana,
                ResourceType::Lemon => self.lemon,
                ResourceType::Plum => self.plum,
                ResourceType::Iron => self.plum,
                ResourceType::Wood => self.plum,
            }
    }
}
