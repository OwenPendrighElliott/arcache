use linked_hash_map::LinkedHashMap;
use rand::Rng;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::cache::{Cache, CacheStats};

/// An internal struct of the TTL cache for storing data along with its expiry time.
#[derive(Clone)]
struct DataWithLifetime<V> {
    data: Arc<V>,
    expiry: Instant,
}

/// The inner data structure for the TTLCache.
struct TTLCacheInner<K, V> {
    ttl: Duration,
    jitter: Duration,
    check_interval: Duration,
    capacity: u64,
    key_value_map: LinkedHashMap<K, DataWithLifetime<V>>,
    hits: u64,
    misses: u64,
}

/// TTLCache is a cache that uses adds a time-to-live (TTL) to each item.
///
/// This cache will automatically evict items that have expired. The TTL is set when the item is added to the cache. A thread runs in the background to continually check for items that have expired. Thus there is no overhead relating to access frequency. If the cache is at capacity and a new item is added, the least recently accessed item is removed.
///
/// All mutability is handled internally with a Mutex, so the cache can be shared between threads. Values are returned as Arcs to allow for shared ownership.
///
/// The TTLCache has additional parameters in its constructor compared to other caches.
///
/// Example:
/// ```
/// use cachers::{Cache, TTLCache};
/// use std::time::Duration;
///
/// fn main() {
///     let ttl = Duration::from_secs(1);
///     let check_interval = Duration::from_millis(100);
///     let jitter = Duration::from_millis(10);
///     let capacity = 10;
///     let cache = TTLCache::<&str, String>::new(ttl, check_interval, jitter, capacity);
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
pub struct TTLCache<K: Eq + Hash + Clone + Send + 'static, V: Send + Sync + 'static> {
    inner: Arc<Mutex<TTLCacheInner<K, V>>>,
}

impl<K: Eq + Hash + Clone + Send + 'static, V: Send + Sync + 'static> TTLCache<K, V> {
    /// Create a new TTLCache with the given time-to-live (TTL), check interval, jitter, and capacity.
    /// + The TTL is the amount of time an item will be stored in the cache before it is evicted.
    /// + The check interval is how often the cache will check for expired items.
    /// + The jitter is a random amount of time added to the check interval to prevent all items from expiring at the same time.
    /// + The capacity is the maximum number of items that can be stored in the cache.
    pub fn new(ttl: Duration, check_interval: Duration, jitter: Duration, capacity: u64) -> Self {
        let inner = Arc::new(Mutex::new(TTLCacheInner {
            ttl,
            jitter,
            check_interval,
            capacity,
            key_value_map: LinkedHashMap::new(),
            hits: 0,
            misses: 0,
        }));

        let inner_clone = Arc::clone(&inner);

        // Background thread for evicting expired items
        thread::spawn(move || loop {
            let sleep_duration = {
                let cache = inner_clone.lock().unwrap();
                // Generate a random divisor in [1, 100)
                let div_factor: u32 = rand::rng().random_range(1..100);
                cache.check_interval + cache.jitter.checked_div(div_factor).unwrap_or_default()
            };
            thread::sleep(sleep_duration);
            let now = Instant::now();
            let mut cache = inner_clone.lock().unwrap();
            while let Some((_, entry)) = cache.key_value_map.front() {
                if entry.expiry < now {
                    cache.key_value_map.pop_front();
                } else {
                    break;
                }
            }
        });

        TTLCache { inner }
    }

    /// Enforce the capacity of the cache by removing the least recently accessed item if the cache is at capacity.
    fn enforce_capacity(inner: &mut TTLCacheInner<K, V>) {
        if inner.key_value_map.len() as u64 >= inner.capacity {
            if let Some(key) = inner.key_value_map.keys().next().cloned() {
                inner.key_value_map.remove(&key);
            }
        }
    }
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Send + Sync + 'static> Cache<K, V>
    for TTLCache<K, V>
{
    /// Get a value from the cache.
    fn get(&self, key: &K) -> Option<Arc<V>> {
        let now = Instant::now();
        let (result, expired) = {
            let mut inner = self.inner.lock().unwrap();
            let ttl = inner.ttl;
            if let Some(entry) = inner.key_value_map.get_refresh(key) {
                if entry.expiry > now {
                    entry.expiry = now + ttl;
                    // Clone the Arc; cloning is cheap.
                    (Some(entry.data.clone()), false)
                } else {
                    (None, true)
                }
            } else {
                (None, false)
            }
        };

        // Update stats in a separate lock block
        let mut inner = self.inner.lock().unwrap();
        if result.is_some() {
            inner.hits += 1;
        } else {
            inner.misses += 1;
            if expired {
                inner.key_value_map.remove(key);
            }
        }
        result
    }

    /// Set a value in the cache.
    fn set(&self, key: K, value: V) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        if !inner.key_value_map.contains_key(&key) {
            Self::enforce_capacity(&mut inner);
        }
        let expiry = Instant::now() + inner.ttl;
        inner
            .key_value_map
            .insert(
                key,
                DataWithLifetime {
                    data: Arc::new(value),
                    expiry,
                },
            )
            .map(|entry| entry.data)
    }

    /// Remove a value from the cache.
    fn remove(&self, key: &K) -> Option<Arc<V>> {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.remove(key).map(|entry| entry.data)
    }

    /// Clear the cache, removing all data.
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

    /// Change the capacity of the cache, if the new capacity is smaller than the current size, the oldest items are removed. Because the TTL is the same for all items this is identical as the ones which expire soonest.
    fn change_capacity(&self, capacity: u64) {
        let mut inner = self.inner.lock().unwrap();
        let old_capacity = inner.capacity;
        inner.capacity = capacity;

        while inner.key_value_map.len() as u64 > inner.capacity {
            if let Some(key) = inner.key_value_map.keys().next().cloned() {
                inner.key_value_map.remove(&key);
            }
        }

        if capacity > old_capacity {
            let additional = (capacity - old_capacity) as usize;
            inner.key_value_map.reserve(additional);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_ttl_cache() {
        let cache = TTLCache::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Duration::from_millis(10),
            2,
        );
        cache.set(1, 1);
        cache.set(2, 2);
        assert_eq!(cache.get(&1).map(|v| *v), Some(1));
        thread::sleep(Duration::from_secs(2));
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_ttl_cache_change_capacity() {
        let cache = TTLCache::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Duration::from_millis(10),
            2,
        );
        cache.set(1, 1);
        cache.set(2, 2);
        cache.change_capacity(1);
        // Depending on insertion order, one key is evicted.
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2).map(|v| *v), Some(2));
    }

    #[test]
    fn test_ttl_cache_clear() {
        let cache = TTLCache::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Duration::from_millis(10),
            2,
        );
        cache.set(1, 1);
        cache.set(2, 2);
        cache.clear();
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }
}
