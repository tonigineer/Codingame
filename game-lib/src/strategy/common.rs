use crate::strategy::Strategy;
use crate::Game;
use rand::seq::SliceRandom;
use std::fmt::Display;
use std::io::{self, Write};
use std::str::FromStr;

pub struct FirstPossibleMove;

impl Strategy for FirstPossibleMove {
    fn compute_move<G: Game>(&self, game: &G) -> G::Move {
        game.get_possible_moves()
            .next()
            .expect("No moves available, game's done. Should not be called.")
    }
}

pub struct RandomMove;

impl Strategy for RandomMove {
    fn compute_move<G: Game>(&self, game: &G) -> G::Move
    where
        G::Move: Clone,
    {
        let mut rng = rand::thread_rng();
        game.get_possible_moves()
            .collect::<Vec<G::Move>>()
            .choose(&mut rng)
            .map(|x| x.clone())
            .expect("No moves available, game's done. Should not be called.")
    }
}

pub fn prompt_user_move<G: Game>(game: &G) -> G::Move
where
    G::Move: Clone + Eq + FromStr + Display,
    <G::Move as FromStr>::Err: Display,
{
    let legal: Vec<G::Move> = game.get_possible_moves().collect();

    loop {
        print!("Your move ({}): ", game.get_current_player_symbol());
        let _ = io::stdout().flush();

        let mut s = String::new();
        if io::stdin().read_line(&mut s).is_err() {
            println!("Couldn't read input. Try again.");
            continue;
        }

        match s.trim().parse::<G::Move>() {
            Ok(m) if legal.contains(&m) => return m,
            Ok(m) => println!("‘{}’ isn’t legal this turn. Try again.", m),
            Err(e) => println!("Can’t parse that: {}. Try again.", e),
        }
    }
}
