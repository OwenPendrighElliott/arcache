use crate::cache::{Cache, CacheStats};
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;

pub struct LRUCache<K: Eq + Hash, V> {
    capacity: u64,
    key_value_map: LinkedHashMap<K, V>,
    hits: u64,
    misses: u64,
}

impl <K: Eq + Hash, V> LRUCache<K, V> {
    pub fn new(capacity: u64) -> Self {
        let mut kv_map = LinkedHashMap::new();
        kv_map.reserve(capacity as usize);
        LRUCache {
            capacity,
            key_value_map: kv_map,
            hits: 0,
            misses: 0,
        }
    }
}


impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> for LRUCache<K, V> {
    fn get(&mut self, key: &K) -> Option<V> {
        match self.key_value_map.get_refresh(key) {
            Some(value) => {
                self.hits += 1;
                Some(value.clone())
            },
            None => {
                self.misses += 1;
                None
            }
        }
    }

    fn set(&mut self, key: &K, value: V) {
        self.key_value_map.insert(key.clone(), value);
        if self.key_value_map.len() as u64 > self.capacity {
            self.key_value_map.pop_front();
        }
    }

    fn remove(&mut self, key: &K) {
        self.key_value_map.remove(key);
    }

    fn clear(&mut self) {
        self.key_value_map.clear();
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
            self.key_value_map.pop_front();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let mut cache = LRUCache::new(2);
        cache.set(&1, 1);
        cache.set(&2, 2);
        assert_eq!(cache.get(&1), Some(1));
        cache.set(&3, 3);
        println!("{:?}", cache.key_value_map);
        assert_eq!(cache.get(&2), None);
        cache.set(&4, 4);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&3), Some(3));
        assert_eq!(cache.get(&4), Some(4));
    }
}