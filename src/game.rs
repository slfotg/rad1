use shakmaty::{Chess, Move, Position};

use crate::hash::CHESS_HASHER;

#[derive(Debug, Clone)]
pub struct Game {
    pub position: Chess,
    pub hash: i64,
}

impl Default for Game {
    fn default() -> Self {
        let position = Chess::default();
        let hash = CHESS_HASHER.hash(&position);
        Self { position, hash }
    }
}

impl Game {
    pub fn new(position: Chess, hash: i64) -> Self {
        Self { position, hash }
    }

    pub fn play_mut(&mut self, chess_move: &Move) {
        let mut next_position = self.position.clone();
        next_position.play_unchecked(chess_move);
        let next_hash =
            CHESS_HASHER.update_hash(self.hash, &self.position, &next_position, chess_move);
        self.position = next_position;
        self.hash = next_hash;
    }

    pub fn play(&self, chess_move: &Move) -> Self {
        let mut next_game = self.clone();
        next_game.play_mut(chess_move);
        next_game
    }
}
