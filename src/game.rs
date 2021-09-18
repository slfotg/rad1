use crate::hash::CHESS_HASHER;
use chess::{BitBoard, Board, ChessMove, EMPTY, MoveGen};
use std::cmp::Ordering;

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

    fn current_color_pieces(&self) -> &BitBoard {
        self.position.color_combined(self.position.side_to_move())
    }

    fn opponent_color_pieces(&self) -> &BitBoard {
        self.position.color_combined(!self.position.side_to_move())
    }

    fn is_capture(&self, chess_move: ChessMove) -> bool {
        let square = BitBoard::from_square(chess_move.get_dest());
        if (square & self.opponent_color_pieces()) != EMPTY {
            true
        } else {
            false
        }
    }

    fn is_promotion(&self, chess_move: ChessMove) -> bool {
        if let Some(_) = chess_move.get_promotion() {
            true
        } else {
            false
        }
    }

    pub fn play_mut(&mut self, chess_move: ChessMove) {
        let next_position = self.position.make_move_new(chess_move);
        let next_hash =
            CHESS_HASHER.update_hash(self.hash, &self.position, &next_position, chess_move);
        self.position = next_position;
        self.hash = next_hash;
        self.history.push(next_hash);
    }

    pub fn play(&self, chess_move: ChessMove) -> Self {
        let mut next_game = self.clone();
        next_game.play_mut(chess_move);
        next_game
    }

    #[inline]
    pub fn legal_moves(&self) -> MoveGen {
        MoveGen::new_legal(&self.position)
    }

    pub fn sorted_moves(&self) -> Vec<ChessMove> {
        let mut moves = self.legal_moves().collect::<Vec<ChessMove>>();
        moves.sort_by(|a, b| self.compare_moves(a, b));
        moves
    }

    pub fn captures(&self) -> MoveGen {
        let mut moves = MoveGen::new_legal(&self.position);
        moves.set_iterator_mask(*self.opponent_color_pieces());
        moves
    }

    fn capture_score(&self, a: &ChessMove) -> i8 {
        let values = [1, 3, 3, 5, 9, 0];
        if self.is_capture(*a) {
            values[a.role() as usize] - values[a.capture().unwrap() as usize]
        } else if a.is_promotion() {
            10
        } else {
            i8::MAX
        }
    }

    fn compare_moves(&self, a: &ChessMove, b: &ChessMove) -> Ordering {
        self.capture_score(a).cmp(&self.capture_score(b))
    }

    pub fn sorted_captures(&self) -> Vec<ChessMove> {
        let mut captures = self.captures().collect::<Vec<ChessMove>>();
        captures.sort_by(|a, b| self.compare_moves(a, b));
        captures
    }

    pub fn swap_turn(&self) -> Option<Game> {
        self.position.null_move().map(|p| Self::from_position(p))
    }
}
