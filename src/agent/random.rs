use super::ChessAgent;
use rand::rngs::ThreadRng;
use rand::Rng;
use shakmaty::Position;

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
    fn take_turn(&mut self, mut game: Game) -> Game {
        let moves = game.position.legal_moves();
        game.play_mut(&moves[self.rng.gen_range(0..moves.len())]);
        game
    }
}
