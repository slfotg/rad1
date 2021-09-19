use super::ChessAgent;
use chess::{Action, ChessMove, Game, MoveGen};
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
    fn get_action(&mut self, game: &Game) -> Action {
        let moves = MoveGen::new_legal(&game.current_position()).collect::<Vec<ChessMove>>();
        Action::MakeMove(moves[self.rng.gen_range(0..moves.len())])
    }
}
