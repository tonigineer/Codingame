use crate::{grid::Grid, position::Position};

use std::collections::VecDeque;
use std::io;
use std::fmt;

fn read_val<T: std::str::FromStr>() -> T
where
    T::Err: std::fmt::Debug,
{
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().parse().unwrap()
}

#[derive(Debug, Clone, Copy)]
pub enum Player {
    Me(i32),
    Opp(i32),
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub id: i32,
    pub player: Player,
    pub head: Position,
    pub body: VecDeque<Position>,
}

impl Snake {
    fn new(id: i32, player: Player) -> Self {
        Self {
            id,
            player,
            head: Position { x: 0, y: 0 },
            body: VecDeque::new(),
        }
    }
}

impl fmt::Display for Snake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body_str = self
            .body
            .iter()
            .map(|p| format!("({},{})", p.x, p.y))
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            f,
            "Snake {} [{:?}] head:({},{}) body:[{}]",
            self.id, self.player, self.head.x, self.head.y, body_str
        )
    }
}

#[derive(Clone)]
pub struct GameState {
    pub me: Player,
    pub opp: Player,
    pub grid: Grid<u8>,
    pub snakes: Vec<Snake>,
    // my_snake_ids: Vec<i32>,
    // opp_snake_ids: Vec<i32>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    /// Creates a new `GameState` from stdin input.
    ///
    /// # Panics
    /// Panics if reading from stdin fails.
    #[must_use]
    pub fn new() -> Self {
        let my_id: i32 = read_val();

        let me = Player::Me(my_id);
        let opp = Player::Opp(my_id ^ 1);

        let _width: usize = read_val();
        let height: usize = read_val();

        let grid = Grid::from(
            (0..height)
                .map(|_| {
                    let mut input_line = String::new();
                    io::stdin().read_line(&mut input_line).unwrap();
                    input_line.trim().to_string()
                })
                .collect::<Vec<String>>()
                .join("\n")
                .as_str(),
        );

        let mut snakes = Vec::new();

        let snakebots_per_player: i32 = read_val();
        (0..snakebots_per_player).for_each(|_| {
            let id: i32 = read_val();
            snakes.push(Snake::new(id, me));
        });

        (0..snakebots_per_player).for_each(|_| {
            let id: i32 = read_val();
            snakes.push(Snake::new(id, opp));
        });

        Self {
            me,
            opp,
            grid,
            snakes,
        }
    }
    /// Updates the `GameState` from stdin input.
    ///
    /// # Panics
    /// Panics if reading from stdin fails.
    pub fn update(&mut self) {
        let power_source_count: usize = read_val();

        (0..power_source_count).for_each(|_| {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let (x, y): (i32, i32) = {
                let (a, b) = input_line.trim().split_once(' ').unwrap();
                (a.parse().unwrap(), b.parse().unwrap())
            };
            self.grid[Position::new(x, y)] = b'P';
        });

        let snakebot_count: usize = read_val();

        (0..snakebot_count).for_each(|_| {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();
            let snakebot_id = inputs[0].parse::<i32>().unwrap();

            let body: Vec<Position> = inputs[1]
                .split(':')
                .map(|s| {
                    let (x, y) = s.trim().split_once(',').unwrap();
                    Position {
                        x: x.parse().unwrap(),
                        y: y.parse().unwrap(),
                    }
                })
                .collect();

            if let Some(snake) = self.snakes.iter_mut().find(|s| s.id == snakebot_id) {
                snake.head = body[0];
                snake.body = body.into_iter().skip(1).collect();
            }
        });

        self.snakes.iter().for_each(|s| {
            eprintln!("{s:?}");
        });
    }

    // pub fn diff(&self, other: &GameState) {
    //     diff_fields!(self, other, units, opp_units);
}
