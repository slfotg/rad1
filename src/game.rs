use crate::hash::CHESS_HASHER;
use chess::{BitBoard, Board, BoardStatus, Color, ChessMove, MoveGen, EMPTY};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    hash: u64,
}

impl Default for Game {
    fn default() -> Self {
        Self::from_board(Board::default())
    }
}

impl Game {
    pub fn new(board: Board, hash: u64) -> Self {
        let mut history = Vec::with_capacity(200);
        history.push(hash);
        Self {
            board,
            hash,
        }
    }

    pub fn from_board(board: Board) -> Self {
        let hash = CHESS_HASHER.hash(&board);
        Self::new(board, hash)
    }

    pub fn get_board(&self) -> Board {
        self.board
    }

    pub fn turn(&self) -> Color {
        self.board.side_to_move()
    }

    pub fn is_game_over(&self) -> bool {
        match self.board.status() {
            BoardStatus::Ongoing => false,
            _ => true,
        }
    }

    pub fn is_check(&self) -> bool {
        self.board.checkers().popcnt() > 0
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    fn opponent_color_pieces(&self) -> &BitBoard {
        self.board.color_combined(!self.board.side_to_move())
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

    pub fn is_legal(&self, chess_move: ChessMove) -> bool {
        self.board.legal(chess_move)
    }

    pub fn play_mut(&mut self, chess_move: ChessMove) {
        let next_board = self.board.make_move_new(chess_move);
        let next_hash =
            CHESS_HASHER.update_hash(self.hash, &self.board, &next_board);
        self.board = next_board;
        self.hash = next_hash;
    }

    pub fn play(&self, chess_move: ChessMove) -> Self {
        let mut next_game = self.clone();
        next_game.play_mut(chess_move);
        next_game
    }

    #[inline]
    pub fn legal_moves(&self) -> MoveGen {
        MoveGen::new_legal(&self.board)
    }

    pub fn sorted_moves(&self) -> Vec<ChessMove> {
        let mut moves = self.legal_moves().collect::<Vec<ChessMove>>();
        moves.sort_by(|a, b| self.compare_moves(a, b));
        moves
    }

    pub fn captures(&self) -> MoveGen {
        let mut moves = MoveGen::new_legal(&self.board);
        moves.set_iterator_mask(*self.opponent_color_pieces());
        moves
    }

    fn capture_score(&self, a: &ChessMove) -> i8 {
        let values = [1, 3, 3, 5, 9, 0];
        if self.is_capture(*a) {
            values[self.board.piece_on(a.get_source()).unwrap() as usize]
                - values[self.board.piece_on(a.get_dest()).unwrap() as usize]
        } else if self.is_promotion(*a) {
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
        self.board.null_move().map(|board| Self {
            board,
            hash: CHESS_HASHER.update_color_hash(self.hash)
        })
    }
}
