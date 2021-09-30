use std::sync::Mutex;

use chess::Board;

const CACHE_SIZE: usize = 16777216;
const DEFAULT_REPLACEMENT_STRATEGY: ReplacementStrategy = ReplacementStrategy::DepthPreferred;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementStrategy {
    Always,
    DepthPreferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachedValue {
    Empty,
    Exact(u64, usize, i16),
    Alpha(u64, usize, i16),
    Beta(u64, usize, i16),
}

impl CachedValue {
    pub fn hash(&self) -> u64 {
        match *self {
            Self::Exact(hash, _, _) => hash,
            Self::Alpha(hash, _, _) => hash,
            Self::Beta(hash, _, _) => hash,
            _ => 0,
        }
    }

    pub fn depth(&self) -> usize {
        match *self {
            Self::Exact(_, depth, _) => depth,
            Self::Alpha(_, depth, _) => depth,
            Self::Beta(_, depth, _) => depth,
            _ => usize::max_value(),
        }
    }

    pub fn value(&self) -> i16 {
        match *self {
            Self::Exact(_, _, value) => value,
            Self::Alpha(_, _, value) => value,
            Self::Beta(_, _, value) => value,
            _ => 0,
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
            cache.push(Mutex::new(CachedValue::Empty));
        }
        let cache_size = cache_size as u64;
        Self {
            cache,
            cache_size,
            strategy,
        }
    }

    pub fn get_evaluation(&self, board: &Board) -> CachedValue {
        let cached_value = {
            *self.cache[(board.get_hash() % self.cache_size) as usize]
                .lock()
                .unwrap()
        };
        match cached_value {
            CachedValue::Empty => CachedValue::Empty,
            val => {
                if val.hash() == board.get_hash() {
                    val
                } else {
                    CachedValue::Empty
                }
            }
        }
    }

    pub fn update_evaluation(&self, board: &Board, cached_eval: CachedValue) {
        let mut cached_value = self.cache[(board.get_hash() % self.cache_size) as usize]
            .lock()
            .unwrap();
        if self.strategy == ReplacementStrategy::Always
            || cached_value.depth() >= cached_eval.depth()
        {
            *cached_value = cached_eval;
        }
    }
}
