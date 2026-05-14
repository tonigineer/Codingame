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

        // Compare simulation from previous step
        bot.compare(&game.game_state);

        bot.update(&game.game_state);
        bot.eval(&game.game_state);

        // Simulate my own actions to predict next state
        bot.simulate(&game.game_state);

        bot.play();
    }
}
