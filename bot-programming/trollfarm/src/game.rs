use std::io::{self, BufRead};
use crate::grid::Grid;
use crate::position::Position;
use crate::types::{Player, Resources, Tree, Troll};

pub struct GameState {
    pub me: Player,
    pub opp: Player,
    pub width: usize,
    pub height: usize,
    pub grid: Grid<u8>,
    pub my_shack: Position,
    pub opp_shack: Position,
    pub my_resources: Resources,
    pub opp_resources: Resources,
    pub my_score: i32,
    pub opp_score: i32,
    pub trees: Vec<Tree>,
    pub trolls: Vec<Troll>,
}

impl GameState {
    pub fn new() -> Self {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();

        let (width, height) = next()
            .split_once(" ")
            .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()))
            .unwrap();

        let rows: Vec<String> = (0..height).map(|_| next()).collect();
        let grid = Grid::from(rows.join("\n"));

        let my_shack = grid.search(b'0').unwrap();
        let opp_shack = grid.search(b'1').unwrap();

        Self {
            me: Player::Me,
            opp: Player::Opp,
            width,
            height,
            grid,
            my_shack,
            opp_shack,
            my_resources: Resources::new(),
            opp_resources: Resources::new(),
            my_score: 0,
            opp_score: 0,
            trees: Vec::new(),
            trolls: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();

        self.my_resources = Resources::parse(&next());
        self.opp_resources = Resources::parse(&next());

        let tree_count: usize = next().trim().parse().unwrap();
        self.trees = (0..tree_count).map(|_| Tree::parse(&next())).collect();

        for tree in &self.trees {
            self.grid[tree.position] = tree.typ.to_byte();
        }

        self.grid.print();

        let troll_count: usize = next().trim().parse().unwrap();
        self.trolls = (0..troll_count).map(|_| Troll::parse(&next())).collect();
    }

    fn winner(&self) -> Option<Player> {
        match self.my_score.cmp(&self.opp_score) {
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(Player::Me),
            std::cmp::Ordering::Less => Some(Player::Opp),
        }
    }

    fn evaluate_moves(&mut self, my_moves: Vec<Command>, opp_moves: Vec<Command>) {}
}

pub enum Command {
    Move(i32, Position),
    Harvest(i32),
    Drop(i32),
}

pub struct Game {
    pub turn: u8,
    pub game_state: GameState,
}

impl Game {
    #[must_use]
    pub fn new() -> Self {
        Self {
            turn: 0,
            game_state: GameState::new(),
        }
    }

    pub fn update(&mut self) {
        self.turn += 1;
        self.game_state.update();
        eprintln!("Turn {} finished", self.turn);
    }
}
