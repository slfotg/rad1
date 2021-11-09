use crate::move_hash;
use crate::node::NodeValue;
use std::cell::RefCell;
use std::sync::Mutex;

use chess::{Board, ChessMove};

const CACHE_SIZE: usize = 30000000;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EvaluationHash {
    hash: u64,
    depth: u8,
    value: NodeValue<i16>,
    best_move_hash: u16,
}

pub struct TranspositionTable {
    cache_size: u64,
    deep_cache: Vec<Mutex<RefCell<(u8, EvaluationHash)>>>,
    shallow_cache: Vec<Mutex<RefCell<EvaluationHash>>>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(CACHE_SIZE)
    }
}

impl TranspositionTable {
    pub fn new(cache_size: usize) -> Self {
        let size = cache_size / 2;
        let mut deep_cache = Vec::with_capacity(size);
        let mut shallow_cache = Vec::with_capacity(size);
        for _ in 0..size {
            shallow_cache.push(Mutex::default());
            let deep_value = EvaluationHash {
                hash: 0,
                depth: 255,
                value: NodeValue::default(),
                best_move_hash: 0,
            };
            deep_cache.push(Mutex::new(RefCell::new((0, deep_value))));
        }
        Self {
            cache_size: size as u64,
            deep_cache,
            shallow_cache,
        }
    }

    pub fn best_move(&self, board: &Board) -> Option<ChessMove> {
        let hash = board.get_hash();
        // try from shallow cache first
        {
            let guard = self.shallow_cache[(hash % self.cache_size) as usize]
                .lock()
                .unwrap();
            let value = guard.borrow();
            if value.hash == hash && value.best_move_hash != 0 {
                return Some(move_hash::get_move(value.best_move_hash));
            }
        }
        // try from deep cache second
        {
            let guard = self.deep_cache[(hash % self.cache_size) as usize]
                .lock()
                .unwrap();
            let value = guard.borrow();
            if value.1.hash == hash && value.1.best_move_hash != 0 {
                return Some(move_hash::get_move(value.1.best_move_hash));
            }
        }
        // otherwise return None
        None
    }

    pub fn get_evaluation_and_depth(&self, board: &Board) -> Option<(NodeValue<i16>, u8)> {
        let hash = board.get_hash();
        // try from deep cache first
        {
            let guard = self.deep_cache[(hash % self.cache_size) as usize]
                .lock()
                .unwrap();
            let value = guard.borrow();
            if value.1.hash == hash && value.1.best_move_hash != 0 {
                return Some((value.1.value, value.1.depth));
            }
        }
        // try from shallow cache second
        {
            let guard = self.shallow_cache[(hash % self.cache_size) as usize]
                .lock()
                .unwrap();
            let value = guard.borrow();
            if value.hash == hash && value.best_move_hash != 0 {
                return Some((value.value, value.depth));
            }
        }
        None
    }

    pub fn update_evaluation_and_best_move(
        &self,
        board: &Board,
        depth: u8,
        node: NodeValue<i16>,
        best_move: Option<ChessMove>,
    ) {
        let hash = board.get_hash();
        // update shallow cache
        {
            let guard = self.shallow_cache[(hash % self.cache_size) as usize]
                .lock()
                .unwrap();
            let mut value = guard.borrow_mut();
            if value.depth <= depth {
                value.depth = depth;
                value.hash = hash;
                value.value = node;
                if let Some(chess_move) = best_move {
                    value.best_move_hash = move_hash::get_hash(chess_move);
                }
            }
        }
        // update deep cache
        {
            let guard = self.deep_cache[(hash % self.cache_size) as usize]
                .lock()
                .unwrap();
            let mut value = guard.borrow_mut();
            if value.1.depth >= depth {
                value.1.depth = depth;
                value.1.hash = hash;
                value.1.value = node;
                if let Some(chess_move) = best_move {
                    value.1.best_move_hash = move_hash::get_hash(chess_move);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TranspositionTable;
    use crate::node::NodeValue;
    use chess::{Board, ChessMove, Square};

    #[test]
    fn initial_board_eval() {
        let tt = TranspositionTable::new(1000);
        let board = Board::default();
        tt.update_evaluation_and_best_move(
            &board,
            1,
            NodeValue::pv_node(50),
            Some(ChessMove::new(Square::E2, Square::E4, None)),
        );
        let (eval, depth) = tt.get_evaluation_and_depth(&board).unwrap();
        assert_eq!(depth, 1);
        assert_eq!(eval, NodeValue::pv_node(50));

        let chess_move = tt.best_move(&board).unwrap();
        assert_eq!(chess_move, ChessMove::new(Square::E2, Square::E4, None));
    }
}
