use super::ChessAgent;
use crate::eval::Evaluation;
use crate::move_sorter::MOVE_SORTER;
use crate::tt::*;
use chess::{Action, Board, BoardStatus, ChessMove, Game};
use std::cmp;
use std::sync::Arc;

// quiescence search
fn q_search(board: &Board, mut alpha: i16, beta: i16) -> i16 {
    let evaluation = Evaluation::evaluate(board);
    if evaluation >= beta {
        beta
    } else {
        if alpha < evaluation {
            alpha = evaluation;
        }
        for m in MOVE_SORTER.sorted_captures(board).into_iter() {
            let score = -q_search(&board.make_move_new(m), -beta, -alpha);
            if score >= beta {
                alpha = beta;
                break;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
}

// this is really just a pure alpha beta search
// with no caching or storing evaluations in nodes
// used for the null move heursitic
fn null_alpha_beta(board: &Board, depth: u8, mut alpha: i16, beta: i16) -> i16 {
    if depth == 0 {
        Evaluation::evaluate(board)
    } else {
        for child_move in MOVE_SORTER.sorted_moves(board) {
            let val = -null_alpha_beta(&board.make_move_new(child_move), depth - 1, -beta, -alpha);
            if val >= beta {
                return beta;
            }
            if val > alpha {
                alpha = val;
            }
        }
        alpha
    }
}

fn cached_evaluation(
    trans_table: &TranspositionTable,
    board: &Board,
    depth: u8,
    value: &mut i16,
    alpha: &mut i16,
    beta: &mut i16,
) -> Option<i16> {
    match trans_table.get_evaluation(board) {
        None => None,
        Some(cached_eval) => {
            if cached_eval.depth() >= depth {
                *value = cached_eval.evaluation();
                match cached_eval.node_type() {
                    NodeType::PvNode => Some(cached_eval.evaluation()),
                    NodeType::AllNode => {
                        *alpha = cmp::max(*alpha, cached_eval.evaluation());
                        None
                    }
                    NodeType::CutNode => {
                        *beta = cmp::min(*beta, cached_eval.evaluation());
                        None
                    }
                }
            } else {
                None
            }
        }
    }
}

fn expand(board: &Board) -> Vec<ChessMove> {
    MOVE_SORTER.sorted_moves(board)
}

fn alpha_beta(
    trans_table: &TranspositionTable,
    board: &Board,
    mut depth: u8,
    mut alpha: i16,
    mut beta: i16,
) -> i16 {
    if board.checkers().popcnt() > 0 && depth > 0 {
        depth += 1;
    }
    let status = board.status();
    let alpha_orig = alpha;
    let mut value = Evaluation::MIN;

    let cached_evaluation =
        cached_evaluation(trans_table, board, depth, &mut value, &mut alpha, &mut beta);

    if let Some(evaluation) = cached_evaluation {
        evaluation
    } else if status != BoardStatus::Ongoing {
        Evaluation::evaluate(board)
    } else if depth == 0 {
        let value = q_search(board, alpha, beta);
        trans_table.update_evaluation(board, CachedValue::new(depth, value, NodeType::PvNode));
        value
    } else {
        if depth >= 3 {
            if let Some(null_move_game) = board.null_move() {
                let score = -null_alpha_beta(&null_move_game, depth - 3, -beta, -beta + 1);
                if score >= beta {
                    return beta;
                }
            }
        }
        let mut best_move = None;
        for child_move in expand(board) {
            let child_value = -alpha_beta(
                trans_table,
                &board.make_move_new(child_move),
                depth - 1,
                -beta,
                -alpha,
            );
            if child_value > value {
                value = child_value;
                best_move = Some(child_move);
            }
            value = cmp::max(child_value, value);
            alpha = cmp::max(alpha, value);
            if alpha >= beta {
                break;
            }
        }
        let cached_eval = if value <= alpha_orig {
            // Beta
            CachedValue::new(depth, value, NodeType::AllNode)
        } else if value >= beta {
            // Alpha
            CachedValue::new(depth, value, NodeType::CutNode)
        } else {
            // Exact
            CachedValue::new(depth, value, NodeType::PvNode)
        };
        trans_table.update_evaluation(board, cached_eval);
        if let Some(best_move) = best_move {
            trans_table.update_best_move(board, depth, best_move);
        }
        value
    }
}

pub struct AlphaBetaChessAgent {
    depth: u8,
    evaluator: Arc<TranspositionTable>,
}

impl AlphaBetaChessAgent {
    pub fn new(depth: u8) -> Self {
        AlphaBetaChessAgent {
            depth,
            evaluator: Arc::default(),
        }
    }
}

impl ChessAgent for AlphaBetaChessAgent {
    fn get_action(&self, game: &Game) -> Action {
        let alpha = Evaluation::MIN;
        let beta = Evaluation::MAX;
        for i in 1..=self.depth {
            alpha_beta(&self.evaluator, &game.current_position(), i, alpha, beta);
            let best_move = self.evaluator.best_move(&game.current_position());
            let evaluation = self
                .evaluator
                .get_shallow_evaluation(&game.current_position());
            println!("{} - {} = {}", i, best_move.unwrap(), evaluation.unwrap());
        }

        // get best move
        let best_move = self.evaluator.best_move(&game.current_position()).unwrap();

        println!("Best move: {}", best_move);
        Action::MakeMove(best_move)
    }
}
