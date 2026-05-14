use crate::game::Game;

#[derive(Debug, Clone)]
enum Move {
    Wait,
}

impl Move {
    fn to_string(&self) -> String {
        match self {
            Move::Wait => "Wait".to_string(),
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

        println!("{}", mv);
    }
}
