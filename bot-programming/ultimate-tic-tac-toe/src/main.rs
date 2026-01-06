use game_lib::games::TicTacToe;
pub mod parse;

fn main() {
    loop {
        let ttt = TicTacToe::new();
        let (opp, moves) = crate::parse::read_input();

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");
        println!("0 0");
    }
}
