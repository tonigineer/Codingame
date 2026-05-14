use std::io::{self, BufRead};
use crate::grid::Grid;
use crate::position::Position;

type DummyResult<T> = Result<T, Box<dyn std::error::Error>>;

pub enum Player {
    Me(i32),
    Opp(i32),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Snake {
    id: usize,
    body: Vec<Position>,
}

impl Snake {
    fn new(id: usize, body: Vec<Position>) -> Self {
        Self { id, body }
    }
}

#[allow(dead_code)]
pub struct GameState {
    me: Player,
    opp: Player,
    width: i32,
    height: i32,
    grid: Grid<u8>,
    snakes_per_player: i32,
    my_snakes: Vec<i32>,
    opp_snakes: Vec<i32>,
    snakes: Vec<Snake>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let mut lines = io::stdin().lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();
        let int = |s: &str| s.trim().parse::<i32>().unwrap();

        let my_id = int(&next());
        let opp_id = 1 - my_id;
        let width = int(&next());
        let height = int(&next());

        let rows: Vec<String> = (0..height).map(|_| next()).collect();
        let grid = Grid::from(rows.join("\n"));

        let snakes_per_player = int(&next());
        let my_snakes: Vec<i32> = (0..snakes_per_player).map(|_| int(&next())).collect();
        let opp_snakes: Vec<i32> = (0..snakes_per_player).map(|_| int(&next())).collect();

        Self {
            me: Player::Me(my_id),
            opp: Player::Opp(opp_id),
            width,
            height,
            grid,
            snakes_per_player,
            my_snakes,
            opp_snakes,
            snakes: Vec::new(),
        }
    }

    fn parse_power_source(s: &str) -> DummyResult<Position> {
        let (a, b) = s
            .trim()
            .split_once(' ')
            .ok_or("Split on space did not work.")?;
        Ok(Position::new(a.parse()?, b.parse()?))
    }

    fn parse_snake(s: &str) -> DummyResult<Snake> {
        let (snake_id, pos_string) = s.trim().split_once(' ').ok_or("Split on whitespace")?;
        let body: Vec<Position> = pos_string
            .trim()
            .split(':')
            .map(|p| {
                let (a, b) = p.trim().split_once(',').ok_or("Split on , did not work")?;
                Ok(Position::new(a.parse()?, b.parse()?))
            })
            .collect::<DummyResult<_>>()?;
        Ok(Snake::new(snake_id.parse()?, body))
    }

    pub fn update(&mut self) {
        let mut lines = io::stdin().lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();
        let int = |s: &str| s.trim().parse::<i32>().unwrap();

        let power_source_count = int(&next());
        for _ in 0..power_source_count as usize {
            let pos = GameState::parse_power_source(&next()).unwrap();
            self.grid[pos] = b'P';
        }

        self.grid.print();

        let snakebot_count = int(&next());
        for _i in 0..snakebot_count as usize {
            let snake = GameState::parse_snake(&next()).unwrap();
            self.snakes.push(snake);
        }

        eprintln!("{:?}", self.snakes);
    }
}

pub struct Game {
    pub turn: u8,
    pub game_state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    #[must_use]
    pub fn new() -> Self {
        let game_state = GameState::new();

        Self {
            turn: 0,
            game_state,
        }
    }

    pub fn update(&mut self) {
        self.turn += 1;
        self.game_state.update();
        eprintln!("finished");
    }
}
