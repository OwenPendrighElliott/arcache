pub mod cache;
pub use crate::cache::fifo::FIFOCache;
pub use crate::cache::lfu::LFUCache;
pub use crate::cache::lifo::LIFOCache;
pub use crate::cache::lru::LRUCache;
pub use crate::cache::mru::MRUCache;
pub use crate::cache::random_replacement::RandomReplacementCache;
pub use crate::cache::ttl::TTLCache;
pub use crate::cache::Cache;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct _ReadmeDoctests;
