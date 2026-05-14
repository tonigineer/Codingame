pub mod bot;
pub mod game;
pub mod grid;
pub mod position;
pub mod types;

fn main() {
    let mut game = game::Game::new();
    // let mut bot = bot::Bot::new();

    loop {
        // game.game_state.grid.print();
        game.update();
        // bot.play(&mut game);

        eprintln!("Should be here now");
        println!("WAIT");
    }
}
