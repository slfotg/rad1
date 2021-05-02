use super::ChessAgent;
use rand::rngs::ThreadRng;
use rand::Rng;
use shakmaty::{Move, Position};

use crate::game::Game;

pub struct RandomChessAgent {
    pub rng: ThreadRng,
}

impl Default for RandomChessAgent {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}

impl ChessAgent for RandomChessAgent {
    fn best_move(&mut self, game: &Game) -> Move {
        let moves = game.position.legal_moves();
        moves[self.rng.gen_range(0..moves.len())].clone()
    }
}
