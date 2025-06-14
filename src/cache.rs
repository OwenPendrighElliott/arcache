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
///
/// The cache trait is useful for defining generic functions that can work with any cache implementation
/// Example:
/// ```
/// use arcache::{Cache, LFUCache, LRUCache};
/// use std::{hash::Hash, sync::Arc};
///
/// fn do_something<C>(cache: C)
/// where
///     C: Cache<&'static str, String>,
/// {
///     cache.set("hello", "world".to_string());
///      if let Some(val) = cache.get(&"hello") {
///         println!("Got: {}", val);
///     }
/// }
///
/// fn main() {
///     // Both caches use &'static str for K and String for V
///     let lru_cache = LRUCache::<&'static str, String>::new(2);
///     do_something(lru_cache);
///
///     let lfu_cache = LFUCache::<&'static str, String>::new(2);
///     do_something(lfu_cache);
/// }
/// ```
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
