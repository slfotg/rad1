use chess::{BitBoard, Board, ChessMove, MoveGen};
use std::cmp::Ordering;

pub struct MoveSorter;

pub const MOVE_SORTER: MoveSorter = MoveSorter;

impl MoveSorter {
    #[inline]
    pub fn sorted_moves(&self, board: &Board) -> Vec<ChessMove> {
        let mut moves = MoveGen::new_legal(board).collect::<Vec<ChessMove>>();
        moves.sort_by(|a, b| self.compare_moves(board, a, b));
        moves
    }

    #[inline]
    pub fn sorted_captures(&self, board: &Board) -> Vec<ChessMove> {
        let mut captures = self.captures(board).collect::<Vec<ChessMove>>();
        captures.sort_by(|a, b| self.compare_moves(board, a, b));
        captures
    }

    #[inline]
    fn is_capture(&self, board: &Board, chess_move: &ChessMove) -> bool {
        let square = BitBoard::from_square(chess_move.get_dest());
        (square & board.color_combined(!board.side_to_move())) != chess::EMPTY
    }

    fn is_promotion(&self, chess_move: &ChessMove) -> bool {
        matches!(chess_move.get_promotion(), Some(_))
    }

    #[inline]
    fn captures(&self, board: &Board) -> MoveGen {
        let mut moves = MoveGen::new_legal(board);
        moves.set_iterator_mask(*board.color_combined(!board.side_to_move()));
        moves
    }

    #[inline]
    fn capture_score(&self, board: &Board, a: &ChessMove) -> i8 {
        let values = [1, 3, 3, 5, 9, 0];
        if self.is_capture(board, a) {
            values[board.piece_on(a.get_source()).unwrap() as usize]
                - values[board.piece_on(a.get_dest()).unwrap() as usize]
        } else if self.is_promotion(a) {
            10
        } else {
            i8::MAX
        }
    }

    #[inline]
    fn compare_moves(&self, board: &Board, a: &ChessMove, b: &ChessMove) -> Ordering {
        self.capture_score(board, a)
            .cmp(&self.capture_score(board, b))
    }
}
