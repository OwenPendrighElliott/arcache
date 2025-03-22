# cache.rs

A crate which implements a variety of caches with different eviction policies. All cache implementations are thread-safe and can be used in a multi-threaded environment. Cache implementations all share the `Cache` trait which means that they are completely interchangeable.

```rust
use cachers::{Cache, LRUCache};

fn main() {
    let cache = LRUCache::<&str, String>::new(10); // mutability is internally handled so you can use `let` instead of `let mut`
    
    // like std::collections::HashMap, you can use the `set` returns the previous value if it exists
    let original_value = cache.set("key", "value".to_string());

    assert!(original_value.is_none());
    
    // get returns an Option<Arc<V>> where V is the value type
    let value = cache.get(&"key");

    assert!(value.is_some());
    assert_eq!(*value.unwrap(), "value".to_string()); // value is wrapped in an Arc so you need to dereference it
    println!("{:?}", cache.stats());
}
```

## Implemented caches

+ `LRUCache`
+ `LFUCache`
+ `MRUCache`
+ `TTLCache`
+ `FIFOCache`

### Yet to be added

+ `ARC`

## Usage

See `/examples` for example usage. You can run these like so:

```bash
cargo run --example lru_example --release
cargo run --example lfu_example --release
```

etc.