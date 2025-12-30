/// Represents the mark (X or O) for a Tic-Tac-Toe player.
///
/// # Examples
///
/// ```
/// use tic_tac_toe::PlayerMark;
/// let mark = PlayerMark::X;
/// assert_eq!(mark, PlayerMark::X);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMark {
    X,
    O,
}

/// A type alias for a move in Tic-Tac-Toe, representing a board position (0-8).
///
/// # Examples
///
/// ```
/// let m: tic_tac_toe::Move = 4; // Center of the board
/// assert_eq!(m, 4);
/// ```
pub type Move = u8;

/// Represents the game board using bitboards for efficient storage.
///
/// # Examples
///
/// ```
/// use tic_tac_toe::{BitBoards, PlayerMark};
/// let board = BitBoards { x_board: 1, o_board: 2 };
/// assert_eq!(board.get(PlayerMark::X), 1);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BitBoards {
    pub x_board: u16,
    pub o_board: u16,
}

impl BitBoards {
    /// Returns the bitboard for the given player mark.
    ///
    /// # Examples
    ///
    /// ```
    /// use tic_tac_toe::{BitBoards, PlayerMark};
    /// let board = BitBoards { x_board: 1, o_board: 2 };
    /// assert_eq!(board.get(PlayerMark::X), 1);
    /// assert_eq!(board.get(PlayerMark::O), 2);
    /// ```
    pub fn get(&self, mark: PlayerMark) -> u16 {
        match mark {
            PlayerMark::X => self.x_board,
            PlayerMark::O => self.o_board,
        }
    }
}

/// Represents the state of a Tic-Tac-Toe game.
///
/// # Examples
///
/// ```
/// use tic_tac_toe::TicTacToe;
/// let game = TicTacToe::new();
/// // The game is initialized
/// ```
#[derive(Debug)]
pub struct TicTacToe {
    pub board: BitBoards,
    pub current_player: PlayerMark,
    pub past_moves: [Move; 9],
}

impl TicTacToe {
    /// Creates a new Tic-Tac-Toe game with an empty board and X as the starting player.
    ///
    /// # Examples
    ///
    /// ```
    /// use tic_tac_toe::{TicTacToe, PlayerMark};
    /// let game = TicTacToe::new();
    /// assert_eq!(game.current_player, PlayerMark::X);
    /// ```
    pub fn new() -> Self {
        Self {
            board: BitBoards {
                x_board: 0u16,
                o_board: 0u16,
            },
            current_player: PlayerMark::X,
            past_moves: [0; 9],
        }
    }
}

impl games::Game for TicTacToe {
    type Player = PlayerMark;
    type Move = Move;

    fn first_move() -> Self {
        Self::new()
    }

    fn get_possible_moves(&self) -> Vec<Move> {
        Vec::with_capacity(9)
    }

    fn apply_move(&mut self, _move: Move) -> Result<(), String> {
        Ok(())
    }

    fn undo_move(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn current_player(&self) -> PlayerMark {
        self.current_player
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_winner(&self) -> Option<PlayerMark> {
        None
    }

    fn render(&self) {
        unimplemented!("Rendering for tic-tac-toe not implemented yet")
    }
}
