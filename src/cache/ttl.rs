use std::hash::Hash;
use linked_hash_map::LinkedHashMap;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::cache::{Cache, CacheStats};

#[derive(Clone)]
struct DataWithLifetime<V> {
    data: V,
    expiry: Instant,
}

struct TTLCacheInner<K, V> {
    ttl: Duration,
    jitter: Duration,
    check_interval: Duration,
    capacity: u64,
    key_value_map: LinkedHashMap<K, DataWithLifetime<V>>,
    hits: u64,
    misses: u64,
}

pub struct TTLCache<K: Eq + Hash + Clone + Send + 'static, V: Clone + Send + 'static> {
    inner: Arc<Mutex<TTLCacheInner<K, V>>>,
}

impl<K: Eq + Hash + Clone + Send + 'static, V: Clone + Send + 'static> TTLCache<K, V> {
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

        // thread for background evictions of expired items
        thread::spawn(move || loop {
            let sleep_duration = {
                let cache = inner_clone.lock().unwrap();
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

    fn enforce_capacity(inner: &mut TTLCacheInner<K, V>) {
        if inner.key_value_map.len() as u64 >= inner.capacity {
            if let Some(key) = inner.key_value_map.keys().next().cloned() {
                inner.key_value_map.remove(&key);
            }
        }
    }
}

impl<K: Eq + Hash + Clone + Send + 'static, V: Clone + Send + 'static> Cache<K, V> for TTLCache<K, V> {
    fn get(&mut self, key: &K) -> Option<V> {
        let now = Instant::now();
        let (result, expired) = {
            let mut inner = self.inner.lock().unwrap();
            let ttl = inner.ttl;
            if let Some(entry) = inner.key_value_map.get_refresh(key) {
                if entry.expiry > now {
                    entry.expiry = now + ttl;
                    (Some(entry.data.clone()), false)
                } else {
                    (None, true)
                }
            } else {
                (None, false)
            }
        };

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

    fn set(&mut self, key: &K, value: V) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.key_value_map.contains_key(key) {
            Self::enforce_capacity(&mut inner);
        }
        let expiry = Instant::now() + inner.ttl;
        inner.key_value_map.insert(
            key.clone(),
            DataWithLifetime {
                data: value,
                expiry,
            },
        );
    }

    fn remove(&mut self, key: &K) {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.remove(key);
    }

    fn clear(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.key_value_map.clear();
    }

    fn stats(&self) -> CacheStats {
        let inner = self.inner.lock().unwrap();
        CacheStats {
            hits: inner.hits,
            misses: inner.misses,
            size: inner.key_value_map.len() as u64,
            capacity: inner.capacity,
        }
    }

    fn change_capacity(&mut self, capacity: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.capacity = capacity;
        while inner.key_value_map.len() as u64 > inner.capacity {
            if let Some(key) = inner.key_value_map.keys().next().cloned() {
                inner.key_value_map.remove(&key);
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_cache() {
        let mut cache = TTLCache::new(Duration::from_secs(1), Duration::from_millis(100), Duration::from_millis(10), 2);
        cache.set(&1, 1);
        cache.set(&2, 2);
        assert_eq!(cache.get(&1), Some(1));
        thread::sleep(Duration::from_secs(2));
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }
}