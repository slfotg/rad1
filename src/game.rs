use crate::hash::CHESS_HASHER;
use chess::{Board, ChessMove, MoveGen};
//use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Game {
    pub position: Board,
    pub hash: u64,
    pub history: Vec<u64>,
}

impl Default for Game {
    fn default() -> Self {
        Self::from_position(Board::default())
    }
}

impl Game {
    pub fn new(position: Board, hash: u64) -> Self {
        let mut history = Vec::with_capacity(200);
        history.push(hash);
        Self { position, hash, history }
    }

    pub fn from_position(position: Board) -> Self {
        let hash = CHESS_HASHER.hash(&position);
        Self::new(position, hash)
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn play_mut(&mut self, chess_move: &ChessMove) {
        let next_position = self.position.make_move_new(*chess_move);
        let next_hash =
            CHESS_HASHER.update_hash(self.hash, &self.position, &next_position, chess_move);
        self.position = next_position;
        self.hash = next_hash;
        self.history.push(next_hash);
    }

    pub fn play(&self, chess_move: &ChessMove) -> Self {
        let mut next_game = self.clone();
        next_game.play_mut(chess_move);
        next_game
    }

    #[inline]
    pub fn legal_moves(&self) -> MoveGen {
        MoveGen::new_legal(&self.position)
    }

    pub fn sorted_moves(&self) -> MoveGen {
        let mut moves = self.legal_moves();
        // TODO sorting... moves.sort_by(Self::compare_moves);
        moves
    }

    pub fn captures(&self) -> MoveGen {
        let mut moves = MoveGen::new_legal(&self.position);
        let targets = self.position.color_combined(!self.position.side_to_move());
        moves.set_iterator_mask(*targets);
        moves
    }

    // fn capture_score(a: &ChessMove) -> i8 {
    //     let values = [0, 1, 3, 3, 5, 9, 0];
    //     if a.is_capture() {
    //         values[a.role() as usize] - values[a.capture().unwrap() as usize]
    //     } else if a.is_promotion() {
    //         10
    //     } else {
    //         i8::MAX
    //     }
    // }

    // fn compare_moves(a: &Move, b: &Move) -> Ordering {
    //     Self::capture_score(a).cmp(&Self::capture_score(b))
    // }

    // pub fn sorted_captures(&self) -> MoveList {
    //     let mut captures = self.captures();
    //     captures.sort_by(Self::compare_moves);
    //     captures
    // }

    pub fn swap_turn(&self) -> Option<Game> {
        self.position.null_move().map(|p| Self::from_position(p))
    }
}
