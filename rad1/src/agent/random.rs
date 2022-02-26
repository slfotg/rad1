use super::ChessAgent;
use crate::Action;
use crate::ChessGame;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;

pub struct RandomChessAgent {
    pub rng: RefCell<ThreadRng>,
}

impl Default for RandomChessAgent {
    fn default() -> Self {
        Self {
            rng: RefCell::new(rand::thread_rng()),
        }
    }
}

impl ChessAgent for RandomChessAgent {
    fn get_action(&self, game: &ChessGame) -> Action {
        let moves = game.current_position().legal_moves();
        Action::MakeMove(moves[self.rng.borrow_mut().gen_range(0..moves.len())])
    }
}
