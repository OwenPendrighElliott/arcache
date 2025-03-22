use cachers::{Cache, LFUCache};
use std::thread;

#[derive(Debug, Clone)]
struct Product {
    _id: String,
    _name: String,
    _price: f64,
}

fn fetch_from_api(id: &str) -> Product {
    // Simulate network delay
    thread::sleep(std::time::Duration::from_millis(300));
    Product {
        _id: id.to_string(),
        _name: format!("Product {}", id),
        _price: 100.0,
    }
}

fn get_product(id: &str, cache: &LFUCache<String, Product>) -> Product {
    if let Some(cached) = cache.get(&id.to_string()) {
        return cached.as_ref().clone();
    }
    let product = fetch_from_api(id);
    cache.set(id.to_string(), product.clone());
    product
}

fn main() {
    let cache = LFUCache::<String, Product>::new(10);

    // fetch product 1 five times
    for _ in 0..5 {
        let id = "1";
        let product = get_product(&id, &cache);
        println!("Product: {:?}", product);
    }

    // fetch product 2 three times
    for _ in 0..3 {
        let id = "2";
        let product = get_product(&id, &cache);
        println!("Product: {:?}", product);
    }

    // fetch product 3 once
    let product = get_product("3", &cache);
    println!("Product: {:?}", product);

    println!("{:?}", cache.stats());
}
