use super::ChessAgent;
use shakmaty::{Color, Move, Position};
use std::cmp;

use crate::game::Game;

mod eval;

use eval::Evaluation;

pub struct NaiveChessAgent {
    pub color: Color,
    pub depth: usize,
}

fn alpha_beta(
    game: Game,
    depth: usize,
    mut alpha: Evaluation,
    beta: Evaluation,
) -> (Evaluation, Option<Move>) {
    if depth == 0 || game.position.is_game_over() {
        (Evaluation::evaluate(&game), None)
    } else {
        let mut value = Evaluation::MIN;
        let moves = game.position.legal_moves();
        let mut selected_move = moves[0].clone();
        for chess_move in moves.iter() {
            let child = game.play(&chess_move);

            let child_value = -alpha_beta(child, depth - 1, -beta.decrement(), -alpha.decrement())
                .0
                .increment();
            if child_value > value {
                value = child_value;
                selected_move = chess_move.clone();
            }
            alpha = cmp::max(alpha, value);
            if alpha >= beta {
                break;
            }
        }
        (value, Some(selected_move))
    }
}

impl ChessAgent for NaiveChessAgent {
    fn take_turn(&mut self, game: Game) -> Game {
        super::check_side_to_move(self.color, &game);
        if let (_, Some(chess_move)) =
            alpha_beta(game.clone(), self.depth, Evaluation::MIN, Evaluation::MAX)
        {
            game.play(&chess_move)
        } else {
            game
        }
    }
}
