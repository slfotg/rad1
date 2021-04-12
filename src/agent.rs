use shakmaty::{Color, Setup};

use crate::game::Game;

mod naive;
mod random;
mod uci;

pub trait ChessAgent {
    fn take_turn(&self, game: Game) -> Game;
}

pub fn random_chess_agent(color: Color) -> impl ChessAgent {
    random::RandomChessAgent::new(color)
}

pub fn command_line_agent(color: Color) -> impl ChessAgent {
    uci::UciAgent::new(color)
}

pub fn naive_chess_agent(color: Color, depth: usize) -> impl ChessAgent {
    naive::NaiveChessAgent::new(color, depth)
}

fn check_side_to_move(color: Color, game: &Game) {
    if game.position.turn() != color {
        panic!("Wrong color's turn to move");
    }
}
