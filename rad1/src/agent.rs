use chess::{Action, Game};
//use crate tt::TranspositionTable;
use crate::eval::Evaluator;
use crate::tt::TranspositionTable;

mod ab;
mod cli;
mod random;

pub trait ChessAgent {
    fn get_action(&self, game: &Game) -> Action;
}

pub fn random_chess_agent() -> random::RandomChessAgent {
    random::RandomChessAgent::default()
}

pub fn command_line_agent() -> cli::CommandLineAgent {
    cli::CommandLineAgent::default()
}

pub fn alpha_beta_agent(
    depth: u8,
    tt: TranspositionTable,
    evaluator: Box<dyn Evaluator<Result = i16>>,
) -> ab::AlphaBetaChessAgent {
    ab::AlphaBetaChessAgent::new(depth, tt, evaluator)
}
