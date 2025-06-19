use crate::routing::{ApiRoute, ServiceHandler};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{RwLock, LazyLock};
use tracing::{info, warn, instrument};

/// Cache service handler - implements /api/v2/cache/* endpoints
pub struct CacheServiceHandler;

// In-memory cache for testing purposes
// In production, this would integrate with Redis or another cache backend
static CACHE_STORE: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

impl CacheServiceHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle GET /api/v2/cache/{key}
    #[instrument(skip(self))]
    async fn handle_get(&self, key: &str) -> Result<Value> {
        let cache = CACHE_STORE.read().unwrap();
        
        match cache.get(key) {
            Some(value) => {
                info!("Cache hit for key: {}", key);
                // Try to parse as JSON, fall back to string
                match serde_json::from_str(value) {
                    Ok(json_value) => Ok(json_value),
                    Err(_) => Ok(json!(value)),
                }
            }
            None => {
                warn!("Cache miss for key: {}", key);
                Err(anyhow::anyhow!("Key '{}' not found in cache", key))
            }
        }
    }

    /// Handle POST /api/v2/cache/{key} - Set value
    #[instrument(skip(self, body))]
    async fn handle_post(&self, key: &str, body: Option<Vec<u8>>) -> Result<Value> {
        let value = match body {
            Some(data) => String::from_utf8(data)?,
            None => return Err(anyhow::anyhow!("Request body required for cache set operation")),
        };

        let mut cache = CACHE_STORE.write().unwrap();
        cache.insert(key.to_string(), value.clone());
        
        info!("Cache set for key: {}", key);
        Ok(json!({
            "success": true,
            "key": key,
            "message": "Cache entry created successfully"
        }))
    }

    /// Handle PUT /api/v2/cache/{key} - Update value
    #[instrument(skip(self, body))]
    async fn handle_put(&self, key: &str, body: Option<Vec<u8>>) -> Result<Value> {
        // Check if key exists first
        {
            let cache = CACHE_STORE.read().unwrap();
            if !cache.contains_key(key) {
                return Err(anyhow::anyhow!("Key '{}' not found in cache", key));
            }
        }

        let value = match body {
            Some(data) => String::from_utf8(data)?,
            None => return Err(anyhow::anyhow!("Request body required for cache update operation")),
        };

        let mut cache = CACHE_STORE.write().unwrap();
        cache.insert(key.to_string(), value.clone());
        
        info!("Cache updated for key: {}", key);
        Ok(json!({
            "success": true,
            "key": key,
            "message": "Cache entry updated successfully"
        }))
    }

    /// Handle DELETE /api/v2/cache/{key}
    #[instrument(skip(self))]
    async fn handle_delete(&self, key: &str) -> Result<Value> {
        let mut cache = CACHE_STORE.write().unwrap();
        
        match cache.remove(key) {
            Some(_) => {
                info!("Cache deleted for key: {}", key);
                Ok(json!({
                    "success": true,
                    "key": key,
                    "message": "Cache entry deleted successfully"
                }))
            }
            None => {
                warn!("Attempted to delete non-existent cache key: {}", key);
                Err(anyhow::anyhow!("Key '{}' not found in cache", key))
            }
        }
    }

    /// Handle GET /api/v2/cache - List all cache keys
    #[instrument(skip(self))]
    async fn handle_list(&self) -> Result<Value> {
        let cache = CACHE_STORE.read().unwrap();
        let keys_with_sizes: Vec<(String, usize)> = cache.iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        
        info!("Retrieved {} cache keys", keys_with_sizes.len());
        Ok(json!({
            "resource": keys_with_sizes.into_iter().map(|(key, size)| json!({
                "key": key,
                "size": size
            })).collect::<Vec<_>>()
        }))
    }

    /// Handle POST /api/v2/cache/_flush - Clear all cache
    #[instrument(skip(self))]
    async fn handle_flush(&self) -> Result<Value> {
        let mut cache = CACHE_STORE.write().unwrap();
        let count = cache.len();
        cache.clear();
        
        info!("Cache flushed, {} entries removed", count);
        Ok(json!({
            "success": true,
            "message": format!("Cache flushed successfully, {} entries removed", count)
        }))
    }
}

