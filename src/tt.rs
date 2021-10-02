use crate::move_hash;
use std::sync::Mutex;

use chess::{Board, ChessMove};

const CACHE_SIZE: usize = 16777216;
const DEFAULT_REPLACEMENT_STRATEGY: ReplacementStrategy = ReplacementStrategy::DepthPreferred;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementStrategy {
    Always,
    DepthPreferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeValue {
    hash: u64,
    depth: usize,
    evaluation: i16,
    best_move_hash: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    PvNode,
    AllNode,
    CutNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedValue {
    pub node_value: NodeValue,
    pub node_type: NodeType,
}

impl Default for CachedValue {
    fn default() -> Self {
        Self {
            node_value: NodeValue::default(),
            node_type: NodeType::PvNode,
        }
    }
}

impl Default for NodeValue {
    fn default() -> Self {
        Self {
            hash: 0,
            depth: 1000,
            evaluation: 0,
            best_move_hash: 0,
        }
    }
}

impl NodeValue {
    pub fn new(hash: u64, depth: usize, evaluation: i16, best_move: Option<ChessMove>) -> Self {
        Self {
            hash,
            depth,
            evaluation,
            best_move_hash: if let Some(chess_move) = best_move {
                move_hash::get_hash(chess_move)
            } else {
                0
            },
        }
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn value(&self) -> i16 {
        self.evaluation
    }

    pub fn best_move(&self) -> Option<ChessMove> {
        if self.best_move_hash != 0 {
            Some(move_hash::get_move(self.best_move_hash))
        } else {
            None
        }
    }
}

pub struct TranspositionTable {
    cache_size: u64,
    cache: Vec<Mutex<CachedValue>>,
    strategy: ReplacementStrategy,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::with_cache_size(CACHE_SIZE)
    }
}

impl TranspositionTable {
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self::new(cache_size, DEFAULT_REPLACEMENT_STRATEGY)
    }

    pub fn new(cache_size: usize, strategy: ReplacementStrategy) -> Self {
        let mut cache = Vec::with_capacity(cache_size);
        for _ in 0..cache_size {
            cache.push(Mutex::new(CachedValue::default()));
        }
        let cache_size = cache_size as u64;
        Self {
            cache,
            cache_size,
            strategy,
        }
    }

    pub fn get_evaluation(&self, board: &Board) -> Option<CachedValue> {
        let cached_value = {
            *self.cache[(board.get_hash() % self.cache_size) as usize]
                .lock()
                .unwrap()
        };
        if cached_value.node_value.hash() == board.get_hash() {
            Some(cached_value)
        } else {
            None
        }
    }

    pub fn update_evaluation(&self, board: &Board, cached_eval: CachedValue) {
        let mut cached_value = self.cache[(board.get_hash() % self.cache_size) as usize]
            .lock()
            .unwrap();
        if self.strategy == ReplacementStrategy::Always
            || cached_value.node_value.depth() >= cached_eval.node_value.depth()
        {
            *cached_value = cached_eval;
        }
    }
}
