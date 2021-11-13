use crate::eval::naive::NaiveEvaluator;
use crate::tt::TranspositionTable;
use chess::{Action, Game};

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
    tt: TranspositionTable<i16>,
    evaluator: NaiveEvaluator,
) -> ab::AlphaBetaChessAgent {
    ab::AlphaBetaChessAgent::new(depth, tt, evaluator)
}