#[async_trait::async_trait]
impl ServiceHandler for CacheServiceHandler {
    #[instrument(skip(self, _query_params, body))] 
    async fn handle_request(
        &self,
        route: &ApiRoute,
        method: &str,
        _query_params: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<Value> {
        match (&route.resource, method) {
            // Handle cache flush
            (Some(resource), "POST") if resource == "_flush" => {
                self.handle_flush().await
            }
            // Handle specific key operations
            (Some(key), "GET") => self.handle_get(key).await,
            (Some(key), "POST") => self.handle_post(key, body).await,
            (Some(key), "PUT") => self.handle_put(key, body).await,
            (Some(key), "DELETE") => self.handle_delete(key).await,
            // Handle cache listing
            (None, "GET") => self.handle_list().await,
            // Unknown operations
            (resource, method) => {
                Err(anyhow::anyhow!(
                    "Unsupported cache operation: {} {}",
                    method,
                    resource.as_deref().unwrap_or("(root)")
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::ApiRoute;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let handler = CacheServiceHandler::new();
        
        // Set a value
        let set_route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("test-key".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let set_result = handler
            .handle_request(&set_route, "POST", HashMap::new(), Some(b"test-value".to_vec()))
            .await
            .unwrap();

        assert_eq!(set_result.get("success"), Some(&json!(true)));
        assert_eq!(set_result.get("key"), Some(&json!("test-key")));

        // Get the value
        let get_result = handler
            .handle_request(&set_route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(get_result, json!("test-value"));
    }

    #[tokio::test]
    async fn test_cache_set_json_value() {
        let handler = CacheServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("json-key".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let json_data = r#"{"name": "test", "value": 42}"#;
        
        // Set JSON value
        handler
            .handle_request(&route, "POST", HashMap::new(), Some(json_data.as_bytes().to_vec()))
            .await
            .unwrap();

        // Get JSON value
        let get_result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(get_result, json!({"name": "test", "value": 42}));
    }

    #[tokio::test]
    async fn test_cache_update() {
        let handler = CacheServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("update-key".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        // Set initial value
        handler
            .handle_request(&route, "POST", HashMap::new(), Some(b"initial-value".to_vec()))
            .await
            .unwrap();

        // Update value
        let update_result = handler
            .handle_request(&route, "PUT", HashMap::new(), Some(b"updated-value".to_vec()))
            .await
            .unwrap();

        assert_eq!(update_result.get("success"), Some(&json!(true)));

        // Verify updated value
        let get_result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(get_result, json!("updated-value"));
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let handler = CacheServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("delete-key".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        // Set value
        handler
            .handle_request(&route, "POST", HashMap::new(), Some(b"delete-me".to_vec()))
            .await
            .unwrap();

        // Delete value
        let delete_result = handler
            .handle_request(&route, "DELETE", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(delete_result.get("success"), Some(&json!(true)));

        // Verify value is gone
        let get_result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await;

        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_cache_list() {
        let handler = CacheServiceHandler::new();
        
        // Set a few values
        let route1 = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("list-key-1".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let route2 = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("list-key-2".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        handler
            .handle_request(&route1, "POST", HashMap::new(), Some(b"value1".to_vec()))
            .await
            .unwrap();

        handler
            .handle_request(&route2, "POST", HashMap::new(), Some(b"value2".to_vec()))
            .await
            .unwrap();

        // List all keys
        let list_route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: None,
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let list_result = handler
            .handle_request(&list_route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        let resources = list_result.get("resource").unwrap().as_array().unwrap();
        assert!(resources.len() >= 2);
    }

    #[tokio::test]
    async fn test_cache_flush() {
        let handler = CacheServiceHandler::new();
        
        // Set some values
        let route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("flush-key".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        handler
            .handle_request(&route, "POST", HashMap::new(), Some(b"flush-me".to_vec()))
            .await
            .unwrap();

        // Flush cache
        let flush_route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("_flush".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let flush_result = handler
            .handle_request(&flush_route, "POST", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(flush_result.get("success"), Some(&json!(true)));

        // Verify cache is empty
        let get_result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await;

        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_cache_get_nonexistent() {
        let handler = CacheServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("nonexistent-key".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_update_nonexistent() {
        let handler = CacheServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "cache".to_string(),
            resource: Some("nonexistent-update-key".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "PUT", HashMap::new(), Some(b"value".to_vec()))
            .await;

        assert!(result.is_err());
    }
}