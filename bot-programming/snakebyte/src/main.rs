pub mod bot;
pub mod game;
pub mod types;

fn main() {
    let mut game = game::Game::default();
    let bot = bot::Bot::new();

    loop {
        game.update();
        bot.play(&game);
    }
}
