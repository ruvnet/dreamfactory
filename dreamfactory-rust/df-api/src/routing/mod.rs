use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Universal API route pattern: /api/v{version}/{service}/{resource}
#[derive(Debug, Clone, PartialEq)]
pub struct ApiRoute {
    pub version: String,
    pub service: String,
    pub resource: Option<String>,
    pub id: Option<String>,
    pub sub_resource: Option<String>,
    pub sub_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    pub version: String,
    pub service: String,
    pub resource: Option<String>,
    pub id: Option<String>,
    pub sub_resource: Option<String>,
    pub sub_id: Option<String>,
}

/// Service registry for handling different service types
#[async_trait]
pub trait ServiceHandler: Send + Sync {
    async fn handle_request(
        &self,
        route: &ApiRoute,
        method: &str,
        query_params: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<serde_json::Value>;
}

pub struct ServiceRegistry {
    handlers: HashMap<String, Arc<dyn ServiceHandler>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, service_name: String, handler: T)
    where
        T: ServiceHandler + 'static,
    {
        self.handlers.insert(service_name, Arc::new(handler));
    }

    pub fn get_handler(&self, service_name: &str) -> Option<Arc<dyn ServiceHandler>> {
        self.handlers.get(service_name).cloned()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse API route from path
pub fn parse_api_route(path: &str) -> Result<ApiRoute> {
    // Universal pattern: /api/v{version}/{service}[/{resource}[/{id}[/{sub_resource}[/{sub_id}]]]]
    let re = Regex::new(r"^/api/v([^/]+)/([^/]+)(?:/([^/]+))?(?:/([^/]+))?(?:/([^/]+))?(?:/([^/]+))?")
        .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

    if let Some(captures) = re.captures(path) {
        let version = captures.get(1)
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| anyhow::anyhow!("Missing version"))?;

        let service = captures.get(2)
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| anyhow::anyhow!("Missing service"))?;

        let resource = captures.get(3).map(|m| m.as_str().to_string());
        let id = captures.get(4).map(|m| m.as_str().to_string());
        let sub_resource = captures.get(5).map(|m| m.as_str().to_string());
        let sub_id = captures.get(6).map(|m| m.as_str().to_string());

        Ok(ApiRoute {
            version,
            service,
            resource,
            id,
            sub_resource,
            sub_id,
        })
    } else {
        Err(anyhow::anyhow!("Invalid API route format: {}", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_service_route() {
        let result = parse_api_route("/api/v2/system").unwrap();
        assert_eq!(result.version, "2");
        assert_eq!(result.service, "system");
        assert_eq!(result.resource, None);
        assert_eq!(result.id, None);
        assert_eq!(result.sub_resource, None); 
        assert_eq!(result.sub_id, None);
    }

    #[test]
    fn test_parse_service_with_resource() {
        let result = parse_api_route("/api/v2/system/service").unwrap();
        assert_eq!(result.version, "2");
        assert_eq!(result.service, "system");
        assert_eq!(result.resource, Some("service".to_string()));
        assert_eq!(result.id, None);
        assert_eq!(result.sub_resource, None);
        assert_eq!(result.sub_id, None);
    }

    #[test]
    fn test_parse_service_with_resource_and_id() {
        let result = parse_api_route("/api/v2/cache/my-key").unwrap();
        assert_eq!(result.version, "2");
        assert_eq!(result.service, "cache");
        assert_eq!(result.resource, Some("my-key".to_string()));
        assert_eq!(result.id, None);
        assert_eq!(result.sub_resource, None);
        assert_eq!(result.sub_id, None);
    }

    #[test]
    fn test_parse_email_send_endpoint() {
        let result = parse_api_route("/api/v2/email/_send").unwrap();
        assert_eq!(result.version, "2");
        assert_eq!(result.service, "email");
        assert_eq!(result.resource, Some("_send".to_string()));
        assert_eq!(result.id, None);
        assert_eq!(result.sub_resource, None);
        assert_eq!(result.sub_id, None);
    }

    #[test]
    fn test_parse_nested_resource() {
        let result = parse_api_route("/api/v2/database/table/users/1").unwrap();
        assert_eq!(result.version, "2");
        assert_eq!(result.service, "database");
        assert_eq!(result.resource, Some("table".to_string()));
        assert_eq!(result.id, Some("users".to_string()));
        assert_eq!(result.sub_resource, Some("1".to_string()));
        assert_eq!(result.sub_id, None);
    }

    #[test]
    fn test_parse_invalid_route() {
        let result = parse_api_route("/invalid/route");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_service() {
        let result = parse_api_route("/api/v2/");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_flexibility() {
        let result = parse_api_route("/api/v1.5/system").unwrap();
        assert_eq!(result.version, "1.5");
        assert_eq!(result.service, "system");
    }

    #[test]
    fn test_service_registry() {
        let mut registry = ServiceRegistry::new();
        
        struct TestHandler;
        #[async_trait]
        impl ServiceHandler for TestHandler {
            async fn handle_request(
                &self,
                _route: &ApiRoute,
                _method: &str,
                _query_params: HashMap<String, String>,
                _body: Option<Vec<u8>>,
            ) -> Result<serde_json::Value> {
                Ok(serde_json::json!({"status": "ok"}))
            }
        }

        registry.register("test".to_string(), TestHandler);
        assert!(registry.get_handler("test").is_some());
        assert!(registry.get_handler("nonexistent").is_none());
    }
}