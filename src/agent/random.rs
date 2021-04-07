use super::ChessAgent;
use rand::rngs::ThreadRng;
use rand::Rng;
use shakmaty::{Color, Position};

use crate::game::Game;

pub struct RandomChessAgent {
    pub color: Color,
    pub rng: ThreadRng,
}

impl ChessAgent for RandomChessAgent {
    fn take_turn(&mut self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
        let moves = game.position.legal_moves();
        game.play(&moves[self.rng.gen_range(0..moves.len())].clone())
    }
}
