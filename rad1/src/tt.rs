use crate::move_hash;
use crate::node::NodeValue;
use std::cell::RefCell;
use std::sync::Mutex;

use crate::ChessMove;
use crate::Position;

const CACHE_SIZE: usize = 30000000;

type ThreadCountHash<T> = (u8, EvaluationHash<T>);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EvaluationHash<T> {
    hash: u64,
    depth: u8,
    value: NodeValue<T>,
    best_move_hash: u16,
}

pub struct TranspositionTable<T> {
    cache_size: u64,
    deep_cache: Vec<Mutex<RefCell<ThreadCountHash<T>>>>,
    shallow_cache: Vec<Mutex<RefCell<EvaluationHash<T>>>>,
}

impl<T> Default for TranspositionTable<T>
where
    T: Copy + Default,
{
    fn default() -> Self {
        Self::new(CACHE_SIZE)
    }
}

impl<T> TranspositionTable<T>
where
    T: Copy + Default,
{
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

    pub fn best_move(&self, position: &Position) -> Option<ChessMove> {
        let hash = position.get_hash();
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

    pub fn get_evaluation_and_depth(&self, position: &Position) -> Option<(NodeValue<T>, u8)> {
        let hash = position.get_hash();
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
        position: &Position,
        depth: u8,
        node: NodeValue<T>,
        best_move: Option<ChessMove>,
    ) {
        let hash = position.get_hash();
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
    use crate::{ChessMove, Position, Square};

    #[test]
    fn insert_new_value() {
        let tt = TranspositionTable::new(1000);
        let position = Position::default();
        tt.update_evaluation_and_best_move(
            &position,
            1,
            NodeValue::pv_node(50),
            Some(ChessMove::new(Square::E2, Square::E4, None)),
        );
        let (eval, depth) = tt.get_evaluation_and_depth(&position).unwrap();
        assert_eq!(depth, 1);
        assert_eq!(eval, NodeValue::pv_node(50));

        let chess_move = tt.best_move(&position).unwrap();
        assert_eq!(chess_move, ChessMove::new(Square::E2, Square::E4, None));
    }

    #[test]
    fn update_shallow_hash() {
        let tt = TranspositionTable::new(1000);
        let position = Position::default();
        tt.update_evaluation_and_best_move(
            &position,
            1,
            NodeValue::pv_node(50),
            Some(ChessMove::new(Square::E2, Square::E4, None)),
        );

        tt.update_evaluation_and_best_move(
            &position,
            8,
            NodeValue::pv_node(100),
            Some(ChessMove::new(Square::D2, Square::D4, None)),
        );
        let (eval, depth) = tt.get_evaluation_and_depth(&position).unwrap();
        assert_eq!(depth, 1);
        assert_eq!(eval, NodeValue::pv_node(50));

        let chess_move = tt.best_move(&position).unwrap();
        assert_eq!(chess_move, ChessMove::new(Square::D2, Square::D4, None));
    }

    #[test]
    fn update_deep_hash() {
        let tt = TranspositionTable::new(1000);
        let position = Position::default();
        tt.update_evaluation_and_best_move(
            &position,
            1,
            NodeValue::pv_node(50),
            Some(ChessMove::new(Square::E2, Square::E4, None)),
        );

        tt.update_evaluation_and_best_move(
            &position,
            0,
            NodeValue::pv_node(100),
            Some(ChessMove::new(Square::D2, Square::D4, None)),
        );
        let (eval, depth) = tt.get_evaluation_and_depth(&position).unwrap();
        assert_eq!(depth, 0);
        assert_eq!(eval, NodeValue::pv_node(100));

        let chess_move = tt.best_move(&position).unwrap();
        assert_eq!(chess_move, ChessMove::new(Square::E2, Square::E4, None));
    }
}
