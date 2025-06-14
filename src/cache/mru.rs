use crate::cache::{Cache, CacheStats};
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// The inner data structure for the MRUCache.
struct MRUCacheInner<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> {
    capacity: u64,
    key_value_map: LinkedHashMap<K, Arc<V>>,
    hits: u64,
    misses: u64,
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> MRUCacheInner<K, V> {
    /// Create a new MRUCacheInner with the given capacity, internally capacity is reserved for the necessary data structures.
    fn new(capacity: u64) -> Self {
        MRUCacheInner {
            capacity,
            key_value_map: LinkedHashMap::with_capacity(capacity as usize),
            hits: 0,
            misses: 0,
        }
    }
}

/// MRUCache is a cache that uses the Most Recently Used (MRU) algorithm to evict items.
///
/// When the cache is full, the item with the most recent access is removed to make space for the new item. This is the opposite of the LRU cache.
///
/// All mutability is handled internally with a Mutex, so the cache can be shared between threads. Values are returned as Arcs to allow for shared ownership.
///
/// Example:
/// ```
/// use arcache::{Cache, MRUCache};
///
/// fn main() {
///     let cache = MRUCache::<&str, String>::new(10);
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
pub struct MRUCache<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> {
    inner: Mutex<MRUCacheInner<K, V>>,
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> MRUCache<K, V> {
    /// Create a new MRUCache with the given capacity.
    pub fn new(capacity: u64) -> Self {
        MRUCache {
            inner: Mutex::new(MRUCacheInner::new(capacity)),
        }
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> Cache<K, V> for MRUCache<K, V> {
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

        if inner.key_value_map.len() as u64 + 1 > inner.capacity {
            inner.key_value_map.pop_back();
        }
        inner.key_value_map.insert(key, arc_value)
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

    /// Change the capacity of the cache, if the new capacity is less than the current capacity, the cache will evict the most recently used items until the size equals the new capacity.
    fn change_capacity(&self, capacity: u64) {
        let mut inner = self.inner.lock().unwrap();
        let old_capacity = inner.capacity;
        inner.capacity = capacity;
        while inner.key_value_map.len() as u64 > inner.capacity {
            inner.key_value_map.pop_back();
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
    fn test_mru_cache() {
        let cache = MRUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        cache.set(3, 3);
        assert_eq!(cache.get(&2).map(|v| *v), Some(2));
        cache.set(4, 4);
        assert_eq!(cache.get(&1).map(|v| *v), None);
        assert_eq!(cache.get(&3).map(|v| *v), Some(3));
        assert_eq!(cache.get(&4).map(|v| *v), Some(4));
    }

    #[test]
    fn test_mru_cache_change_capacity() {
        let cache = MRUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.change_capacity(1);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        assert_eq!(cache.get(&2).map(|v| *v), None);
    }

    #[test]
    fn test_mru_cache_clear() {
        let cache = MRUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.clear();
        assert_eq!(cache.get(&1).map(|v| *v), None);
        assert_eq!(cache.get(&2).map(|v| *v), None);
    }
}
