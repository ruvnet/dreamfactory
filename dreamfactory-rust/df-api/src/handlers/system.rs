use crate::routing::{ApiRoute, ServiceHandler};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, instrument};

/// System service handler - implements /api/v2/system/* endpoints
pub struct SystemServiceHandler;

impl SystemServiceHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle /api/v2/system/service endpoint
    #[instrument(skip(self))]
    async fn handle_service_endpoint(&self, method: &str) -> Result<Value> {
        match method {
            "GET" => {
                info!("Retrieving system services");
                Ok(json!({
                    "resource": [
                        {
                            "name": "system",
                            "label": "System Management",  
                            "description": "System administration and configuration",
                            "type": "system",
                            "is_active": true
                        },
                        {
                            "name": "cache", 
                            "label": "Cache Service",
                            "description": "Distributed caching service",
                            "type": "cache",
                            "is_active": true
                        },
                        {
                            "name": "email",
                            "label": "Email Service", 
                            "description": "Email sending and template management",
                            "type": "email",
                            "is_active": true
                        },
                        {
                            "name": "database",
                            "label": "Database Service",
                            "description": "Database operations and management", 
                            "type": "database",
                            "is_active": true
                        },
                        {
                            "name": "files",
                            "label": "File Storage",
                            "description": "File storage and management",
                            "type": "files", 
                            "is_active": true
                        }
                    ]
                }))
            }
            _ => Err(anyhow::anyhow!("Method {} not allowed for /system/service", method)),
        }
    }

    /// Handle /api/v2/system/admin endpoint  
    #[instrument(skip(self))]
    async fn handle_admin_endpoint(&self, method: &str) -> Result<Value> {
        match method {
            "GET" => {
                info!("Retrieving system admin info");
                Ok(json!({
                    "is_hosted": false,
                    "host": {
                        "name": "DreamFactory",
                        "version": env!("CARGO_PKG_VERSION"),
                        "edition": "Open Source",
                        "is_trial": false
                    },
                    "config": {
                        "allow_guest_user": false,
                        "allow_open_registration": false,
                        "open_reg_role_id": null,
                        "open_reg_email_service_id": null,
                        "open_reg_email_template_id": null,
                        "invite_email_service_id": null,
                        "invite_email_template_id": null,
                        "password_email_service_id": null,
                        "password_email_template_id": null
                    },
                    "authentication": {
                        "default_auth_provider": "local",
                        "available_providers": ["local"]
                    }
                }))
            }
            _ => Err(anyhow::anyhow!("Method {} not allowed for /system/admin", method)),
        }
    }

    /// Handle /api/v2/system/environment endpoint
    #[instrument(skip(self))]
    async fn handle_environment_endpoint(&self, method: &str) -> Result<Value> {
        match method {
            "GET" => {
                info!("Retrieving system environment info");
                Ok(json!({
                    "platform": {
                        "name": "DreamFactory",
                        "version": env!("CARGO_PKG_VERSION"),
                        "build": "rust", 
                        "host_os": std::env::consts::OS,
                        "arch": std::env::consts::ARCH
                    },
                    "server": {
                        "software": "Axum/Tower",
                        "version": "0.7"
                    },
                    "database": {
                        "drivers": ["mysql", "postgresql", "sqlite"]
                    },
                    "cache": {
                        "drivers": ["redis", "memory"]  
                    },
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }
            _ => Err(anyhow::anyhow!("Method {} not allowed for /system/environment", method)),
        }
    }
}

impl ServiceHandler for SystemServiceHandler {
    #[instrument(skip(self, query_params, body))]
    async fn handle_request(
        &self,
        route: &ApiRoute,
        method: &str,
        _query_params: HashMap<String, String>,
        _body: Option<Vec<u8>>,
    ) -> Result<Value> {
        match route.resource.as_deref() {
            Some("service") => self.handle_service_endpoint(method).await,
            Some("admin") => self.handle_admin_endpoint(method).await,
            Some("environment") => self.handle_environment_endpoint(method).await,
            Some(resource) => Err(anyhow::anyhow!("Unknown system resource: {}", resource)),
            None => {
                // Default system endpoint - return service information
                self.handle_service_endpoint(method).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::ApiRoute;

    #[tokio::test]
    async fn test_system_service_endpoint() {
        let handler = SystemServiceHandler::new();
        let route = ApiRoute {
            version: "2".to_string(),
            service: "system".to_string(),
            resource: Some("service".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert!(result.get("resource").is_some());
        let services = result.get("resource").unwrap().as_array().unwrap();
        assert!(!services.is_empty());
        
        // Check that system service is included
        let system_service = services.iter().find(|s| s.get("name") == Some(&json!("system")));
        assert!(system_service.is_some());
    }

    #[tokio::test]
    async fn test_system_admin_endpoint() {
        let handler = SystemServiceHandler::new();
        let route = ApiRoute {
            version: "2".to_string(),
            service: "system".to_string(),
            resource: Some("admin".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert!(result.get("is_hosted").is_some());
        assert!(result.get("host").is_some());
        assert!(result.get("config").is_some());
        assert!(result.get("authentication").is_some());
    }

    #[tokio::test]
    async fn test_system_environment_endpoint() {
        let handler = SystemServiceHandler::new();
        let route = ApiRoute {
            version: "2".to_string(),
            service: "system".to_string(),
            resource: Some("environment".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert!(result.get("platform").is_some());
        assert!(result.get("server").is_some());
        assert!(result.get("database").is_some());
        assert!(result.get("cache").is_some());
        assert!(result.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_system_invalid_method() {
        let handler = SystemServiceHandler::new();
        let route = ApiRoute {
            version: "2".to_string(),
            service: "system".to_string(),
            resource: Some("service".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "POST", HashMap::new(), None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_system_unknown_resource() {
        let handler = SystemServiceHandler::new();
        let route = ApiRoute {
            version: "2".to_string(),
            service: "system".to_string(),
            resource: Some("unknown".to_string()),
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
    async fn test_system_default_endpoint() {
        let handler = SystemServiceHandler::new();
        let route = ApiRoute {
            version: "2".to_string(),
            service: "system".to_string(),
            resource: None,
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        // Should return same as /service endpoint
        assert!(result.get("resource").is_some());
    }
}