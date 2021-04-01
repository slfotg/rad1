use super::ChessAgent;
use shakmaty::{Chess, Color, Position, Role, Setup};

pub struct NaiveChessAgent {
    pub color: Color,
    pub depth: usize,
}

fn max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

fn min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

fn heuristic(game: &Chess) -> f64 {
    if game.is_game_over() {
        match game.outcome().unwrap().winner() {
            Option::None => 0.0,
            Option::Some(Color::White) => f64::MAX,
            Option::Some(Color::Black) => f64::MIN,
        }
    } else {
        let mut value = 0.0;
        for (_, piece) in game.board().pieces() {
            let piece_value = match piece.role {
                Role::Pawn => 1.0,
                Role::Knight => 3.0,
                Role::Bishop => 3.0,
                Role::Rook => 5.0,
                Role::Queen => 9.0,
                Role::King => 0.0,
            };
            if piece.color == Color::White {
                value += piece_value;
            } else {
                value -= piece_value;
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

fn max_alpha_beta(game: Chess, depth: usize, mut alpha: f64, beta: f64) -> f64 {
    if depth == 0 || game.is_game_over() {
        heuristic(&game)
    } else {
        let mut value = f64::MIN;
        let children = children(game);

        for child in children.iter() {
            value = max(value, min_alpha_beta(child.clone(), depth - 1, alpha, beta));
            alpha = max(alpha, value);
            if alpha >= beta {
                break;
            }
        }
        value
    }
}

fn min_alpha_beta(game: Chess, depth: usize, alpha: f64, mut beta: f64) -> f64 {
    if depth == 0 || game.is_game_over() {
        heuristic(&game)
    } else {
        let mut value = f64::MAX;
        let children = children(game);

        for child in children.iter() {
            value = min(value, max_alpha_beta(child.clone(), depth - 1, alpha, beta));
            beta = min(beta, value);
            if beta <= alpha {
                break;
            }
        }
        value
    }
}

impl ChessAgent for NaiveChessAgent {
    fn take_turn(&mut self, mut position: Chess) -> Chess {
        super::check_side_to_move(&self.color, &position);
        let moves = position.legal_moves();
        let mut value;
        let mut alpha = f64::MIN;
        let mut beta = f64::MAX;
        let mut selected_move = moves[0].clone();
        if Color::White == position.turn() {
            value = f64::MIN;
            for chess_move in moves.iter() {
                let mut child = position.clone();
                child.play_unchecked(chess_move);

                let child_value = min_alpha_beta(child, self.depth - 1, alpha, beta);
                if child_value > value {
                    value = child_value;
                    selected_move = chess_move.clone();
                }
                alpha = max(alpha, value);
            }
        } else {
            value = f64::MAX;
            for chess_move in moves.iter() {
                let mut child = position.clone();
                child.play_unchecked(chess_move);
                let child_value = max_alpha_beta(child, self.depth - 1, alpha, beta);
                if child_value < value {
                    value = child_value;
                    selected_move = chess_move.clone();
                }
                beta = min(beta, value);
                if beta <= alpha {
                    break;
                }
            }
        }
        position.play_unchecked(&selected_move);
        position
    }
}
