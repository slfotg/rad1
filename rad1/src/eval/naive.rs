use super::Evaluator;
use chess::{Board, BoardStatus, Color, Piece, Square};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NaiveEvaluator;

impl NaiveEvaluator {
    const MIN: i16 = -30000;
    const MAX: i16 = 30000;
    const ZERO: i16 = 0;
    const PIECE_VALUES: [i16; 6] = [10, 30, 30, 50, 90, 0];
    #[rustfmt::skip]
    const _SQUARE_VALUES: [i16; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 1, 2, 2, 2, 2, 1, 0,
        0, 1, 2, 3, 3, 2, 1, 0,
        0, 1, 2, 3, 3, 2, 1, 0,
        0, 1, 2, 2, 2, 2, 1, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[rustfmt::skip]
    const WHITE_PAWN_VALUES: [i16; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 2, 2, 1, 0, 0,
        1, 1, 2, 3, 3, 2, 1, 1,
        2, 2, 3, 3, 3, 3, 2, 2,
        3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[rustfmt::skip]
    const BLACK_PAWN_VALUES: [i16; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0,
        4, 4, 4, 4, 4, 4, 4, 4,
        3, 3, 3, 3, 3, 3, 3, 3,
        2, 2, 3, 3, 3, 3, 2, 2,
        1, 1, 2, 3, 3, 2, 1, 1,
        0, 0, 1, 2, 2, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[rustfmt::skip]
    const KNIGHT_VALUES: [i16; 64] = [
        0, 1, 2, 2, 2, 2, 1, 0,
        1, 2, 3, 4, 4, 3, 2, 1,
        2, 3, 5, 5, 5, 5, 3, 2,
        2, 4, 5, 5, 5, 5, 4, 2,
        2, 4, 5, 5, 5, 5, 4, 2,
        2, 3, 5, 5, 5, 5, 3, 2,
        1, 2, 3, 4, 4, 3, 2, 1,
        0, 1, 2, 2, 2, 2, 1, 0,
    ];

    #[rustfmt::skip]
    const BISHOP_VALUES: [i16; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 3, 2, 2, 2, 2, 3, 0,
        0, 2, 3, 3, 3, 3, 2, 0,
        0, 2, 3, 4, 4, 3, 2, 0,
        0, 2, 3, 4, 4, 3, 2, 0,
        0, 2, 3, 3, 3, 3, 2, 0,
        0, 3, 2, 2, 2, 2, 3, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[inline]
    fn piece_value(piece: Piece) -> i16 {
        Self::PIECE_VALUES[piece.to_index()]
    }

    #[inline]
    fn position_value(piece: Piece, color: Color, square: Square) -> i16 {
        match piece {
            Piece::Pawn => match color {
                Color::Black => Self::BLACK_PAWN_VALUES[square.to_index()],
                Color::White => Self::WHITE_PAWN_VALUES[square.to_index()],
            },
            Piece::Knight => Self::KNIGHT_VALUES[square.to_index()],
            Piece::Bishop => Self::BISHOP_VALUES[square.to_index()],
            Piece::Queen => Self::BISHOP_VALUES[square.to_index()],
            _ => 0,
        }
    }
}

impl Evaluator for NaiveEvaluator {
    type Result = i16;
    #[inline]
    fn min_value(&self) -> Self::Result {
        Self::MIN
    }

    #[inline]
    fn max_value(&self) -> Self::Result {
        Self::MAX
    }

    #[inline]
    fn evaluate(&self, board: &Board) -> Self::Result {
        match board.status() {
            BoardStatus::Stalemate => Self::ZERO,
            BoardStatus::Checkmate => Self::MIN,
            BoardStatus::Ongoing => {
                let mut evaluation = 0;
                let my_color = board.side_to_move();
                let my_pieces = board.color_combined(my_color);
                let their_pieces = board.color_combined(!my_color);
                let pawns = board.pieces(Piece::Pawn);
                let knights = board.pieces(Piece::Knight);
                let bishops = board.pieces(Piece::Bishop);
                let queens = board.pieces(Piece::Queen);

                // Piece Values
                for &piece in chess::ALL_PIECES.iter() {
                    let pieces = board.pieces(piece);
                    let value = Self::piece_value(piece);
                    evaluation += value
                        * ((my_pieces & pieces).popcnt() as i16
                            - (their_pieces & pieces).popcnt() as i16);
                }

                // Position Values
                // Pawns:
                for square in *pawns & *my_pieces {
                    evaluation += Self::position_value(Piece::Pawn, my_color, square);
                }
                for square in *pawns & *their_pieces {
                    evaluation -= Self::position_value(Piece::Pawn, !my_color, square);
                }
                // Knights:
                for square in *knights & *my_pieces {
                    evaluation += Self::position_value(Piece::Knight, my_color, square);
                }
                for square in *knights & *their_pieces {
                    evaluation -= Self::position_value(Piece::Knight, !my_color, square);
                }
                // Bishops:
                for square in *bishops & *my_pieces {
                    evaluation += Self::position_value(Piece::Bishop, my_color, square);
                }
                for square in *bishops & *their_pieces {
                    evaluation -= Self::position_value(Piece::Bishop, !my_color, square);
                }
                // Queens:
                for square in *queens & *my_pieces {
                    evaluation += Self::position_value(Piece::Queen, my_color, square);
                }
                for square in *queens & *their_pieces {
                    evaluation -= Self::position_value(Piece::Queen, !my_color, square);
                }
                evaluation
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NaiveEvaluator;
    use crate::eval::Evaluator;
    use chess::{Board, ChessMove, Square};

    #[test]
    fn initial_board_eval() {
        let board = Board::default();
        let evaluator = NaiveEvaluator;
        let evaluation = evaluator.evaluate(&board);
        assert_eq!(evaluation, 0);
    }

    #[test]
    fn e4_black_turn_eval() {
        let board = Board::default();
        let chess_move = ChessMove::new(Square::E2, Square::E4, None);
        let board = board.make_move_new(chess_move);
        let evaluator = NaiveEvaluator;
        let evaluation = evaluator.evaluate(&board);
        assert_eq!(evaluation, -3);
    }
}
