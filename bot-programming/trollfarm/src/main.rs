pub mod bot;
pub mod game;
pub mod grid;
pub mod position;
pub mod types;

fn main() {
    let mut game = game::Game::new();
    let mut bot = bot::Bot::new();

    loop {
        game.update();
        bot.update(&game.game_state);

        bot.play(&game.game_state);

        println!("WAIT");
    }
}
