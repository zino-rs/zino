//! Global cache for the application.

use crate::{state::State, JsonValue};
use lru::LruCache;
use parking_lot::RwLock;
use std::{num::NonZeroUsize, sync::LazyLock};

/// Global cache built on the top of [`LruCache`].
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalCache;

impl GlobalCache {
    /// Puts a key-value pair into the global cache.
    /// If the key already exists in the cache, then it updates the key’s value and
    /// returns the old value. Otherwise, `None` is returned.
    #[inline]
    pub fn put(key: impl Into<String>, value: impl Into<JsonValue>) -> Option<JsonValue> {
        let mut cache = GLOBAL_CACHE.write();
        cache.put(key.into(), value.into())
    }

    /// Pushes a key-value pair into the global cache. If an entry with the key already
    /// exists in the cache or another cache entry is removed (due to the LRU’s capacity),
    /// then it returns the old entry’s key-value pair. Otherwise, returns `None`.
    #[inline]
    pub fn push(
        key: impl Into<String>,
        value: impl Into<JsonValue>,
    ) -> Option<(String, JsonValue)> {
        let mut cache = GLOBAL_CACHE.write();
        cache.push(key.into(), value.into())
    }

    /// Returns a cloned value of the key in the global cache or `None`
    /// if it is not present in the cache. Moves the key to the head of the LRU list if it exists.
    #[inline]
    pub fn get(key: &str) -> Option<JsonValue> {
        let mut cache = GLOBAL_CACHE.write();
        cache.get(key).cloned()
    }

    /// Returns a cloned value of the key in the global cache or `None`
    /// if it is not present in the cache. It does not update the LRU list
    /// so the key’s position will be unchanged.
    #[inline]
    pub fn peek(key: &str) -> Option<JsonValue> {
        let cache = GLOBAL_CACHE.read();
        cache.peek(key).cloned()
    }

    /// Returns a bool indicating whether the given key is in the global cache.
    /// Does not update the LRU list.
    #[inline]
    pub fn contains(key: &str) -> bool {
        let cache = GLOBAL_CACHE.read();
        cache.contains(key)
    }

    /// Removes and returns the value corresponding to the key from the global cache or
    /// `None` if it does not exist.
    #[inline]
    pub fn pop(key: &str) -> Option<JsonValue> {
        let mut cache = GLOBAL_CACHE.write();
        cache.pop(key)
    }

    /// Removes and returns the key-value pair from the global cache or
    /// `None` if it does not exist.
    #[inline]
    pub fn pop_entry(key: &str) -> Option<(String, JsonValue)> {
        let mut cache = GLOBAL_CACHE.write();
        cache.pop_entry(key)
    }

    /// Removes and returns the key-value pair corresponding to the least recently used item
    /// or `None` if the global cache is empty.
    #[inline]
    pub fn pop_lru() -> Option<(String, JsonValue)> {
        let mut cache = GLOBAL_CACHE.write();
        cache.pop_lru()
    }

    /// Marks the key as the most recently used one.
    #[inline]
    pub fn promote(key: &str) {
        let mut cache = GLOBAL_CACHE.write();
        cache.promote(key)
    }

    /// Marks the key as the least recently used one.
    #[inline]
    pub fn demote(key: &str) {
        let mut cache = GLOBAL_CACHE.write();
        cache.demote(key)
    }

    /// Returns the number of key-value pairs that are currently in the global cache.
    #[inline]
    pub fn len() -> usize {
        let cache = GLOBAL_CACHE.read();
        cache.len()
    }

    /// Returns a bool indicating whether the global cache is empty or not.
    #[inline]
    pub fn is_empty() -> bool {
        let cache = GLOBAL_CACHE.read();
        cache.is_empty()
    }

    /// Returns the maximum number of key-value pairs the global cache can hold.
    #[inline]
    pub fn cap() -> NonZeroUsize {
        let cache = GLOBAL_CACHE.read();
        cache.cap()
    }

    /// Resizes the global cache. If the new capacity is smaller than the size of
    /// the current cache any entries past the new capacity are discarded.
    #[inline]
    pub fn resize(cap: NonZeroUsize) {
        let mut cache = GLOBAL_CACHE.write();
        cache.resize(cap)
    }

    /// Clears the contents of the global cache.
    pub fn clear() {
        let mut cache = GLOBAL_CACHE.write();
        cache.clear()
    }
}

/// Global cache.
static GLOBAL_CACHE: LazyLock<RwLock<LruCache<String, JsonValue>>> = LazyLock::new(|| {
    let capacity = if let Some(cache) = State::shared().get_config("cache") {
        cache
            .get("capacity")
            .expect("the `capacity` field is missing")
            .as_integer()
            .expect("the `capacity` field should be an integer")
            .try_into()
            .expect("the `capacity` field should be a positive integer")
    } else {
        10000
    };
    RwLock::new(LruCache::new(
        NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::MIN),
    ))
});
