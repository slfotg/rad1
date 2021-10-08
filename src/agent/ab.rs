use super::ChessAgent;
use crate::eval::Evaluation;
use crate::move_sorter::MOVE_SORTER;
use crate::tt::*;
use chess::{Action, Board, BoardStatus, ChessMove, Game};
use std::cmp;
use std::sync::Arc;
use tokio::runtime::Runtime;

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
        for child_move in MOVE_SORTER.sorted_moves(board, None) {
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

fn check_extension(board: &Board, depth: &mut u8, check_extension_enabled: &mut bool) {
    if *check_extension_enabled && board.checkers().popcnt() > 0 {
        *depth += 1;
        // only allow one check extension in a search path
        *check_extension_enabled = false;
    }
}

pub struct AlphaBetaChessAgent {
    depth: u8,
    evaluator: Arc<TranspositionTable>,
    runtime: Runtime,
}

impl AlphaBetaChessAgent {
    pub fn new(depth: u8) -> Self {
        let runtime = Runtime::new().unwrap();
        let evaluator = Arc::default();
        AlphaBetaChessAgent {
            depth,
            evaluator,
            runtime,
        }
    }

    fn cached_evaluation(
        &self,
        board: &Board,
        depth: u8,
        alpha: &mut i16,
        beta: &mut i16,
    ) -> Option<i16> {
        match self.evaluator.get_evaluation(board) {
            None => None,
            Some(cached_eval) => {
                if cached_eval.depth() >= depth {
                    let value = cached_eval.evaluation();
                    match cached_eval.node_type() {
                        NodeType::PvNode => Some(value),
                        NodeType::AllNode => {
                            *alpha = cmp::max(*alpha, value);
                            None
                        }
                        NodeType::CutNode => {
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
        &self,
        board: &Board,
        depth: u8,
        alpha: i16,
        beta: i16,
        value: i16,
        best_move: ChessMove,
    ) {
        let tt = Arc::clone(&self.evaluator);
        let board = board.clone();
        self.runtime.spawn(async move {
            let cached_eval = if value <= alpha {
                // Beta
                CachedValue::new(depth, value, NodeType::AllNode)
            } else if value >= beta {
                // Alpha
                CachedValue::new(depth, value, NodeType::CutNode)
            } else {
                // Exact
                CachedValue::new(depth, value, NodeType::PvNode)
            };
            tt.update_evaluation(&board, cached_eval);
            tt.update_best_move(&board, depth, best_move);
        });
    }

    fn expand(&self, board: &Board) -> Vec<ChessMove> {
        MOVE_SORTER.sorted_moves(board, self.evaluator.best_move(board))
    }

    fn null_window_search(
        &self,
        board: &Board,
        depth: u8,
        alpha: i16,
        beta: i16,
        check_extension_enabled: bool,
    ) -> i16 {
        // Search with null window at first
        let value = -self.alpha_beta(
            board,
            depth - 1,
            -alpha - 1,
            -alpha,
            check_extension_enabled,
        );
        // Re-search the path with regular window if alpha < value < beta
        if alpha < value && value < beta {
            -self.alpha_beta(board, depth - 1, -beta, -alpha, check_extension_enabled)
        } else {
            value
        }
    }

    fn principal_variation_search(
        &self,
        board: &Board,
        depth: u8,
        mut alpha: i16,
        beta: i16,
        check_extension_enabled: bool,
    ) -> (i16, ChessMove) {
        let moves = self.expand(board);
        let mut best_move = moves[0];

        // Search down the principal variation path first with regular window
        let value = -self.alpha_beta(
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
            let value = self.null_window_search(
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
        &self,
        board: &Board,
        mut depth: u8,
        mut alpha: i16,
        mut beta: i16,
        mut check_extension_enabled: bool,
    ) -> i16 {
        check_extension(board, &mut depth, &mut check_extension_enabled);
        let status = board.status();
        let alpha_orig = alpha;
        // Get cached evaluation if it exists and update alpha/beta accordingly
        // If an exact value is already cached, return that immediately
        if let Some(value) = self.cached_evaluation(board, depth, &mut alpha, &mut beta) {
            return value;
        }
        // If game is over, return evaluation
        if status != BoardStatus::Ongoing {
            return Evaluation::evaluate(board);
        }
        // If depth is 0, evaluate after quiesence search, cache and return
        if depth == 0 {
            let value = q_search(board, alpha, beta);
            self.evaluator
                .update_evaluation(board, CachedValue::new(depth, value, NodeType::PvNode));
            return value;
        }
        // depth >= 3, try null-move pruning
        if depth >= 3 {
            if let Some(null_move_game) = board.null_move() {
                let score = -null_alpha_beta(&null_move_game, depth - 3, -beta, -beta + 1);
                if score >= beta {
                    return beta;
                }
            }
        }
        // perform principal search
        let (value, best_move) =
            self.principal_variation_search(board, depth, alpha, beta, check_extension_enabled);
        // update value/best_move in transpostion tables
        self.update_cache(board, depth, alpha_orig, beta, value, best_move);
        value
    }
}

impl ChessAgent for AlphaBetaChessAgent {
    fn get_action(&self, game: &Game) -> Action {
        let alpha = Evaluation::MIN;
        let beta = Evaluation::MAX;

        for i in 1..=self.depth {
            self.alpha_beta(&game.current_position(), i, alpha, beta, true);
            let best_move = self.evaluator.best_move(&game.current_position());
            let evaluation = self
                .evaluator
                .get_shallow_evaluation(&game.current_position());
            println!("{} - {:?} = {:?}", i, best_move, evaluation);
        }

        // get best move
        let best_move = self.evaluator.best_move(&game.current_position()).unwrap();

        println!("Best move: {}", best_move);
        Action::MakeMove(best_move)
    }
}
