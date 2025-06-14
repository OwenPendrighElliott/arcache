use arcache::{Cache, LRUCache};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lru_cache(c: &mut Criterion) {
    c.bench_function("lru_set_1k", |b| {
        b.iter(|| {
            let cache = LRUCache::new(100);
            for i in 0..1000 {
                cache.set(i, black_box(i + 1));
            }
        })
    });

    c.bench_function("lru_ge_1k", |b| {
        let cache = LRUCache::new(100);
        for i in 0..1000 {
            cache.set(i, i + 1);
        }
        b.iter(|| {
            for i in 0..1000 {
                black_box(cache.get(&i));
            }
        })
    });

    c.bench_function("lru_evict_1k", |b| {
        b.iter(|| {
            let cache = LRUCache::new(50);
            for i in 0..1000 {
                cache.set(i, black_box(i));
            }
        })
    });
}

criterion_group!(benches, bench_lru_cache);
criterion_main!(benches);
