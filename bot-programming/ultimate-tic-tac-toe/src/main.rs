use game_lib::games::TicTacToe;
pub mod parse;

fn main() {
    loop {
        let ttt = TicTacToe::new();
        let (opp, moves) = crate::parse::read_input();

        println!("0 0");
        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");
    }
}
