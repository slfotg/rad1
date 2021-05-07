use std::sync::Mutex;

use crate::game::Game;

const CACHE_SIZE: usize = 16777216;

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
            _ => 0,
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
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(CACHE_SIZE)
    }
}

impl TranspositionTable {
    pub fn new(cache_size: usize) -> Self {
        let mut cache = Vec::with_capacity(cache_size);
        for _ in 0..cache_size {
            cache.push(Mutex::new(CachedValue::Empty));
        }
        let cache_size = cache_size as u64;
        Self { cache, cache_size }
    }

    pub fn get_evaluation(&self, game: &Game) -> CachedValue {
        let cached_value = {
            *self.cache[(game.hash() % self.cache_size) as usize]
                .lock()
                .unwrap()
        };
        match cached_value {
            CachedValue::Empty => CachedValue::Empty,
            val => {
                if val.hash() == game.hash() {
                    val
                } else {
                    CachedValue::Empty
                }
            }
        }
    }

    pub fn update_evaluation(&self, game: &Game, cached_eval: CachedValue) {
        let mut cached_value = self.cache[(game.hash() % self.cache_size) as usize]
            .lock()
            .unwrap();
        *cached_value = cached_eval;
    }
}
