use super::ChessAgent;
use chess::{Action, ChessMove, Game, MoveGen};
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
    fn get_action(&self, game: &Game) -> Action {
        let moves = MoveGen::new_legal(&game.current_position()).collect::<Vec<ChessMove>>();
        Action::MakeMove(moves[self.rng.borrow_mut().gen_range(0..moves.len())])
    }
}
