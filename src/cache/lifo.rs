use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::cache::{Cache, CacheStats};

/// LIFOCacheInner contains the inner data structure for the LIFOCache.
struct LIFOCacheInner<K: Eq + Hash + Send, V: Send + Sync> {
    capacity: u64,
    key_value_map: HashMap<K, Arc<V>>,
    lifo: Vec<K>,
    hits: u64,
    misses: u64,
}

impl<K: Eq + Hash + Send, V: Send + Sync> LIFOCacheInner<K, V> {
    /// Create a new LIFOCacheInner with the given capacity, internally capacity is reserved for the necessary data structures.
    fn new(capacity: u64) -> Self {
        LIFOCacheInner {
            capacity,
            key_value_map: HashMap::with_capacity(capacity as usize),
            lifo: Vec::with_capacity(capacity as usize),
            hits: 0,
            misses: 0,
        }
    }
}

/// LIFOCache is a last-in-first-out cache implementation.
///
/// When the cache is full, the newest item is evicted from the cache.
///
/// All mutability is handled internally with a Mutex, so the cache can be shared between threads. Values are returned as Arcs to allow for shared ownership.
///
/// Example:
/// ```
/// use cachers::{Cache, LIFOCache};
///
/// fn main() {
///     let cache = LIFOCache::<&str, String>::new(10);
///     
///     let original_value = cache.set("key", "value".to_string());
///
///     assert!(original_value.is_none());
///     
///     let value = cache.get(&"key");
///
///     assert!(value.is_some());
///     assert_eq!(*value.unwrap(), "value".to_string());
///     println!("{:?}", cache.stats());
/// }
/// ```
pub struct LIFOCache<K: Eq + Hash + Send, V: Send + Sync> {
    inner: Mutex<LIFOCacheInner<K, V>>,
}

impl<K: Eq + Hash + Sync + Send, V: Send + Sync> LIFOCache<K, V> {
    /// Create a new LIFOCache with the given capacity.
    pub fn new(capacity: u64) -> Self {
        LIFOCache {
            inner: Mutex::new(LIFOCacheInner::new(capacity)),
        }
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> Cache<K, V> for LIFOCache<K, V> {
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
            if let Some(oldest_key) = inner.lifo.pop() {
                inner.key_value_map.remove(&oldest_key);
            }
        }
        let arc_value = Arc::new(value);
        let result = inner.key_value_map.insert(key.clone(), arc_value);
        inner.lifo.push(key);
        result
    }

    /// Remove a value from the cache.
    fn remove(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.key_value_map.remove(key);
        if let Some(pos) = inner.lifo.iter().position(|k| k == key) {
            inner.lifo.remove(pos);
        }
        result
    }

    /// Clear the cache.
    fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.clear();
        inner.lifo.clear();
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
            if let Some(oldest_key) = inner.lifo.pop() {
                inner.key_value_map.remove(&oldest_key);
            }
        }

        if old_capacity < inner.capacity {
            let additional = (inner.capacity - old_capacity) as usize;
            inner.key_value_map.reserve(additional);
            inner.lifo.reserve(additional);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifo_cache() {
        let cache = LIFOCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        cache.set(3, 3);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        assert_eq!(cache.get(&2), None);
        cache.set(4, 4);
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        assert_eq!(cache.get(&4).map(|v| *v), Some(4));
    }

    #[test]
    fn test_lifo_cache_clear() {
        let cache = LIFOCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.clear();
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_lifo_cache_change_capacity() {
        let cache = LIFOCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.change_capacity(1);
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
    }
}
