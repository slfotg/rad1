use shakmaty::{Chess, Move, MoveList, Position};
use std::cmp::Ordering;

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

    #[inline]
    pub fn hash(&self) -> i64 {
        self.hash
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

    #[inline]
    pub fn legal_moves(&self) -> MoveList {
        self.position.legal_moves()
    }

    pub fn sorted_moves(&self) -> MoveList {
        let mut moves = self.legal_moves();
        moves.sort_by(Self::compare_moves);
        moves
    }

    pub fn captures(&self) -> MoveList {
        self.legal_moves()
            .into_iter()
            .filter(Move::is_capture)
            .collect()
    }

    fn capture_score(a: &Move) -> i8 {
        let values = [0, 1, 3, 3, 5, 9, 0];
        if a.is_capture() {
            values[a.role() as usize] - values[a.capture().unwrap() as usize]
        } else if a.is_promotion() {
            10
        } else {
            i8::MAX
        }
    }

    fn compare_moves(a: &Move, b: &Move) -> Ordering {
        Self::capture_score(a).cmp(&Self::capture_score(b))
    }

    pub fn sorted_captures(&self) -> MoveList {
        let mut captures = self.captures();
        captures.sort_by(Self::compare_moves);
        captures
    }
}
