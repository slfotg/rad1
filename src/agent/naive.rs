use super::ChessAgent;
use shakmaty::{Chess, Color, Position, Role, Setup, Square};
use std::cmp;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::game::Game;

pub struct NaiveChessAgent {
    pub color: Color,
    pub depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Value {
    piece_value: i32,
    position_value: i32,
}

impl Value {
    const MAX: Value = Value {
        piece_value: i32::MAX,
        position_value: i32::MAX,
    };
    const MIN: Value = Value {
        piece_value: i32::MIN,
        position_value: i32::MIN,
    };
    const ZERO: Value = Value {
        piece_value: 0,
        position_value: 0,
    };
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            piece_value: self.piece_value + other.piece_value,
            position_value: self.position_value + other.position_value,
        }
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            piece_value: self.piece_value - other.piece_value,
            position_value: self.position_value - other.position_value,
        }
    }
}

impl SubAssign for Value {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

fn heuristic(game: &Chess) -> Value {
    if game.is_game_over() {
        match game.outcome().unwrap().winner() {
            Option::None => Value::ZERO,
            Option::Some(Color::White) => Value::MAX,
            Option::Some(Color::Black) => Value::MIN,
        }
    } else {
        let mut value = Value::ZERO;
        for (square, piece) in game.board().pieces() {
            let piece_value = match piece.role {
                Role::Pawn => 1,
                Role::Knight => 3,
                Role::Bishop => 3,
                Role::Rook => 5,
                Role::Queen => 9,
                Role::King => 0,
            };
            let position_value = match square {
                Square::D5 => 4,
                Square::E5 => 4,
                Square::D4 => 4,
                Square::E4 => 4,
                Square::C6 => 3,
                Square::D6 => 3,
                Square::E6 => 3,
                Square::F6 => 3,
                Square::C5 => 3,
                Square::F5 => 3,
                Square::C4 => 3,
                Square::F4 => 3,
                Square::C3 => 3,
                Square::D3 => 3,
                Square::E3 => 3,
                Square::F3 => 3,
                Square::B7 => 2,
                Square::C7 => 2,
                Square::D7 => 2,
                Square::E7 => 2,
                Square::F7 => 2,
                Square::G7 => 2,
                Square::B6 => 2,
                Square::B5 => 2,
                Square::B4 => 2,
                Square::B3 => 2,
                Square::G6 => 2,
                Square::G5 => 2,
                Square::G4 => 2,
                Square::G3 => 2,
                Square::B2 => 2,
                Square::C2 => 2,
                Square::D2 => 2,
                Square::E2 => 2,
                Square::F2 => 2,
                Square::G2 => 2,
                _ => 1,
            };
            let partial_value = Value {
                piece_value,
                position_value,
            };
            if piece.color == Color::White {
                value += partial_value;
            } else {
                value -= partial_value;
            }
        }
        value
    }
}

fn children(game: Chess) -> Vec<Chess> {
    game.legal_moves()
        .into_iter()
        .map(|m| {
            let mut child = game.clone();
            child.play_unchecked(&m);
            child
        })
        .collect()
}

fn max_alpha_beta(game: Chess, depth: usize, mut alpha: Value, beta: Value) -> Value {
    if depth == 0 || game.is_game_over() {
        heuristic(&game)
    } else {
        let mut value = Value::MIN;
        let mut children = children(game);
        children.sort_by(|a, b| heuristic(&b).partial_cmp(&heuristic(&a)).unwrap());
        for child in children.iter() {
            value = cmp::max(value, min_alpha_beta(child.clone(), depth - 1, alpha, beta));
            alpha = cmp::max(alpha, value);
            if alpha >= beta {
                break;
            }
        }
        value
    }
}

fn min_alpha_beta(game: Chess, depth: usize, alpha: Value, mut beta: Value) -> Value {
    if depth == 0 || game.is_game_over() {
        heuristic(&game)
    } else {
        let mut value = Value::MAX;
        let mut children = children(game);
        children.sort_by(|a, b| heuristic(&a).partial_cmp(&heuristic(&b)).unwrap());
        for child in children.iter() {
            value = cmp::min(value, max_alpha_beta(child.clone(), depth - 1, alpha, beta));
            beta = cmp::min(beta, value);
            if beta <= alpha {
                break;
            }
        }
        value
    }
}

impl ChessAgent for NaiveChessAgent {
    fn take_turn(&mut self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
        let moves = game.position.legal_moves();
        let mut value;
        let mut alpha = Value::MIN;
        let mut beta = Value::MAX;
        let mut selected_move = moves[0].clone();
        if Color::White == game.position.turn() {
            value = Value::MIN;
            for chess_move in moves.iter() {
                let mut child = game.position.clone();
                child.play_unchecked(chess_move);

                let child_value = min_alpha_beta(child, self.depth - 1, alpha, beta);
                if child_value > value {
                    value = child_value;
                    selected_move = chess_move.clone();
                }
                alpha = cmp::max(alpha, value);
            }
        } else {
            value = Value::MAX;
            for chess_move in moves.iter() {
                let mut child = game.position.clone();
                child.play_unchecked(chess_move);
                let child_value = max_alpha_beta(child, self.depth - 1, alpha, beta);
                if child_value < value {
                    value = child_value;
                    selected_move = chess_move.clone();
                }
                beta = cmp::min(beta, value);
                if beta <= alpha {
                    break;
                }
            }
        }
        game.play(&selected_move)
    }
}
