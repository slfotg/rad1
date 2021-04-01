use rand;
use shakmaty::{Chess, Color, Position};

mod naive;
mod random;
mod uci;

pub trait ChessAgent {
    fn take_turn(&mut self, position: Chess) -> Chess;
}

pub fn random_chess_agent(color: Color) -> impl ChessAgent {
    random::RandomChessAgent {
        color,
        rng: rand::thread_rng(),
    }
}

pub fn command_line_agent(color: Color) -> impl ChessAgent {
    uci::UciAgent { color }
}

pub fn naive_chess_agent(color: Color, depth: usize) -> impl ChessAgent {
    naive::NaiveChessAgent { color, depth }
}

fn check_side_to_move(color: &Color, position: &impl Position) {
    if position.turn() != *color {
        panic!("Wrong color's turn to move");
    }
}
