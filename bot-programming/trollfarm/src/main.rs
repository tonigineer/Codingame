pub mod entities;
pub mod game;
pub mod grid;
pub mod player;
pub mod position;
pub mod prediction;

use std::time::Instant;

fn main() {
    let mut game = game::Game::new();
    let mut me = player::Player::new(game::Side::Me);

    loop {
        game.update();
        let turn_start = Instant::now();

        let t0 = Instant::now();
        me.compare(&game);
        let compare_us = t0.elapsed().as_micros();

        let t0 = Instant::now();
        me.think(&game);
        let think_us = t0.elapsed().as_micros();

        let t0 = Instant::now();
        me.simulate(&game);
        let simulate_us = t0.elapsed().as_micros();

        game::Game::output(&me.actions);

        eprintln!(
            "[TIMING] compare: {}µs | think: {}µs | simulate: {}µs | total: {}µs",
            compare_us,
            think_us,
            simulate_us,
            turn_start.elapsed().as_micros()
        );
    }
}
