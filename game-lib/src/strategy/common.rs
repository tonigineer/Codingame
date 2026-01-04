use crate::strategy::minimax;
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

pub struct MinimaxMove;

impl Strategy for MinimaxMove {
    fn compute_move<G: Game>(&self, game: &G) -> G::Move
    where
        G: Clone,
        G::Move: Clone,
        <G as Game>::PlayerMask: Eq,
    {
        let mut new_game = game.clone();
        let side = game.get_current_player();

        let mut minimax = minimax::Minimax::new(9);
        let mv = minimax.get_move(&mut new_game, side);

        mv
        // game.get_possible_moves()
        //     .next()
        //     .expect("No moves available, game's done. Should not be called.")

        // let start = std::time::Instant::now();
        // let duration = start.elapsed();

        // if self.verbose {
        //     println!("Minimax will play: {}", mv);
        //     println!("- states cached = {}", minimax.transpositions.len());
        //     println!("- duration = {:?}", duration);
        //     println!("Press Enter to continue...");

        //     let mut dummy = String::new();
        //     std::io::stdin().read_line(&mut dummy).ok();
        // }
    }
}

pub trait Strategy {
    fn compute_move<G: Game>(&self, game: &G) -> G::Move;
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
