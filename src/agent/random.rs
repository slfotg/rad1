use super::ChessAgent;
use crate::game::Game;
use chess::ChessMove;
use rand::rngs::ThreadRng;
use rand::Rng;

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
    fn best_move(&mut self, game: &Game) -> ChessMove {
        let moves = game.legal_moves().collect::<Vec<ChessMove>>();
        moves[self.rng.gen_range(0..moves.len())].clone()
    }
}
