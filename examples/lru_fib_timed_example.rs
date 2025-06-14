use cachers::{Cache, LRUCache};
use std::time::Instant;

// Fibonacci with LRU caching
fn lru_fib(n: u64, cache: &LRUCache<u64, u64>) -> u64 {
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

// Fibonacci without caching (naive recursion)
fn naive_fib(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    naive_fib(n - 1) + naive_fib(n - 2)
}

fn main() {
    let n = 45;

    // Measure naive Fibonacci time
    let start = Instant::now();
    let result_naive = naive_fib(n);
    let duration_naive = start.elapsed();
    println!(
        "Naive Fibonacci({}) = {} (Time: {:?})",
        n, result_naive, duration_naive
    );

    // Measure LRU cached Fibonacci time
    let cache = LRUCache::new(100);
    let start = Instant::now();
    let result_cached = lru_fib(n, &cache);
    let duration_cached = start.elapsed();
    println!(
        "Cached Fibonacci({}) = {} (Time: {:?})",
        n, result_cached, duration_cached
    );

    assert_eq!(result_naive, result_cached);
    println!("Results are equal!");

    let speedup = (duration_naive.as_secs_f64() / duration_cached.as_secs_f64()).round();
    println!("Speedup: {:.2}x", speedup);

    // Display cache stats
    println!("Cache Stats: {:?}", cache.stats());
}
