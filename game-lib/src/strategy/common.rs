use crate::Game;

use rand::seq::SliceRandom;
use std::io::{self, Write};

pub struct FirstPossibleMove;

impl Strategy for FirstPossibleMove {
    fn compute_move<G: Game>(&self, game: &G) -> usize {
        game.get_possible_moves().next().unwrap()
    }
}

pub struct RandomMove;

impl Strategy for RandomMove {
    fn compute_move<G: Game>(&self, game: &G) -> usize {
        let mut rng = rand::thread_rng();
        let moves: Vec<usize> = game.get_possible_moves().collect();
        moves.choose(&mut rng).copied().expect("No moves available")
    }
}

pub trait Strategy {
    fn compute_move<G: Game>(&self, game: &G) -> usize;
}

pub fn prompt_user_move<G: Game>(game: &G) -> usize {
    let legal: Vec<usize> = game.get_possible_moves().collect();

    loop {
        print!("Your move ({}): ", game.get_current_player_symbol());
        let _ = io::stdout().flush();

        let mut s = String::new();
        if io::stdin().read_line(&mut s).is_err() {
            println!("Couldn't read input. Try again.");
            continue;
        }

        match s.trim().parse::<usize>() {
            Ok(m) if legal.contains(&m) => return m,
            Ok(m) => println!("‘{}’ isn’t legal this turn. Try again.", m),
            Err(e) => println!("Can’t parse that: {}. Try again.", e),
        }
    }
}
