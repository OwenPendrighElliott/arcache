use cachers::{Cache, TTLCache};
use rand::rng;
use rand::{seq::SliceRandom, Rng};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct UserData {
    _id: String,
    _name: String,
    _email: String,
}

/// Simulates fetching user data from a database.
fn fetch_user_data(user_id: &str) -> UserData {
    thread::sleep(Duration::from_millis(300)); // Simulate network delay
    UserData {
        _id: user_id.to_string(),
        _name: format!("User {}", user_id),
        _email: format!("user{}@example.com", user_id),
    }
}

/// Retrieves user data using a shared TTL cache.
fn get_user_data(user_id: &str, cache: &TTLCache<String, UserData>) -> UserData {
    if let Some(cached) = cache.get(&user_id.to_string()) {
        return cached.as_ref().clone();
    }
    let user_data = fetch_user_data(user_id);
    cache.set(user_id.to_string(), user_data.clone());
    user_data
}

fn main() {
    let mut user_ids: Vec<String> = vec![];
    let num_users = 20;
    let repetitions = 4;
    let num_threads = 10;
    let cache_capacity = 20;
    let ttl_duration = Duration::from_secs(2);
    let ttl_jitter = Duration::from_millis(10);
    let background_interval = Duration::from_millis(100);

    for _ in 0..repetitions {
        for i in 0..num_users {
            user_ids.push(format!("{}", i));
        }
    }

    let mut random = rng();
    user_ids.shuffle(&mut random);

    // --- Single-threaded execution using TTLCache ---
    let ttl_cache = TTLCache::<String, UserData>::new(
        ttl_duration,
        background_interval,
        ttl_jitter,
        cache_capacity,
    );
    let start = Instant::now();
    for user_id in &user_ids {
        let data = get_user_data(user_id, &ttl_cache);
        println!("Single-threaded result: {:?}", data);
    }
    let single_duration = start.elapsed();
    println!("Single-threaded execution time: {:?}", single_duration);

    // --- Multithreaded execution ---
    let arc_cache = Arc::new(TTLCache::<String, UserData>::new(
        ttl_duration,
        background_interval,
        ttl_jitter,
        cache_capacity,
    ));
    let start = Instant::now();
    let mut handles = Vec::new();

    let chunk_size = (user_ids.len() + num_threads - 1) / num_threads;
    for chunk in user_ids.chunks(chunk_size) {
        let cache_clone = Arc::clone(&arc_cache);
        let chunk: Vec<String> = chunk.to_vec();
        let handle = thread::spawn(move || {
            let mut local_rng = rng();
            for user_id in &chunk {
                let delay = local_rng.random_range(0..=3);
                thread::sleep(Duration::from_millis(delay));
                let data = get_user_data(user_id, &cache_clone);
                println!("Multithreaded result: {:?}", data);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let multi_duration = start.elapsed();
    println!("Multithreaded execution time: {:?}", multi_duration);

    let speedup = single_duration.as_secs_f64() / multi_duration.as_secs_f64();
    println!("Multithreading speedup: {:.2}x", speedup);

    // Optionally, if your TTLCache has stats:
    println!("Cache stats: {:?}", arc_cache.stats());
}
