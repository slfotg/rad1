use chess::{Board, BoardStatus, Piece};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Evaluation {}

impl Eq for Evaluation {}

impl Evaluation {
    pub const MIN: i16 = i16::MIN + 1; // -32767
    pub const MAX: i16 = i16::MAX; //  32767
    pub const ZERO: i16 = 0;
    const PIECE_VALUES: [i16; 6] = [10, 30, 30, 50, 90, 0];
    #[rustfmt::skip]
    const SQUARE_VALUES: [i16; 64] = [
        1, 1, 1, 1, 1, 1, 1, 1,
        1, 2, 2, 2, 2, 2, 2, 1,
        1, 2, 3, 3, 3, 3, 2, 1,
        1, 2, 3, 4, 4, 3, 2, 1,
        1, 2, 3, 4, 4, 3, 2, 1,
        1, 2, 3, 3, 3, 3, 2, 1,
        1, 2, 2, 2, 2, 2, 2, 1,
        1, 1, 1, 1, 1, 1, 1, 1,
    ];

    #[inline]
    fn piece_value(piece: Piece) -> i16 {
        Self::PIECE_VALUES[piece.to_index()]
    }

    #[inline]
    pub fn evaluate(board: &Board) -> i16 {
        match board.status() {
            BoardStatus::Stalemate => Self::ZERO,
            BoardStatus::Checkmate => Self::MIN,
            BoardStatus::Ongoing => {
                let mut evaluation = 0;
                let my_pieces = board.color_combined(board.side_to_move());
                let their_pieces = board.color_combined(!board.side_to_move());

                // Piece Values
                for &piece in chess::ALL_PIECES.iter() {
                    let pieces = board.pieces(piece);
                    let value = Self::piece_value(piece);
                    evaluation += value
                        * ((my_pieces & pieces).popcnt() as i16
                            - (their_pieces & pieces).popcnt() as i16);
                }

                // Position Values
                for square in *my_pieces {
                    evaluation += Self::SQUARE_VALUES[square.to_index()];
                }
                for square in *their_pieces {
                    evaluation -= Self::SQUARE_VALUES[square.to_index()];
                }
                evaluation
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluation;
    use chess::{Board, ChessMove, Square};

    #[test]
    fn initial_board_eval() {
        let board = Board::default();
        let evaluation = Evaluation::evaluate(&board);
        assert_eq!(evaluation, 0);
    }

    #[test]
    fn e4_black_turn_eval() {
        let board = Board::default();
        let chess_move = ChessMove::new(Square::E2, Square::E4, None);
        let board = board.make_move_new(chess_move);
        let evaluation = Evaluation::evaluate(&board);
        assert_eq!(evaluation, -2);
    }
}
