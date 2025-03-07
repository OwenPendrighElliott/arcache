pub mod cache;
pub use crate::cache::fifo::FIFOCache;
pub use crate::cache::lru::LRUCache;
pub use crate::cache::Cache;
// pub use crate::cache::lfu::LFUCache;
pub use crate::cache::mru::MRUCache;
pub use crate::cache::ttl::TTLCache;
