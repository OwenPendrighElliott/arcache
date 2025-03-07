use std::hash::Hash;
use std::sync::Arc;

pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
    pub capacity: u64,
}

pub trait Cache<K: Eq + Hash + Clone + Sync, V>: Send + Sync {
    fn get(&mut self, key: &K) -> Option<Arc<V>>;
    fn set(&mut self, key: K, value: V);
    fn remove(&mut self, key: &K);
    fn clear(&mut self);
    fn stats(&self) -> CacheStats;
    fn change_capacity(&mut self, capacity: u64);
}

pub mod fifo;
pub mod lfu;
pub mod lru;
pub mod mru;
pub mod ttl;
