use crate::game::Game;

use std::fmt;

#[derive(Debug, Clone)]
enum Move {
    Wait,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Wait => write!(f, "Wait"),
        }
    }
}

pub struct Bot {
    _name: &'static str,
    moves: Vec<Move>,
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            _name: "Snakebyte",
            moves: Vec::new(),
        }
    }

    pub fn think(&mut self, _game: &Game) {
        self.moves = Vec::from([Move::Wait]);
    }

    pub fn play(&mut self, _game: &Game) {
        let mv = self
            .moves
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>()
            .join(";");

        println!("{mv}");
    }
}
