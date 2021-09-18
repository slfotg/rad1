use crate::game::Game;
use chess::ChessMove;

mod ab;
mod random;
mod cli;

pub trait ChessAgent {
    fn best_move(&mut self, game: &Game) -> ChessMove;
}

pub fn random_chess_agent() -> random::RandomChessAgent {
    random::RandomChessAgent::default()
}

pub fn command_line_agent() -> cli::CommandLineAgent {
    cli::CommandLineAgent::default()
}

pub fn alpha_beta_agent(depth: usize) -> ab::AlphaBetaChessAgent {
    ab::AlphaBetaChessAgent::new(depth)
}
