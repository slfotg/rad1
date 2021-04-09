use shakmaty::*;
use std::cmp::Ordering;
use std::ops::Neg;

use crate::game::Game;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evaluation {
    Heuristic(i32, i32),
    Win(i32),
    Lose(i32),
    Draw(i32),
}

impl Neg for Evaluation {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        match self {
            Self::Heuristic(a, b) => Self::Heuristic(-a, -b),
            Self::Win(a) => Self::Lose(a),
            Self::Lose(a) => Self::Win(a),
            Self::Draw(a) => Self::Draw(a),
        }
    }
}

impl Ord for Evaluation {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Heuristic(a, b) => match other {
                Self::Heuristic(c, d) => match a.cmp(c) {
                    Ordering::Equal => b.cmp(d),
                    ordering => ordering,
                },
                Self::Lose(_) => Ordering::Greater,
                Self::Win(_) => Ordering::Less,
                Self::Draw(_) => match (a.cmp(&0), b.cmp(&0)) {
                    (Ordering::Equal, Ordering::Equal) => Ordering::Greater,
                    (Ordering::Equal, ordering) => ordering,
                    (ordering, _) => ordering,
                },
            },
            Self::Win(a) => match other {
                Self::Win(b) => b.cmp(&a),
                _ => Ordering::Greater,
            },
            Self::Lose(a) => match other {
                Self::Lose(b) => a.cmp(&b),
                _ => Ordering::Less,
            },
            Self::Draw(a) => match other {
                Self::Draw(b) => a.cmp(&b),
                Self::Win(_) => Ordering::Less,
                Self::Lose(_) => Ordering::Greater,
                Self::Heuristic(a, b) => match (a.cmp(&0), b.cmp(&0)) {
                    (Ordering::Equal, Ordering::Equal) => Ordering::Less,
                    (Ordering::Equal, ordering) => ordering.reverse(),
                    (ordering, _) => ordering.reverse(),
                },
            },
        }
    }
}

impl PartialOrd for Evaluation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Option::Some(self.cmp(other))
    }
}

