use super::ChessAgent;
use rand::rngs::ThreadRng;
use rand::Rng;
use shakmaty::{Color, Position};
use std::cell::RefCell;

use crate::game::Game;

pub struct RandomChessAgent {
    color: Color,
    rng: RefCell<ThreadRng>,
}

impl RandomChessAgent {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            rng: RefCell::new(rand::thread_rng()),
        }
    }
}

impl ChessAgent for RandomChessAgent {
    fn take_turn(&self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
        let moves = game.position.legal_moves();
        game.play(&moves[self.rng.borrow_mut().gen_range(0..moves.len())].clone())
    }
}
