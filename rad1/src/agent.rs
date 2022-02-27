use crate::tt::TranspositionTable;
use crate::Action;
use crate::ChessGame;

mod ab;
mod cli;
mod random;

/// A ChessAgent determines what [`Action`] to take given the
/// current state of the chess game
///
/// This trait is designed so implementations can be (but not required to be)
/// stateless since chess is
/// a [Perfect Information](https://en.wikipedia.org/wiki/Perfect_information)
/// game, which basically means that all of the information you need to make
/// a perfect decision is given to you from the current game state,
/// represented by [`ChessGame`].
pub trait ChessAgent {
    /// Given the current game state,
    /// determine what the best action to take is.
    ///
    /// # Arguments
    ///
    /// * `game` - A [`ChessGame`] representing the current game state
    fn get_action(&self, game: &ChessGame) -> Action;
}

/// Returns a [`ChessAgent`] that makes a valid but random [`Action`]
///
/// This is pretty useless but acts a control for manual testing
/// and adding new features.
pub fn random_chess_agent() -> random::RandomChessAgent {
    random::RandomChessAgent::default()
}

/// Returns a [`ChessAgent`] that creates an [`Action`] from stdin.
///
/// This is mainly used for playing against the computer from the terminal.
pub fn command_line_agent() -> cli::CommandLineAgent {
    cli::CommandLineAgent::default()
}

/// Returns the main [`ChessAgent`] used by this Chess Engine.
pub fn alpha_beta_agent(depth: u8, tt: TranspositionTable<i16>) -> ab::AlphaBetaChessAgent {
    ab::AlphaBetaChessAgent::new(depth, tt)
}
