use std::hash::Hash;
use std::sync::Arc;

/// CacheStats contains cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
    pub capacity: u64,
}

/// Cache trait defines the methods that a cache should implement and provides a shared interface for different cache implementations
pub trait Cache<K: Eq + Hash + Clone + Send + Sync, V: Send + Sync>: Send + Sync {
    fn get(&self, key: &K) -> Option<Arc<V>>;
    fn set(&self, key: K, value: V) -> Option<Arc<V>>;
    fn remove(&self, key: &K) -> Option<Arc<V>>;
    fn clear(&self);
    fn stats(&self) -> CacheStats;
    fn change_capacity(&self, capacity: u64);
}

pub mod fifo;
pub mod lfu;
pub mod lifo;
pub mod lru;
pub mod mru;
pub mod random_replacement;
pub mod ttl;