impl Evaluation {
    pub const MIN: Self = Self::Lose(0);
    pub const MAX: Self = Self::Win(0);
    const _ZERO: Self = Self::Heuristic(0, 0);
    const PIECE_VALUES: [i32; 7] = [0, 1, 3, 3, 5, 9, 0];
    const PIECE_FACTORS: [i32; 7] = [0, 1, 1, 1, 1, 1, 0];
    const COLOR_FACTORS: [i32; 2] = [-1, 1];
    #[rustfmt::skip]
    const SQUARE_VALUES: [i32; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 1, 2, 2, 2, 2, 1, 0,
        0, 1, 2, 3, 3, 2, 1, 0,
        0, 1, 2, 3, 3, 2, 1, 0,
        0, 1, 2, 2, 2, 2, 1, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    pub fn increment(self) -> Self {
        match self {
            Self::Win(a) => Self::Win(a + 1),
            Self::Lose(a) => Self::Lose(a + 1),
            Self::Draw(a) => Self::Draw(a + 1),
            a => a,
        }
    }

    pub fn decrement(self) -> Self {
        match self {
            Self::Win(a) => {
                if a > 0 {
                    Self::Win(a - 1)
                } else {
                    Self::Win(0)
                }
            }
            Self::Lose(a) => {
                if a > 0 {
                    Self::Lose(a - 1)
                } else {
                    Self::Lose(0)
                }
            }
            Self::Draw(a) => {
                if a > 0 {
                    Self::Draw(a - 1)
                } else {
                    Self::Draw(0)
                }
            }
            a => a,
        }
    }

    #[inline]
    fn piece_value(piece: &Piece) -> i32 {
        Self::PIECE_VALUES[usize::from(piece.role)]
    }

    #[inline]
    fn position_value(square: &Square, piece: &Piece) -> i32 {
        Self::PIECE_FACTORS[usize::from(piece.role)] * Self::SQUARE_VALUES[usize::from(*square)]
    }

    #[inline]
    pub fn evaluate(game: &Game) -> Evaluation {
        let color = game.position.turn() as usize;
        if game.position.is_game_over() {
            match game.position.outcome().unwrap().winner() {
                Option::None => Self::Draw(0),
                Option::Some(color) => {
                    if color == game.position.turn() {
                        Self::Win(0)
                    } else {
                        Self::Lose(0)
                    }
                }
            }
        } else {
            let mut evaluation = (0, 0);
            for (square, piece) in game.position.board().pieces() {
                let color = piece.color as usize;
                evaluation.0 += Self::COLOR_FACTORS[color] * Self::piece_value(&piece);
                evaluation.1 += Self::COLOR_FACTORS[color] * Self::position_value(&square, &piece);
            }
            Self::Heuristic(
                Self::COLOR_FACTORS[color] * evaluation.0,
                Self::COLOR_FACTORS[color] * evaluation.1,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluation;
    use crate::game::Game;
    use shakmaty::*;
    use std::cmp::Ordering;
    use test_case::test_case;

    #[test]
    fn initial_board_eval() {
        let game = Game::default();
        let evaluation = Evaluation::evaluate(&game);
        assert_eq!(evaluation, Evaluation::Heuristic(0, 0));
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
        assert_eq!(evaluation, Evaluation::Heuristic(0, -2));
    }
    #[test_case( Evaluation::Win(8)           , Evaluation::Lose(8)            ; "win"       )]
    #[test_case( Evaluation::Lose(8)          , Evaluation::Win(8)             ; "lose"      )]
    #[test_case( Evaluation::Draw(0)          , Evaluation::Draw(0)            ; "draw"      )]
    #[test_case( Evaluation::_ZERO            , Evaluation::_ZERO              ; "zero"      )]
    #[test_case( Evaluation::Heuristic(32, 14), Evaluation::Heuristic(-32, -14); "heuristic" )]
    fn neg(evaluation: Evaluation, expected: Evaluation) {
        assert_eq!(-evaluation, expected);
    }

    #[test_case( Evaluation::Win(5)         , Evaluation::Win(5)            , Ordering::Equal  ; "win eq"            )]
    #[test_case( Evaluation::Win(4)         , Evaluation::Win(5)            , Ordering::Greater; "win gt"            )]
    #[test_case( Evaluation::Draw(5)        , Evaluation::Draw(5)           , Ordering::Equal  ; "draw eq"           )]
    #[test_case( Evaluation::Draw(5)        , Evaluation::Draw(4)           , Ordering::Greater; "draw gt"           )]
    #[test_case( Evaluation::Lose(5)        , Evaluation::Lose(5)           , Ordering::Equal  ; "lose eq"           )]
    #[test_case( Evaluation::Lose(5)        , Evaluation::Lose(4)           , Ordering::Greater; "lose gt"           )]
    #[test_case( Evaluation::Heuristic(4, 5), Evaluation::Heuristic(4, 5)   , Ordering::Equal  ; "heuristic eq"      )]
    #[test_case( Evaluation::_ZERO          , Evaluation::_ZERO             , Ordering::Equal  ; "zero eq"           )]
    #[test_case( Evaluation::Win(2)         , Evaluation::Lose(2)           , Ordering::Greater; "win gt lose"       )]
    #[test_case( Evaluation::Win(2)         , Evaluation::Draw(2)           , Ordering::Greater; "win gt draw"       )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::Draw(2)           , Ordering::Less   ; "lose lt draw"      )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::Win(2)            , Ordering::Less   ; "lose lt win"       )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::_ZERO             , Ordering::Less   ; "lose lt zero"      )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::Heuristic(-88, 90), Ordering::Less   ; "lose lt heuristic" )]
    #[test_case( Evaluation::Draw(2)        , Evaluation::Lose(2)           , Ordering::Greater; "draw gt lose"      )]
    #[test_case( Evaluation::Draw(2)        , Evaluation::Win(2)            , Ordering::Less   ; "draw lt win"       )]
    #[test_case( Evaluation::Draw(2)        , Evaluation::_ZERO             , Ordering::Less   ; "draw lt zero"      )]
    #[test_case( Evaluation::_ZERO          , Evaluation::Draw(2)           , Ordering::Greater; "zero gt draw"      )]
    #[test_case( Evaluation::Heuristic(1, 0), Evaluation::Draw(2)           , Ordering::Greater; "1 0 gt draw"       )]
    #[test_case( Evaluation::Heuristic(1, 0), Evaluation::Heuristic(0, 0)   , Ordering::Greater; "1 0 gt 0 0"        )]
    #[test_case( Evaluation::Heuristic(0, 1), Evaluation::Heuristic(0, 0)   , Ordering::Greater; "0 1 gt 0 0"        )]
    fn cmp(left: Evaluation, right: Evaluation, expected: Ordering) {
        assert_eq!(left.cmp(&right), expected);
        assert_eq!(right.cmp(&left), expected.reverse());
    }

    #[test]
    fn ordering() {
        let mut evals = vec![
            Evaluation::_ZERO,
            Evaluation::Win(3),
            Evaluation::Draw(0),
            Evaluation::_ZERO,
            Evaluation::Heuristic(0, 1),
            Evaluation::Heuristic(0, -4),
            Evaluation::Heuristic(0, 0),
            Evaluation::Lose(0),
            Evaluation::Heuristic(1, 0),
            Evaluation::Win(1),
            Evaluation::Heuristic(1, 3),
            Evaluation::Draw(1),
            Evaluation::Heuristic(-3, 0),
            Evaluation::Heuristic(-4, 0),
            Evaluation::Heuristic(-3, 1),
            Evaluation::Lose(4),
        ];
        evals.sort();
        assert_eq!(
            evals,
            vec![
                Evaluation::Lose(0),
                Evaluation::Lose(4),
                Evaluation::Heuristic(-4, 0),
                Evaluation::Heuristic(-3, 0),
                Evaluation::Heuristic(-3, 1),
                Evaluation::Heuristic(0, -4),
                Evaluation::Draw(0),
                Evaluation::Draw(1),
                Evaluation::_ZERO,
                Evaluation::_ZERO,
                Evaluation::_ZERO,
                Evaluation::Heuristic(0, 1),
                Evaluation::Heuristic(1, 0),
                Evaluation::Heuristic(1, 3),
                Evaluation::Win(3),
                Evaluation::Win(1),
            ]
        );
    }
}
