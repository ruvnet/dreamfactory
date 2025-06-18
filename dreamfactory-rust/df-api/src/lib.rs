pub mod routing;
pub mod handlers;
pub mod middleware;
pub mod services;

use anyhow::Result;
use axum::{
    routing::{any, get},
    Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::{
    handlers::{
        universal_api_handler, health_check, api_info,
        system::SystemServiceHandler,
        cache::CacheServiceHandler,
        email::EmailServiceHandler,
    },
    routing::ServiceRegistry,
};

/// Create the main API router with all services registered
pub fn create_api_router() -> Result<Router> {
    info!("Creating API router with service registry");

    // Create service registry
    let mut registry = ServiceRegistry::new();
    
    // Register services
    registry.register("system".to_string(), SystemServiceHandler::new());
    registry.register("cache".to_string(), CacheServiceHandler::new());
    registry.register("email".to_string(), EmailServiceHandler::new());
    
    let registry_arc = Arc::new(registry);

    // Create router
    let router = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        .route("/", get(api_info))
        
        // Universal API route handler - matches /api/v{version}/{service}/*
        .route("/api/*path", any(universal_api_handler))
        
        // Add the service registry to state
        .with_state(registry_arc)
        
        // Add middleware
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        )
        .layer(TraceLayer::new_for_http());

    info!("API router created successfully");
    Ok(router)
}

/// Configuration for the API service
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub log_level: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            log_level: "info".to_string(),
        }
    }
}

/// Initialize tracing for the API service
pub fn init_tracing(level: &str) -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("df_api={},tower_http=debug,axum::rejection=trace", level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;

    #[tokio::test]
    async fn test_api_router_creation() {
        let router = create_api_router().unwrap();
        assert!(router.into_make_service().is_ok());
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let response = server.get("/health").await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert_eq!(json.get("status"), Some(&json!("ok")));
        assert!(json.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_api_info_endpoint() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let response = server.get("/").await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert_eq!(json.get("name"), Some(&json!("DreamFactory API")));
        assert!(json.get("version").is_some());
    }

    #[tokio::test]
    async fn test_system_service_endpoint() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let response = server.get("/api/v2/system/service").await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert!(json.get("resource").is_some());
        
        let services = json.get("resource").unwrap().as_array().unwrap();
        assert!(!services.is_empty());
    }

    #[tokio::test]
    async fn test_system_admin_endpoint() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let response = server.get("/api/v2/system/admin").await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert!(json.get("is_hosted").is_some());
        assert!(json.get("host").is_some());
        assert!(json.get("config").is_some());
    }

    #[tokio::test]
    async fn test_cache_endpoints() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        // Set a cache value
        let response = server
            .post("/api/v2/cache/test-key")
            .text("test-value")
            .await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert_eq!(json.get("success"), Some(&json!(true)));
        
        // Get the cache value
        let response = server.get("/api/v2/cache/test-key").await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert_eq!(json, json!("test-value"));
        
        // Delete the cache value
        let response = server.delete("/api/v2/cache/test-key").await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert_eq!(json.get("success"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn test_email_send_endpoint() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let email_data = json!({
            "to": ["test@example.com"],
            "subject": "Test Email",
            "body_text": "This is a test email"
        });
        
        let response = server
            .post("/api/v2/email/_send")
            .json(&email_data)
            .await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert_eq!(json.get("success"), Some(&json!(true)));
        assert!(json.get("message_id").is_some());
    }

    #[tokio::test]
    async fn test_email_templates_endpoint() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let response = server.get("/api/v2/email/template").await;
        response.assert_status_ok();
        
        let json = response.json::<serde_json::Value>();
        assert!(json.get("resource").is_some());
        
        let templates = json.get("resource").unwrap().as_array().unwrap();
        assert_eq!(templates.len(), 2);
    }

    #[tokio::test]
    async fn test_invalid_api_route() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let response = server.get("/api/invalid").await;
        response.assert_status_bad_request();
        
        let json = response.json::<serde_json::Value>();
        assert!(json.get("error").is_some());
    }

    #[tokio::test]
    async fn test_nonexistent_service() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();
        
        let response = server.get("/api/v2/nonexistent/resource").await;
        response.assert_status_not_found();
        
        let json = response.json::<serde_json::Value>();
        assert!(json.get("error").is_some());
    }

    #[tokio::test]
    async fn test_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
        assert_eq!(config.log_level, "info");
    }
}