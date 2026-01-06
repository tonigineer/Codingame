mod bot;
mod game;
mod types;

use std::collections::HashMap;
use std::time::Instant;

use bot::Bot;
use game::Game;
use types::{Agent, Command};

use crate::types::PlayingSide;

fn main() {
    let mut game = Game::new();

    loop {
        game.update_turn();

        let start = Instant::now();
        let mut iteration = 0;
        let mut best_commands: HashMap<Agent, (Command, Command)> = HashMap::new();

        let mut player = Bot::new(&PlayingSide::Player, &game);
        let opponent = Bot::new(&PlayingSide::Opponent, &game);

        loop {
            iteration += 1;
            let temp_game = game.clone();

            let player_commands = player.play(&temp_game);

            // let mut temp_counter_game = game.clone();
            // temp_counter_game.apply_moves(&player_commands);
            // let opponent_commands = opponent.counter_play(&temp_counter_game, &player_commands);
            // temp_game.apply_both_player(&player_commands, &oppenent_commands);

            // let player_score, opponent_score = temp_game.referee();
            best_commands = player_commands;

            let now = Instant::now();
            if now.duration_since(start).as_millis() > 49 {
                eprintln!("{}", iteration);
                break;
            }

            break;
        }

        game.output_commands(&best_commands);
    }
}
