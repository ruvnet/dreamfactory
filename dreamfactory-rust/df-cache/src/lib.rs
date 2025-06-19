use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Cache entry with TTL support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub value: serde_json::Value,
    pub expires_at: Option<SystemTime>,
}

/// Cache service trait
#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>>;
    async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<bool>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn flush(&self) -> Result<()>;
    async fn keys(&self, pattern: Option<&str>) -> Result<Vec<String>>;
}

/// In-memory cache implementation
#[derive(Debug)]
pub struct MemoryCache {
    store: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(expires_at) = entry.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheService for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let mut store = self.store.lock().unwrap();
        
        if let Some(entry) = store.get(key) {
            if self.is_expired(entry) {
                store.remove(key);
                Ok(None)
            } else {
                Ok(Some(entry.value.clone()))
            }
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<()> {
        let expires_at = ttl.map(|duration| SystemTime::now() + duration);
        let entry = CacheEntry { value, expires_at };
        
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), entry);
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut store = self.store.lock().unwrap();
        Ok(store.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut store = self.store.lock().unwrap();
        
        if let Some(entry) = store.get(key) {
            if self.is_expired(entry) {
                store.remove(key);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn flush(&self) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.clear();
        Ok(())
    }

    async fn keys(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        let mut store = self.store.lock().unwrap();
        
        // Remove expired entries first
        let now = SystemTime::now();
        store.retain(|_, entry| {
            if let Some(expires_at) = entry.expires_at {
                now <= expires_at
            } else {
                true
            }
        });
        
        let keys: Vec<String> = if let Some(pattern) = pattern {
            store.keys()
                .filter(|key| key.contains(pattern))
                .cloned()
                .collect()
        } else {
            store.keys().cloned().collect()
        };
        
        Ok(keys)
    }
}

pub fn hello() -> &'static str {
    "df-cache"
}