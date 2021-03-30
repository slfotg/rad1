use super::ChessAgent;
use rand::rngs::ThreadRng;
use rand::Rng;
use shakmaty::{Chess, Color, Position};

pub struct RandomChessAgent {
    pub color: Color,
    pub rng: ThreadRng,
}

impl ChessAgent for RandomChessAgent {
    fn take_turn(&mut self, mut position: Chess) -> Chess {
        super::check_side_to_move(&self.color, &position);
        let moves = position.legal_moves();
        position.play_unchecked(&moves[self.rng.gen_range(0..moves.len())].clone());
        position
    }
}
