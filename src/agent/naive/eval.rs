use shakmaty::*;
use std::cmp::Ordering;
use std::ops::Neg;

use crate::game::Game;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evaluation {
    Heuristic(i16),
    Win(u8),
    Lose(u8),
    Draw(u8),
}

impl Eq for Evaluation {}

impl Neg for Evaluation {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        match self {
            Self::Heuristic(a) => Self::Heuristic(-a),
            Self::Win(a) => Self::Lose(a),
            Self::Lose(a) => Self::Win(a),
            Self::Draw(a) => Self::Draw(a),
        }
    }
}

impl Ord for Evaluation {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Heuristic(a) => match other {
                Self::Heuristic(b) => a.partial_cmp(&b).unwrap(),
                Self::Lose(_) => Ordering::Greater,
                Self::Win(_) => Ordering::Less,
                Self::Draw(_) => match a.partial_cmp(&0).unwrap() {
                    Ordering::Equal => Ordering::Greater,
                    ordering => ordering,
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
                Self::Heuristic(a) => match a.partial_cmp(&0).unwrap() {
                    Ordering::Equal => Ordering::Less,
                    ordering => ordering.reverse(),
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
    const _ZERO: Self = Self::Heuristic(0);
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
    fn piece_value(piece: &Piece) -> i16 {
        Self::PIECE_VALUES[piece.role as usize]
    }

    #[inline]
    fn position_value(square: &Square, piece: &Piece) -> i16 {
        Self::PIECE_FACTORS[piece.role as usize] * Self::SQUARE_VALUES[*square as usize]
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
            let mut evaluation = 0;
            for (square, piece) in game.position.board().pieces() {
                let color = piece.color as usize;
                evaluation += Self::COLOR_FACTORS[color] * Self::piece_value(&piece);
                evaluation += Self::COLOR_FACTORS[color] * Self::position_value(&square, &piece);
            }
            Self::Heuristic(Self::COLOR_FACTORS[color] * evaluation)
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
        assert_eq!(evaluation, Evaluation::Heuristic(0));
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
        assert_eq!(evaluation, Evaluation::Heuristic(-2));
    }
    #[test_case( Evaluation::Win(8)           , Evaluation::Lose(8)            ; "win"       )]
    #[test_case( Evaluation::Lose(8)          , Evaluation::Win(8)             ; "lose"      )]
    #[test_case( Evaluation::Draw(0)          , Evaluation::Draw(0)            ; "draw"      )]
    #[test_case( Evaluation::_ZERO            , Evaluation::_ZERO              ; "zero"      )]
    #[test_case( Evaluation::Heuristic(32)    , Evaluation::Heuristic(-32)     ; "heuristic" )]
    fn neg(evaluation: Evaluation, expected: Evaluation) {
        assert_eq!(-evaluation, expected);
    }

    #[test_case( Evaluation::Win(5)         , Evaluation::Win(5)            , Ordering::Equal  ; "win eq"            )]
    #[test_case( Evaluation::Win(4)         , Evaluation::Win(5)            , Ordering::Greater; "win gt"            )]
    #[test_case( Evaluation::Draw(5)        , Evaluation::Draw(5)           , Ordering::Equal  ; "draw eq"           )]
    #[test_case( Evaluation::Draw(5)        , Evaluation::Draw(4)           , Ordering::Greater; "draw gt"           )]
    #[test_case( Evaluation::Lose(5)        , Evaluation::Lose(5)           , Ordering::Equal  ; "lose eq"           )]
    #[test_case( Evaluation::Lose(5)        , Evaluation::Lose(4)           , Ordering::Greater; "lose gt"           )]
    #[test_case( Evaluation::Heuristic(45)  , Evaluation::Heuristic(45)     , Ordering::Equal  ; "heuristic eq"      )]
    #[test_case( Evaluation::_ZERO          , Evaluation::_ZERO             , Ordering::Equal  ; "zero eq"           )]
    #[test_case( Evaluation::Win(2)         , Evaluation::Lose(2)           , Ordering::Greater; "win gt lose"       )]
    #[test_case( Evaluation::Win(2)         , Evaluation::Draw(2)           , Ordering::Greater; "win gt draw"       )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::Draw(2)           , Ordering::Less   ; "lose lt draw"      )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::Win(2)            , Ordering::Less   ; "lose lt win"       )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::_ZERO             , Ordering::Less   ; "lose lt zero"      )]
    #[test_case( Evaluation::Lose(2)        , Evaluation::Heuristic(-88)    , Ordering::Less   ; "lose lt heuristic" )]
    #[test_case( Evaluation::Draw(2)        , Evaluation::Lose(2)           , Ordering::Greater; "draw gt lose"      )]
    #[test_case( Evaluation::Draw(2)        , Evaluation::Win(2)            , Ordering::Less   ; "draw lt win"       )]
    #[test_case( Evaluation::Draw(2)        , Evaluation::_ZERO             , Ordering::Less   ; "draw lt zero"      )]
    #[test_case( Evaluation::_ZERO          , Evaluation::Draw(2)           , Ordering::Greater; "zero gt draw"      )]
    #[test_case( Evaluation::Heuristic(1)   , Evaluation::Draw(2)           , Ordering::Greater; "1 0 gt draw"       )]
    #[test_case( Evaluation::Heuristic(1)   , Evaluation::Heuristic(0)      , Ordering::Greater; "1 0 gt 0 0"        )]
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
            Evaluation::Heuristic(1),
            Evaluation::Heuristic(-4),
            Evaluation::Heuristic(0),
            Evaluation::Lose(0),
            Evaluation::Heuristic(10),
            Evaluation::Win(1),
            Evaluation::Heuristic(13),
            Evaluation::Draw(1),
            Evaluation::Heuristic(-30),
            Evaluation::Heuristic(-40),
            Evaluation::Heuristic(-31),
            Evaluation::Lose(4),
        ];
        evals.sort();
        assert_eq!(
            evals,
            vec![
                Evaluation::Lose(0),
                Evaluation::Lose(4),
                Evaluation::Heuristic(-40),
                Evaluation::Heuristic(-31),
                Evaluation::Heuristic(-30),
                Evaluation::Heuristic(-4),
                Evaluation::Draw(0),
                Evaluation::Draw(1),
                Evaluation::_ZERO,
                Evaluation::_ZERO,
                Evaluation::_ZERO,
                Evaluation::Heuristic(1),
                Evaluation::Heuristic(10),
                Evaluation::Heuristic(13),
                Evaluation::Win(3),
                Evaluation::Win(1),
            ]
        );
    }
}
