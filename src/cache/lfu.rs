use crate::cache::{Cache, CacheStats};
use linked_hash_set::LinkedHashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// The inner data structure for the LFUCache.
struct LFUCacheInner<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> {
    capacity: u64,
    key_value_map: HashMap<K, Arc<V>>,
    counter: HashMap<K, u64>,
    freq_map: HashMap<u64, LinkedHashSet<K>>,
    hits: u64,
    misses: u64,
    min_freq: u64,
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> LFUCacheInner<K, V> {
    /// Create a new LFUCacheInner with the given capacity, internally capacity is reserved for the necessary data structures.
    fn new(capacity: u64) -> Self {
        LFUCacheInner {
            capacity: capacity,
            key_value_map: HashMap::with_capacity(capacity as usize),
            counter: HashMap::with_capacity(capacity as usize),
            freq_map: HashMap::new(),
            hits: 0,
            misses: 0,
            min_freq: 0,
        }
    }

    /// Increase the frequency of the given key.
    fn increase_freq(&mut self, key: &K) {
        let freq = *self.counter.get(key).unwrap_or(&0);
        *self.counter.entry(key.clone()).or_default() += 1;
        self.freq_map.entry(freq).or_default().remove(key);

        if self.freq_map.get(&freq).is_none() {
            if freq == self.min_freq {
                self.min_freq += 1;
            }
            self.freq_map.remove(&freq);
        }
        self.freq_map
            .entry(freq + 1)
            .or_default()
            .insert(key.clone());
    }

    /// Remove the least frequent item from the cache.
    fn remove_least_freq(&mut self) {
        if let Some(bucket) = self.freq_map.get_mut(&self.min_freq) {
            if let Some(key) = bucket.pop_front() {
                self.key_value_map.remove(&key);
                self.counter.remove(&key);
            }
            if bucket.is_empty() {
                self.freq_map.remove(&self.min_freq);
            }
        }
    }
}

/// LFUCache is a cache that uses the Least Frequently Used (LFU) algorithm to evict items.
///
/// When the cache is full, the item with the lowest frequency of access is evicted.
///
/// All mutability is handled internally with a Mutex, so the cache can be shared between threads. Values are returned as Arcs to allow for shared ownership.
///
/// Example:
/// ```
/// use arcache::{Cache, LFUCache};
///
/// fn main() {
///     let cache = LFUCache::<&str, String>::new(10);
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
pub struct LFUCache<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> {
    inner: Mutex<LFUCacheInner<K, V>>,
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> LFUCache<K, V> {
    /// Create a new LFUCache with the given capacity.
    pub fn new(capacity: u64) -> Self {
        LFUCache {
            inner: Mutex::new(LFUCacheInner::new(capacity)),
        }
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Send + Sync> Cache<K, V> for LFUCache<K, V> {
    /// Get a value from the cache.
    fn get(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.key_value_map.get(key).cloned();

        if result.is_some() {
            inner.hits += 1;
            inner.increase_freq(key);
        } else {
            inner.misses += 1;
        }
        result
    }

    /// Set a value in the cache.
    fn set(&self, key: K, value: V) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        let arc_value = Arc::new(value);
        let existing_value = inner.key_value_map.get(&key).cloned();

        if existing_value.is_some() {
            inner.key_value_map.insert(key.clone(), arc_value);
            inner.increase_freq(&key);
        } else {
            if inner.key_value_map.len() as u64 >= inner.capacity {
                inner.remove_least_freq();
            }
            inner.key_value_map.insert(key.clone(), arc_value);
            *inner.counter.entry(key.clone()).or_default() += 1;
            inner.freq_map.entry(1).or_default().insert(key);
            inner.min_freq = 1;
        }
        existing_value
    }

    /// Remove a value from the cache.
    fn remove(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();

        let result = inner.key_value_map.remove(key);
        // if let Some(_) = result {
        //     inner.counter.remove(key);
        //     inner.freq_map.get_mut(&1).map(|bucket| bucket.remove(key));
        // }

        if result.is_some() {
            inner.counter.remove(key);
            let freq = *inner.counter.get(key).unwrap_or(&0);
            if let Some(bucket) = inner.freq_map.get_mut(&freq) {
                bucket.remove(key);
                if bucket.is_empty() {
                    inner.freq_map.remove(&1);
                    inner.min_freq = 0;
                }
            }
        }
        result
    }

    /// Clear the cache.
    fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.clear();
        inner.freq_map.clear();
        inner.counter.clear();
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

    /// Change the capacity of the cache, if the new capacity is smaller than the current size, the least frequently used items are removed.
    fn change_capacity(&self, capacity: u64) {
        let mut inner = self.inner.lock().unwrap();
        let old_capacity = inner.capacity;
        inner.capacity = capacity;
        while inner.key_value_map.len() as u64 > inner.capacity {
            inner.remove_least_freq();
        }

        if old_capacity < inner.capacity {
            let additional = (inner.capacity - old_capacity) as usize;
            inner.key_value_map.reserve(additional);
            inner.counter.reserve(additional);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfu_cache() {
        let cache = LFUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        cache.set(3, 3);
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        assert_eq!(cache.get(&3).map(|v| *v), Some(3));
        cache.set(4, 4);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_lfu_cache_change_capacity() {
        let cache = LFUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.change_capacity(1);
        assert_eq!(cache.get(&2).map(|v| *v), Some(2));
        assert_eq!(cache.get(&1), None);
    }

    #[test]
    fn test_lfu_cache_clear() {
        let cache = LFUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.clear();
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_lfu_cache_stats() {
        let cache = LFUCache::new(2);
        cache.set(1, 1);
        cache.set(2, 2);
        cache.get(&1);
        cache.get(&2);
        cache.get(&3);
        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 2);
        assert_eq!(stats.capacity, 2);
    }
}
