pub mod bot;
pub mod game;
pub mod grid;
pub mod position;
pub mod prediction;
pub mod types;

use std::time::Instant;

fn main() {
    let mut game = game::Game::new();
    let mut bot = bot::Bot::new();

    loop {
        game.update();
        let turn_start = Instant::now();

        let t0 = Instant::now();
        bot.compare(&game.game_state);
        let compare_us = t0.elapsed().as_micros();

        let t0 = Instant::now();
        bot.update(&game.game_state);
        bot.think(&game.game_state);
        let think_us = t0.elapsed().as_micros();

        let t0 = Instant::now();
        bot.simulate(&game.game_state);
        let simulate_us = t0.elapsed().as_micros();

        bot.play();

        eprintln!(
            "[TIMING] compare: {}µs | think: {}µs | simulate: {}µs | total: {}µs",
            compare_us,
            think_us,
            simulate_us,
            turn_start.elapsed().as_micros()
        );
    }
}
