use crate::tt::TranspositionTable;
use crate::Action;
use crate::ChessGame;

mod ab;
mod cli;
mod random;

pub trait ChessAgent {
    fn get_action(&self, game: &ChessGame) -> Action;
}

pub fn random_chess_agent() -> random::RandomChessAgent {
    random::RandomChessAgent::default()
}

pub fn command_line_agent() -> cli::CommandLineAgent {
    cli::CommandLineAgent::default()
}

pub fn alpha_beta_agent(depth: u8, tt: TranspositionTable<i16>) -> ab::AlphaBetaChessAgent {
    ab::AlphaBetaChessAgent::new(depth, tt)
}
