use cachers::{Cache, FIFOCache, LFUCache, LIFOCache, LRUCache, MRUCache, RandomReplacementCache};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_all(c: &mut Criterion) {
    // A list of (label, factory) pairs, where 'factory' creates a fresh cache each time.
    let cache_factories: Vec<(&'static str, Box<dyn Fn() -> Box<dyn Cache<i32, i32>>>)> = vec![
        ("LRU", Box::new(|| Box::new(LRUCache::new(100)))),
        ("MRU", Box::new(|| Box::new(MRUCache::new(100)))),
        ("FIFO", Box::new(|| Box::new(FIFOCache::new(100)))),
        ("LIFO", Box::new(|| Box::new(LIFOCache::new(100)))),
        ("LFU", Box::new(|| Box::new(LFUCache::new(100)))),
        (
            "RANDOM",
            Box::new(|| Box::new(RandomReplacementCache::new(100))),
        ),
        (
            "TTL",
            Box::new(|| {
                Box::new(cachers::TTLCache::<i32, i32>::new(
                    std::time::Duration::from_secs(1),
                    100,
                ))
            }),
        ),
    ];

    for (label, factory) in cache_factories {
        // Benchmark "set" operations
        c.bench_function(&format!("{}_set", label), |b| {
            b.iter(|| {
                let cache = factory();
                for i in 0..100 {
                    cache.set(i, black_box(i + 1));
                }
            })
        });

        // Benchmark "get" operations
        c.bench_function(&format!("{}_get", label), |b| {
            // Pre-fill the cache before timing gets
            let cache = factory();
            for i in 0..100 {
                cache.set(i, i + 1);
            }

            b.iter(|| {
                for i in 0..100 {
                    black_box(cache.get(&i));
                }
            })
        });

        // Benchmark how it handles evictions
        c.bench_function(&format!("{}_evict", label), |b| {
            b.iter(|| {
                let cache = factory();
                // Fill up to capacity
                for i in 0..100 {
                    cache.set(i, black_box(i));
                }
                // Add more to force evictions
                for i in 100..200 {
                    cache.set(i, black_box(i));
                }
            })
        });
    }
}

criterion_group!(benches, bench_all);
criterion_main!(benches);
