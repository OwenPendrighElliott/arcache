use cachers::prelude::*;

fn main() {
    let mut cache = LRUCache::new(2);
    cache.set(1, 1);
    cache.set(2, 2);
    assert_eq!(cache.get(&1).map(|v| *v), Some(1));
    cache.set(3, 3);
    assert_eq!(cache.get(&2), None);
    cache.set(4, 4);
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&3).map(|v| *v), Some(3));
    assert_eq!(cache.get(&4).map(|v| *v), Some(4));
}
