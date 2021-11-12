use super::ChessAgent;
use crate::eval::Evaluator;
use crate::move_sorter::MOVE_SORTER;
use crate::node::NodeValue;
use crate::tt::*;
use chess::{Action, Board, BoardStatus, ChessMove, Game};
use std::cmp;
use std::sync::Arc;

pub struct AlphaBetaChessAgent {
    depth: u8,
    tt: Arc<TranspositionTable<i16>>,
    evaluator: Arc<dyn Evaluator<Result = i16>>,
}

impl AlphaBetaChessAgent {
    pub fn new(
        depth: u8,
        tt: TranspositionTable<i16>,
        evaluator: Arc<dyn Evaluator<Result = i16>>,
    ) -> Self {
        AlphaBetaChessAgent {
            depth,
            tt: Arc::new(tt),
            evaluator,
        }
    }

    pub fn set_evaluator(&mut self, evaluator: Arc<dyn Evaluator<Result = i16>>) {
        self.evaluator = evaluator;
    }

    fn cached_evaluation(
        tt: &TranspositionTable<i16>,
        board: &Board,
        depth: u8,
        alpha: &mut i16,
        beta: &mut i16,
    ) -> Option<i16> {
        match tt.get_evaluation_and_depth(board) {
            None => None,
            Some((cached_eval, evaluation_depth)) => {
                if evaluation_depth >= depth {
                    match cached_eval {
                        NodeValue::Principal { value } => Some(value),
                        NodeValue::All { value } => {
                            *alpha = cmp::max(*alpha, value);
                            None
                        }
                        NodeValue::Cut { value } => {
                            *beta = cmp::min(*beta, value);
                            None
                        }
                    }
                } else {
                    None
                }
            }
        }
    }

    fn update_cache(
        tt: &TranspositionTable<i16>,
        board: &Board,
        depth: u8,
        alpha: i16,
        beta: i16,
        value: i16,
        best_move: ChessMove,
    ) {
        let board = *board;
        let node = if value <= alpha {
            // Beta
            NodeValue::all_node(value)
        } else if value >= beta {
            // Alpha
            NodeValue::cut_node(value)
        } else {
            // Exact
            NodeValue::pv_node(value)
        };
        tt.update_evaluation_and_best_move(&board, depth, node, Some(best_move));
    }

    fn check_extension(board: &Board, depth: &mut u8, check_extension_enabled: &mut bool) {
        if *check_extension_enabled && board.checkers().popcnt() > 0 {
            *depth += 1;
            // only allow one check extension in a search path
            *check_extension_enabled = false;
        }
    }

    fn expand(tt: &TranspositionTable<i16>, board: &Board) -> Vec<ChessMove> {
        MOVE_SORTER.sorted_moves(board, tt.best_move(board))
    }

