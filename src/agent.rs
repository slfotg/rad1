use shakmaty::Move;

use crate::game::Game;

mod naive;
mod random;
mod uci;

pub trait ChessAgent {
    fn best_move(&mut self, game: &Game) -> Move;
}

pub fn random_chess_agent() -> impl ChessAgent {
    random::RandomChessAgent::default()
}

pub fn command_line_agent() -> impl ChessAgent {
    uci::UciAgent::default()
}

pub fn naive_chess_agent(depth: usize) -> impl ChessAgent {
    naive::NaiveChessAgent::new(depth)
}
