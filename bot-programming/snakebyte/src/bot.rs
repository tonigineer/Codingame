use crate::game::Game;
pub struct Bot {
    name: &'static str,
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self { name: "Snakebyte" }
    }

    pub fn play(&self, game: &Game) {
        println!("GAME: {:?} - TURN: {:?}", self.name, game.turn);
    }
}
