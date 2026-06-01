mod bot;
mod game;
mod utils;

use std::time::Instant;

fn main() {
    let mut game = game::Game::new();
    let mut bot = bot::Bot::new();

    loop {
        let turn_start = Instant::now();
        game.update();

        // let t0 = Instant::now();
        // game.make_snapshot();
        // let compare_us = t0.elapsed().as_micros();

        let t0 = Instant::now();
        bot.play(&mut game);
        let play_us = t0.elapsed().as_micros();

        // let t0 = Instant::now();
        // game.simulate(&bot.actions);
        // let simulate_us = t0.elapsed().as_micros();

        game::Game::output(&bot.actions);

        // eprintln!(
        //     "[TIMING] compare: {}µs | think: {}µs | simulate: {}µs | total: {}µs",
        //     compare_us,
        //     think_us,
        //     simulate_us,
        //     turn_start.elapsed().as_micros()
        // );

        eprintln!(
            "[TIMING] play: {}µs | total: {}µs",
            play_us,
            turn_start.elapsed().as_micros()
        );
    }
}
