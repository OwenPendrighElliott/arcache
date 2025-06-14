use crate::cache::{Cache, CacheStats};
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// The inner data structure for the LRUCache.
struct LRUCacheInner<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> {
    capacity: u64,
    key_value_map: LinkedHashMap<K, Arc<V>>,
    hits: u64,
    misses: u64,
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> LRUCacheInner<K, V> {
    /// Create a new LRUCacheInner with the given capacity, internally capacity is reserved for the necessary data structures.
    fn new(capacity: u64) -> Self {
        LRUCacheInner {
            capacity,
            key_value_map: LinkedHashMap::with_capacity(capacity as usize),
            hits: 0,
            misses: 0,
        }
    }
}

/// LRUCache is a cache that uses the Least Frequently Recently (LRU) algorithm to evict items.
///
/// When the cache is full, the item which was least recently accessed is removed to make space for the new item.
///
/// All mutability is handled internally with a Mutex, so the cache can be shared between threads. Values are returned as Arcs to allow for shared ownership.
///
/// Example:
/// ```
/// use arcache::{Cache, LRUCache};
///
/// fn main() {
///     let cache = LRUCache::<&str, String>::new(10);
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
pub struct LRUCache<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> {
    inner: Mutex<LRUCacheInner<K, V>>,
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> LRUCache<K, V> {
    /// Create a new LRUCache with the given capacity.
    pub fn new(capacity: u64) -> Self {
        LRUCache {
            inner: Mutex::new(LRUCacheInner::new(capacity)),
        }
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> Cache<K, V> for LRUCache<K, V> {
    /// Get a value from the cache.
    fn get(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.key_value_map.get_refresh(key).cloned();
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
        let arc_value = Arc::new(value);
        let result = inner.key_value_map.insert(key, arc_value);
        if inner.key_value_map.len() as u64 > inner.capacity {
            inner.key_value_map.pop_front();
        }
        result
    }

    /// Remove a value from the cache.
    fn remove(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.remove(key)
    }

    /// Clear the cache, removing all items.
    fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.clear();
    }

    /// Get the cache statistics.
    fn stats(&self) -> CacheStats {
        let inner = self.inner.lock().unwrap();
        CacheStats {
            hits: inner.hits,
            misses: inner.misses,
            size: inner.key_value_map.len() as u64,
            capacity: inner.capacity,
        }
    }

    /// Change the capacity of the cache, if the new capacity is smaller than the current size, the least recently accessed items are removed
    fn change_capacity(&self, capacity: u64) {
        let mut inner = self.inner.lock().unwrap();
        let old_capacity = inner.capacity;
        inner.capacity = capacity;
        while inner.key_value_map.len() as u64 > inner.capacity {
            inner.key_value_map.pop_front();
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
    fn test_lru_cache() {
        let cache = LRUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        cache.set(3, 3);
        assert_eq!(cache.get(&2).map(|v| *v), None);
        cache.set(4, 4);
        assert_eq!(cache.get(&1).map(|v| *v), None);
        assert_eq!(cache.get(&3).map(|v| *v), Some(3));
        assert_eq!(cache.get(&4).map(|v| *v), Some(4));
    }

    #[test]
    fn test_lru_cache_change_capacity() {
        let cache = LRUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.change_capacity(1);
        assert_eq!(cache.get(&1).map(|v| *v), None);
        assert_eq!(cache.get(&2).map(|v| *v), Some(2));
    }

    #[test]
    fn test_lru_cache_clear() {
        let cache = LRUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.clear();
        assert_eq!(cache.get(&1).map(|v| *v), None);
        assert_eq!(cache.get(&2).map(|v| *v), None);
    }

    #[test]
    fn test_lru_stats() {
        let cache = LRUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.set(3, 3);
        assert_eq!(cache.stats().hits, 0);
        cache.get(&1);
        cache.get(&2);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);
        cache.get(&3);
        assert_eq!(cache.stats().hits, 2);
        assert_eq!(cache.stats().misses, 1);

        cache.set(4, 4);
        assert_eq!(cache.stats().size, 2);
        cache.get(&2);
        assert_eq!(cache.stats().misses, 2);
        cache.get(&4);
        assert_eq!(cache.stats().hits, 3);
    }
}
