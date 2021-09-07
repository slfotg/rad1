use shakmaty::Move;

use crate::game::Game;

mod ab;
mod naive;
mod random;
mod cli;

pub trait ChessAgent {
    fn best_move(&mut self, game: &Game) -> Move;
}

pub fn random_chess_agent() -> impl ChessAgent {
    random::RandomChessAgent::default()
}

pub fn command_line_agent() -> impl ChessAgent {
    cli::CommandLineAgent::default()
}

pub fn naive_chess_agent(depth: usize) -> impl ChessAgent {
    naive::NaiveChessAgent::new(depth)
}

pub fn alpha_beta_agent(depth: usize) -> impl ChessAgent {
    ab::AlphaBetaChessAgent::new(depth)
}
