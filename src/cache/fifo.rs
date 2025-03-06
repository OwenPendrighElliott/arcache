use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use crate::cache::{Cache, CacheStats};

pub struct FIFOCache<K: Eq + Hash, V> {
    capacity: u64,
    key_value_map: HashMap<K, V>,
    fifo: VecDeque<K>,
    hits: u64,
    misses: u64,
}

impl<K: Eq + Hash, V> FIFOCache<K, V> {
    pub fn new(capacity: u64) -> Self {
        FIFOCache {
            capacity,
            key_value_map: HashMap::with_capacity(capacity as usize),
            fifo: VecDeque::with_capacity(capacity as usize),
            hits: 0,
            misses: 0,
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> for FIFOCache<K, V> {
    fn get(&mut self, key: &K) -> Option<V> {
        match self.key_value_map.get(key) {
            Some(value) => {
                self.hits += 1;
                Some(value.clone())
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    fn set(&mut self, key: &K, value: V) {
        if self.key_value_map.len() as u64 >= self.capacity {
            if let Some(oldest_key) = self.fifo.pop_front() {
                self.key_value_map.remove(&oldest_key);
            }
        }
        self.key_value_map.insert(key.clone(), value);
        self.fifo.push_back(key.clone());
    }

    fn remove(&mut self, key: &K) {
        self.key_value_map.remove(key);
    }

    fn clear(&mut self) {
        self.key_value_map.clear();
        self.fifo.clear();
    }

    fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.key_value_map.len() as u64,
            capacity: self.capacity,
        }
    }

    fn change_capacity(&mut self, capacity: u64) {
        self.capacity = capacity;
        while self.key_value_map.len() as u64 > self.capacity {
            if let Some(oldest_key) = self.fifo.pop_front() {
                self.key_value_map.remove(&oldest_key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_cache() {
        let mut cache = FIFOCache::new(2);
        cache.set(&1, 1);
        cache.set(&2, 2);
        assert_eq!(cache.get(&1), Some(1));
        cache.set(&3, 3);
        assert_eq!(cache.get(&2), Some(2));
        assert_eq!(cache.get(&1), None);
        cache.set(&4, 4);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&3), Some(3));
        assert_eq!(cache.get(&4), Some(4));
    }

    #[test]
    fn test_fifo_cache_clear() {
        let mut cache = FIFOCache::new(2);
        cache.set(&1, 1);
        cache.set(&2, 2);
        cache.clear();
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_fifo_cache_change_capacity() {
        let mut cache = FIFOCache::new(2);
        cache.set(&1, 1);
        cache.set(&2, 2);
        cache.change_capacity(1);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(2));
    }
}
