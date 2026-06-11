use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

pub struct CacheManager<K, V> {
    store: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
}

impl<K, V> Clone for CacheManager<K, V> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            default_ttl: self.default_ttl,
        }
    }
}

impl<K: Eq + std::hash::Hash, V: Clone> CacheManager<K, V> {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut expired = false;

        let result = {
            let map = self.store.read().unwrap();
            if let Some(entry) = map.get(key) {
                if Instant::now() > entry.expires_at {
                    expired = true;
                    None
                } else {
                    Some(entry.value.clone())
                }
            } else {
                None
            }
        };

        if expired {
            // Lazy eviction
            self.store.write().unwrap().remove(key);
        }

        result
    }

    pub fn set(&self, key: K, value: V) {
        self.set_with_ttl(key, value, self.default_ttl);
    }

    pub fn set_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut map = self.store.write().unwrap();
        map.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + ttl,
            },
        );
    }

    pub fn invalidate(&self, key: &K) {
        self.store.write().unwrap().remove(key);
    }

    pub fn invalidate_all(&self) {
        self.store.write().unwrap().clear();
    }
}
