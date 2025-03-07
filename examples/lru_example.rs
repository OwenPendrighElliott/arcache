use cachers::{Cache, LRUCache};

fn lru_fib(n: u64, cache: &mut LRUCache<u64, u64>) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    match cache.get(&n) {
        Some(v) => *v,
        None => {
            let result = lru_fib(n - 1, cache) + lru_fib(n - 2, cache);
            cache.set(n, result);
            result
        }
    }
}

fn main() {
    let mut cache = LRUCache::new(10);
    println!("{}", lru_fib(10, &mut cache));
    println!("{:?}", cache.stats());
    println!("{}", lru_fib(10, &mut cache));
    println!("{:?}", cache.stats());
    println!("{}", lru_fib(20, &mut cache));
    println!("{:?}", cache.stats());
}
