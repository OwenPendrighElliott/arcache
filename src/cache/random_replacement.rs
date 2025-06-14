use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::cache::{Cache, CacheStats};

/// RandomReplacementCacheInner contains the inner data structure for the RandomReplacementCache.
struct RandomReplacementCacheInner<K: Eq + Hash + Send, V: Send + Sync> {
    capacity: u64,
    key_value_map: HashMap<K, Arc<V>>,
    keys: Vec<K>,
    hits: u64,
    misses: u64,
}

impl<K: Eq + Hash + Send, V: Send + Sync> RandomReplacementCacheInner<K, V> {
    /// Create a new RandomReplacementCacheInner with the given capacity, internally capacity is reserved for the necessary data structures.
    fn new(capacity: u64) -> Self {
        RandomReplacementCacheInner {
            capacity,
            key_value_map: HashMap::with_capacity(capacity as usize),
            keys: Vec::with_capacity(capacity as usize),
            hits: 0,
            misses: 0,
        }
    }
}

/// RandomReplacementCache is a cache which evicts items randomly.
///
/// When the cache is full, a random item is removed to make space for the new item.
///
/// All mutability is handled internally with a Mutex, so the cache can be shared between threads. Values are returned as Arcs to allow for shared ownership.
///
/// Example:
/// ```
/// use arcache::{Cache, RandomReplacementCache};
///
/// let cache = RandomReplacementCache::<&str, String>::new(10);
///     
/// let original_value = cache.set("key", "value".to_string());
///
/// assert!(original_value.is_none());
///     
/// let value = cache.get(&"key");
///
/// assert!(value.is_some());
/// assert_eq!(*value.unwrap(), "value".to_string());
/// println!("{:?}", cache.stats());
/// ```
pub struct RandomReplacementCache<K: Eq + Hash + Send, V: Send + Sync> {
    inner: Mutex<RandomReplacementCacheInner<K, V>>,
}

impl<K: Eq + Hash + Sync + Send, V: Send + Sync> RandomReplacementCache<K, V> {
    /// Create a new RandomReplacementCache with the given capacity.
    pub fn new(capacity: u64) -> Self {
        RandomReplacementCache {
            inner: Mutex::new(RandomReplacementCacheInner::new(capacity)),
        }
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> Cache<K, V>
    for RandomReplacementCache<K, V>
{
    /// Get a value from the cache.
    fn get(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.key_value_map.get(key).cloned();

        if result.is_some() {
            inner.hits += 1;
        } else {
            inner.misses += 1;
        }
        result
    }

    /// Set a value in the cache.
    fn set(&self, key: K, value: V) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        if inner.key_value_map.len() as u64 >= inner.capacity {
            let index = rand::rng().random_range(0..inner.keys.len());
            let removed_key = inner.keys.swap_remove(index);
            inner.key_value_map.remove(&removed_key);
        }
        let arc_value = Arc::new(value);
        inner.keys.push(key.clone());
        inner.key_value_map.insert(key, arc_value)
    }

    /// Remove a value from the cache.
    fn remove(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.key_value_map.remove(key);
        if let Some(pos) = inner.keys.iter().position(|k| k == key) {
            inner.keys.remove(pos);
        }
        result
    }

    /// Clear the cache.
    fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.clear();
        inner.keys.clear();
    }

    /// Get cache statistics.
    fn stats(&self) -> CacheStats {
        let inner = self.inner.lock().unwrap();
        CacheStats {
            hits: inner.hits,
            misses: inner.misses,
            size: inner.key_value_map.len() as u64,
            capacity: inner.capacity,
        }
    }

    /// Change the capacity of the cache, if the new capacity is smaller than the current size, the oldest items are removed.
    fn change_capacity(&self, capacity: u64) {
        let mut inner = self.inner.lock().unwrap();
        let old_capacity = inner.capacity;
        inner.capacity = capacity;
        while inner.key_value_map.len() as u64 > inner.capacity {
            let index = rand::rng().random_range(0..inner.keys.len());
            let removed_key = inner.keys.swap_remove(index);
            inner.key_value_map.remove(&removed_key);
        }

        if inner.capacity > old_capacity {
            let additional = (inner.capacity - old_capacity) as usize;
            inner.key_value_map.reserve(additional);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_replacement_cache() {
        let cache = RandomReplacementCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        cache.set(3, 3);
        assert_eq!(cache.get(&3).map(|v| *v), Some(3));
        assert!(cache.get(&1).is_none() || cache.get(&2).is_none());
        cache.set(4, 4);
        assert_eq!(cache.get(&4).map(|v| *v), Some(4));
    }

    #[test]
    fn test_random_replacement_cache_clear() {
        let cache = RandomReplacementCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.clear();
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_random_replacement_cache_change_capacity() {
        let cache = RandomReplacementCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.change_capacity(1);
        assert!(cache.get(&1).is_none() || cache.get(&2).is_none());
    }
}
