use super::ChessAgent;
use crate::eval::Evaluation;
use crate::tt::*;
use crate::move_sorter::MOVE_SORTER;
use chess::{Board, BoardStatus, ChessMove};
use std::cmp;
use std::cmp::Ordering;

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
fn null_alpha_beta(board: &Board, depth: usize, mut alpha: i16, beta: i16) -> i16 {
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
    depth: usize,
    value: &mut i16,
    alpha: &mut i16,
    beta: &mut i16,
) -> Option<i16> {
    match trans_table.get_evaluation(board) {
        CachedValue::Empty => None,
        cached_eval => {
            if cached_eval.depth() >= depth {
                *value = cached_eval.value();
                match cached_eval {
                    CachedValue::Exact(_, _, evaluation) => Some(evaluation),
                    CachedValue::Alpha(_, _, evaluation) => {
                        *alpha = cmp::max(*alpha, evaluation);
                        None
                    }
                    CachedValue::Beta(_, _, evaluation) => {
                        *beta = cmp::max(*beta, evaluation);
                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

struct Node {
    hash: u64,
    evaluation: Option<i16>,
    chess_move: Option<ChessMove>,
    children: Vec<Node>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            hash: Board::default().get_hash(),
            evaluation: None,
            chess_move: None,
            children: vec![],
        }
    }
}

impl Node {
    fn new(board: Board, chess_move: Option<ChessMove>) -> Self {
        Self {
            hash: board.get_hash(),
            evaluation: None,
            chess_move,
            children: vec![],
        }
    }

    fn best_move(&self) -> Option<ChessMove> {
        self.children[0].chess_move
    }

    fn first_child(&mut self) -> Self {
        self.children.remove(0)
    }

    fn find_child(&mut self, board: &Board) -> Option<Self> {
        for i in 0..self.children.len() {
            if self.children[i].hash == board.get_hash() {
                return Some(self.children.remove(i));
            }
        }
        None
    }

    fn is_expanded(&self) -> bool {
        !self.children.is_empty()
    }

    fn expand(&mut self, board: &Board) {
        if !self.is_expanded() {
            let moves = MOVE_SORTER.sorted_moves(board);
            for m in moves.into_iter() {
                self.children.push(Node::new(board.make_move_new(m), Some(m)));
            }
        }
    }

    fn size(&self) -> usize {
        let mut size = 1;
        for i in 0..self.children.len() {
            size += self.children[i].size();
        }
        size
    }

    fn sort_children_by_evaluation(&mut self) {
        self.children
            .sort_by(|a, b| match (a.evaluation, b.evaluation) {
                (None, None) => Ordering::Equal,
                (None, _) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a_val), Some(b_val)) => a_val.cmp(&b_val),
            });
    }

    fn alpha_beta(
        &mut self,
        trans_table: &TranspositionTable,
        board: &Board,
        mut depth: usize,
        mut alpha: i16,
        mut beta: i16,
    ) -> i16 {
        if board.checkers().popcnt() > 0 && depth > 0 {
            depth += 1;
        }
        let status = board.status();
        let alpha_orig = alpha;
        let mut value = Evaluation::MIN;
        if depth > 0 {
            self.expand(board);
        }
        let cached_evaluation =
            cached_evaluation(trans_table, board, depth, &mut value, &mut alpha, &mut beta);
        let value = if let Some(evaluation) = cached_evaluation {
            evaluation
        } else if status != BoardStatus::Ongoing {
            Evaluation::evaluate(board)
        } else if depth == 0 {
            let value = q_search(board, alpha, beta);
            trans_table.update_evaluation(board, CachedValue::Exact(board.get_hash(), depth, value));
            value
        } else {
            if depth >= 3 {
                if let Some(null_move_game) = board.null_move() {
                    let score = -null_alpha_beta(&null_move_game, depth - 3, -beta, -beta + 1);
                    if score >= beta {
                        self.evaluation = Some(beta);
                        return beta;
                    }
                }
            }
            for child_node in self.children.iter_mut() {
                let child_move = child_node.chess_move.clone().unwrap();

                let child_value = -child_node.alpha_beta(
                    trans_table,
                    &board.make_move_new(child_move),
                    depth - 1,
                    -beta,
                    -alpha,
                );
                value = cmp::max(child_value, value);
                alpha = cmp::max(alpha, value);
                if alpha >= beta {
                    break;
                }
            }
            self.sort_children_by_evaluation();
            let cached_eval = if value <= alpha_orig {
                CachedValue::Beta(board.get_hash(), depth, value)
            } else if value >= beta {
                CachedValue::Alpha(board.get_hash(), depth, value)
            } else {
                CachedValue::Exact(board.get_hash(), depth, value)
            };
            trans_table.update_evaluation(&board, cached_eval);
            value
        };
        self.evaluation = Some(value);
        value
    }
}

pub struct AlphaBetaChessAgent {
    depth: usize,
    evaluator: TranspositionTable,
    head: Option<Node>,
}

impl AlphaBetaChessAgent {
    pub fn new(depth: usize) -> Self {
        AlphaBetaChessAgent {
            depth,
            evaluator: TranspositionTable::default(),
            head: Some(Node::default()),
        }
    }

    fn size(&self) -> usize {
        self.head.as_ref().unwrap().size()
    }

    fn update_head(&mut self, board: &Board) {
        let child = self.head.as_mut().unwrap().find_child(board);
        if child.is_some() {
            self.head = child;
        } else {
            self.head = Some(Node::new(*board, None));
            self.evaluator = TranspositionTable::default();
        }
    }
}

impl ChessAgent for AlphaBetaChessAgent {
    fn best_move(&mut self, board: &Board) -> ChessMove {
        self.update_head(&board);
        let alpha = Evaluation::MIN;
        let beta = Evaluation::MAX;
        let mut head = self.head.take().unwrap();
        for i in 1..=self.depth {
            head.alpha_beta(&self.evaluator, &board, i, alpha, beta);
            println!(
                "{} - {} = {}",
                i,
                &head.best_move().unwrap(),
                head.evaluation.unwrap(),
            );
        }

        // get best move
        let best_move = head.best_move().unwrap();

        // update head of tree
        let first_child = head.first_child();
        self.head = Some(first_child);

        println!("Best move: {}", best_move);
        println!("Size: {}", self.size());
        best_move
    }
}
