use crate::game::Game;
use chess::{BoardStatus, Piece};

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
    pub fn evaluate(game: &Game) -> i16 {
        let board = game.get_board();
        match board.status() {
            BoardStatus::Stalemate => Self::ZERO,
            BoardStatus::Checkmate => Self::MIN,
            BoardStatus::Ongoing => {
                let mut evaluation = 0;
                let my_pieces = board.color_combined(game.turn());
                let their_pieces = board.color_combined(!game.turn());

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
    use crate::game::Game;
    use chess::{ChessMove, Square};

    #[test]
    fn initial_board_eval() {
        let game = Game::default();
        let evaluation = Evaluation::evaluate(&game);
        assert_eq!(evaluation, 0);
    }

    #[test]
    fn e4_black_turn_eval() {
        let mut game = Game::default();
        let chess_move = ChessMove::new(Square::E2, Square::E4, None);
        game.play_mut(chess_move);
        let evaluation = Evaluation::evaluate(&game);
        assert_eq!(evaluation, -2);
    }
}
