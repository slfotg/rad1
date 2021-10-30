use crate::move_hash;
use std::cell::RefCell;

use chess::{Board, ChessMove};

const CACHE_SIZE: usize = 50000000;
const SHALLOW_HASH_SIZE: u64 = 500000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    PvNode,
    AllNode,
    CutNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedValue {
    depth: u8,
    evaluation: i16,
    node_type: NodeType,
}

impl CachedValue {
    pub fn new(depth: u8, evaluation: i16, node_type: NodeType) -> Self {
        Self {
            depth,
            evaluation,
            node_type,
        }
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn evaluation(&self) -> i16 {
        self.evaluation
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvaluationHash {
    hash: u64,
    depth: u8,
    evaluation: i16,
    node_type: NodeType,
}

impl Default for EvaluationHash {
    fn default() -> Self {
        Self {
            hash: 0,
            depth: 255,
            evaluation: 0,
            node_type: NodeType::PvNode,
        }
    }
}

impl EvaluationHash {
    fn cached_value(&self) -> CachedValue {
        CachedValue::new(self.depth, self.evaluation, self.node_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShallowHash {
    hash: u64,
    depth: u8,
    evaluation: i16,
    best_move_hash: u16,
}

impl Default for ShallowHash {
    fn default() -> Self {
        Self {
            hash: 0,
            depth: 0,
            evaluation: 0,
            best_move_hash: 0,
        }
    }
}

#[derive(Clone)]
pub struct TranspositionTable {
    cache_size: u64,
    cache: Vec<RefCell<EvaluationHash>>,
    shallow_hash: Vec<RefCell<ShallowHash>>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::with_cache_size(CACHE_SIZE)
    }
}

impl TranspositionTable {
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self::new(cache_size)
    }

    pub fn clear(&mut self) {
        for i in 0..self.cache.len() {
            self.cache[i] = RefCell::default();
        }
        for i in 0..self.shallow_hash.len() {
            self.shallow_hash[i] = RefCell::default();
        }
    }

    pub fn new(cache_size: usize) -> Self {
        let mut cache = Vec::with_capacity(cache_size);
        for _ in 0..cache_size {
            cache.push(RefCell::default());
        }
        let mut shallow_hash = Vec::with_capacity(SHALLOW_HASH_SIZE as usize);
        for _ in 0..SHALLOW_HASH_SIZE {
            shallow_hash.push(RefCell::default());
        }
        let cache_size = cache_size as u64;
        Self {
            cache_size,
            cache,
            shallow_hash,
        }
    }

    pub fn get_evaluation(&self, board: &Board) -> Option<CachedValue> {
        let evaluation_hash =
            { *self.cache[(board.get_hash() % self.cache_size) as usize].borrow() };
        if evaluation_hash.hash == board.get_hash() {
            Some(evaluation_hash.cached_value())
        } else {
            None
        }
    }

    pub fn get_evaluation_debug(&self, board: &Board) -> Option<CachedValue> {
        let evaluation_hash =
            { *self.cache[(board.get_hash() % self.cache_size) as usize].borrow() };
        println!(
            "current hash: {}, board hash: {}, value: {}",
            evaluation_hash.hash,
            board.get_hash(),
            evaluation_hash.evaluation
        );
        if evaluation_hash.hash == board.get_hash() {
            Some(evaluation_hash.cached_value())
        } else {
            None
        }
    }

    pub fn update_evaluation(&self, board: &Board, cached_eval: CachedValue) {
        {
            let mut evaluation_hash =
                self.cache[(board.get_hash() % self.cache_size) as usize].borrow_mut();
            if evaluation_hash.depth >= cached_eval.depth() {
                evaluation_hash.hash = board.get_hash();
                evaluation_hash.depth = cached_eval.depth();
                evaluation_hash.evaluation = cached_eval.evaluation();
                evaluation_hash.node_type = cached_eval.node_type();
            }
        }
        {
            let mut shallow_hash =
                self.shallow_hash[(board.get_hash() % SHALLOW_HASH_SIZE) as usize].borrow_mut();
            if shallow_hash.depth <= cached_eval.depth() {
                shallow_hash.hash = board.get_hash();
                shallow_hash.depth = cached_eval.depth();
                shallow_hash.evaluation = cached_eval.evaluation();
            }
        }
    }

    pub fn best_move(&self, board: &Board) -> Option<ChessMove> {
        let shallow_hash =
            self.shallow_hash[(board.get_hash() % SHALLOW_HASH_SIZE) as usize].borrow();
        if shallow_hash.best_move_hash == 0 || shallow_hash.hash != board.get_hash() {
            None
        } else {
            Some(move_hash::get_move(shallow_hash.best_move_hash))
        }
    }

    pub fn update_best_move(&self, board: &Board, depth: u8, best_move: ChessMove) {
        let mut shallow_hash =
            self.shallow_hash[(board.get_hash() % SHALLOW_HASH_SIZE) as usize].borrow_mut();
        if shallow_hash.depth <= depth {
            shallow_hash.hash = board.get_hash();
            shallow_hash.depth = depth;
            shallow_hash.best_move_hash = move_hash::get_hash(best_move);
        }
    }

    pub fn get_shallow_evaluation(&self, board: &Board) -> Option<i16> {
        let shallow_hash =
            self.shallow_hash[(board.get_hash() % SHALLOW_HASH_SIZE) as usize].borrow();
        if shallow_hash.best_move_hash == 0 || shallow_hash.hash != board.get_hash() {
            None
        } else {
            Some(shallow_hash.evaluation)
        }
    }
}