    // quiescence search
    fn q_search(
        evaluator: &dyn Evaluator<Result = i16>,
        board: &Board,
        mut alpha: i16,
        beta: i16,
    ) -> i16 {
        let evaluation = evaluator.evaluate(board);
        if evaluation >= beta {
            beta
        } else {
            if alpha < evaluation {
                alpha = evaluation;
            }
            for m in MOVE_SORTER.sorted_captures(board).into_iter() {
                let score = -Self::q_search(evaluator, &board.make_move_new(m), -beta, -alpha);
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
    fn null_alpha_beta(
        evaluator: &dyn Evaluator<Result = i16>,
        board: &Board,
        depth: u8,
        mut alpha: i16,
        beta: i16,
    ) -> i16 {
        if depth == 0 {
            evaluator.evaluate(board)
        } else {
            for child_move in MOVE_SORTER.sorted_moves(board, None) {
                let val = -Self::null_alpha_beta(
                    evaluator,
                    &board.make_move_new(child_move),
                    depth - 1,
                    -beta,
                    -alpha,
                );
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

    fn null_window_search(
        evaluator: &dyn Evaluator<Result = i16>,
        tt: &TranspositionTable<i16>,
        board: &Board,
        depth: u8,
        alpha: i16,
        beta: i16,
        check_extension_enabled: bool,
    ) -> i16 {
        // Search with null window at first
        let value = -Self::alpha_beta(
            evaluator,
            tt,
            board,
            depth - 1,
            -alpha - 1,
            -alpha,
            check_extension_enabled,
        );
        // Re-search the path with regular window if alpha < value < beta
        if alpha < value && value < beta {
            -Self::alpha_beta(
                evaluator,
                tt,
                board,
                depth - 1,
                -beta,
                -alpha,
                check_extension_enabled,
            )
        } else {
            value
        }
    }

    fn principal_variation_search(
        evaluator: &dyn Evaluator<Result = i16>,
        tt: &TranspositionTable<i16>,
        board: &Board,
        depth: u8,
        mut alpha: i16,
        beta: i16,
        check_extension_enabled: bool,
    ) -> (i16, ChessMove) {
        let moves = Self::expand(tt, board);
        let mut best_move = moves[0];

        // Search down the principal variation path first with regular window
        let value = -Self::alpha_beta(
            evaluator,
            tt,
            &board.make_move_new(moves[0]),
            depth - 1,
            -beta,
            -alpha,
            check_extension_enabled,
        );
        if value > alpha {
            alpha = value;
        }
        if alpha >= beta {
            return (alpha, best_move);
        }

        // Search the rest of the paths with null windows
        for &child_move in moves.iter().skip(1) {
            let value = Self::null_window_search(
                evaluator,
                tt,
                &board.make_move_new(child_move),
                depth,
                alpha,
                beta,
                check_extension_enabled,
            );
            if value > alpha {
                alpha = value;
                best_move = child_move;
            }
            if alpha >= beta {
                break;
            }
        }
        (alpha, best_move)
    }

    fn alpha_beta(
        evaluator: &dyn Evaluator<Result = i16>,
        tt: &TranspositionTable<i16>,
        board: &Board,
        mut depth: u8,
        mut alpha: i16,
        mut beta: i16,
        mut check_extension_enabled: bool,
    ) -> i16 {
        Self::check_extension(board, &mut depth, &mut check_extension_enabled);
        let status = board.status();
        let alpha_orig = alpha;
        // Get cached evaluation if it exists and update alpha/beta accordingly
        // If an exact value is already cached, return that immediately
        if let Some(value) = Self::cached_evaluation(tt, board, depth, &mut alpha, &mut beta) {
            return value;
        }
        // If game is over, return evaluation
        if status != BoardStatus::Ongoing {
            return evaluator.evaluate(board);
        }
        // If depth is 0, evaluate after quiesence search, cache and return
        if depth == 0 {
            let value = Self::q_search(evaluator, board, alpha, beta);
            tt.update_evaluation_and_best_move(board, depth, NodeValue::pv_node(value), None);
            return value;
        }
        // depth >= 3, try null-move pruning
        if depth >= 3 {
            if let Some(null_move_game) = board.null_move() {
                let score =
                    -Self::null_alpha_beta(evaluator, &null_move_game, depth - 3, -beta, -beta + 1);
                if score >= beta {
                    return beta;
                }
            }
        }
        // perform principal search
        let (value, best_move) = Self::principal_variation_search(
            evaluator,
            tt,
            board,
            depth,
            alpha,
            beta,
            check_extension_enabled,
        );
        // update value/best_move in transpostion tables
        Self::update_cache(tt, board, depth, alpha_orig, beta, value, best_move);
        value
    }
}

impl ChessAgent for AlphaBetaChessAgent {
    fn get_action(&self, game: &Game) -> Action {
        let alpha = self.evaluator.min_value();
        let beta = self.evaluator.max_value();

        for i in 1..=self.depth {
            Self::alpha_beta(
                self.evaluator.as_ref(),
                &self.tt,
                &game.current_position(),
                i,
                alpha,
                beta,
                true,
            );
        }

        // get best move
        let best_move = Self::expand(&self.tt, &game.current_position())[0];

        Action::MakeMove(best_move)
    }
}
