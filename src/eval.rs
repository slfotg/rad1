use shakmaty::*;

use crate::game::Game;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Evaluation {}

impl Eq for Evaluation {}

impl Evaluation {
    pub const MIN: i16 = i16::MIN + 1; // -32767
    pub const MAX: i16 = i16::MAX; //  32767
    pub const ZERO: i16 = 0;
    const PIECE_VALUES: [i16; 7] = [0, 10, 30, 30, 50, 90, 0];
    const PIECE_FACTORS: [i16; 7] = [0, 1, 1, 1, 1, 1, 0];
    const COLOR_FACTORS: [i16; 2] = [-1, 1];
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
    fn piece_value(piece: &Piece) -> i16 {
        Self::PIECE_VALUES[piece.role as usize]
    }

    #[inline]
    fn position_value(square: &Square, piece: &Piece) -> i16 {
        Self::PIECE_FACTORS[piece.role as usize] * Self::SQUARE_VALUES[*square as usize]
    }

    #[inline]
    pub fn evaluate(game: &Game) -> i16 {
        let color = game.position.turn() as usize;
        if game.position.is_game_over() {
            match game.position.outcome().unwrap().winner() {
                Option::None => Self::ZERO,
                Option::Some(color) => {
                    if color == game.position.turn() {
                        Self::MAX
                    } else {
                        Self::MIN
                    }
                }
            }
        } else {
            let mut evaluation = 0;
            for (square, piece) in game.position.board().pieces() {
                let color = piece.color as usize;
                evaluation += Self::COLOR_FACTORS[color] * Self::piece_value(&piece);
                evaluation += Self::COLOR_FACTORS[color] * Self::position_value(&square, &piece);
            }
            Self::COLOR_FACTORS[color] * evaluation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluation;
    use crate::game::Game;
    use shakmaty::*;

    #[test]
    fn initial_board_eval() {
        let game = Game::default();
        let evaluation = Evaluation::evaluate(&game);
        assert_eq!(evaluation, 0);
    }

    #[test]
    fn e4_black_turn_eval() {
        let mut game = Game::default();
        let m: Move = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        };
        game.play_mut(&m);
        let evaluation = Evaluation::evaluate(&game);
        assert_eq!(evaluation, -2);
    }
}
