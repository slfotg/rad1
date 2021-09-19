use super::ChessAgent;
use chess::{Board, ChessMove, MoveGen};
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
    fn best_move(&mut self, board: &Board) -> ChessMove {
        let moves = MoveGen::new_legal(board).collect::<Vec<ChessMove>>();
        moves[self.rng.gen_range(0..moves.len())]
    }
}
