use crate::routing::{parse_api_route, ServiceRegistry};
use axum::{
    extract::{Query, State},
    http::{Method, StatusCode},
    response::Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument};

pub mod system;
pub mod cache;
pub mod email;

/// Main API handler that processes all /api/v* requests
#[instrument(skip(registry, body))]
pub async fn universal_api_handler(
    method: Method,
    uri: axum::http::Uri,
    Query(query_params): Query<HashMap<String, String>>,
    State(registry): State<Arc<ServiceRegistry>>,
    body: Option<axum::body::Bytes>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let path = uri.path();
    
    info!("Processing API request: {} {}", method, path);

    // Parse the API route
    let route = match parse_api_route(path) {
        Ok(route) => route,
        Err(e) => {
            error!("Failed to parse API route '{}': {}", path, e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": {
                        "code": 400,
                        "message": "Invalid API route format",
                        "details": e.to_string()
                    }
                }))
            ));
        }
    };

    // Get the service handler
    let handler = match registry.get_handler(&route.service) {
        Some(handler) => handler,
        None => {
            error!("Service '{}' not found", route.service);
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": {
                        "code": 404,
                        "message": format!("Service '{}' not found", route.service)
                    }
                }))
            ));
        }
    };

    // Convert body to bytes if present
    let body_bytes = body.map(|b| b.to_vec());
    let method_str = method.as_str();

    // Handle the request
    match handler.handle_request(&route, method_str, query_params, body_bytes).await {
        Ok(response) => {
            info!("API request processed successfully");
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to process API request: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "code": 500,
                        "message": "Internal server error",
                        "details": e.to_string()
                    }
                }))
            ))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "dreamfactory-api"
    }))
}

/// API info endpoint
pub async fn api_info() -> Json<Value> {
    Json(json!({
        "name": "DreamFactory API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Universal REST API framework",
        "documentation": "https://wiki.dreamfactory.com/DreamFactory/API",
        "supported_versions": ["v1", "v2"]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::{ApiRoute, ServiceHandler, ServiceRegistry};
    use axum::body::Bytes;
    use axum::http::Uri;
    use std::collections::HashMap;
    use tokio_test;

    struct MockServiceHandler {
        response: Value,
    }

    impl ServiceHandler for MockServiceHandler {
        async fn handle_request(
            &self,
            _route: &ApiRoute,
            _method: &str,
            _query_params: HashMap<String, String>,
            _body: Option<Vec<u8>>,
        ) -> anyhow::Result<Value> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_universal_api_handler_success() {
        let mut registry = ServiceRegistry::new();
        registry.register(
            "test".to_string(),
            MockServiceHandler {
                response: json!({"result": "success"}),
            },
        );

        let uri: Uri = "/api/v2/test/resource".parse().unwrap();
        let query_params = HashMap::new();
        let registry_arc = Arc::new(registry);

        let result = universal_api_handler(
            Method::GET,
            uri,
            Query(query_params),
            State(registry_arc),
            None,
        ).await;

        assert!(result.is_ok());
        let Json(response) = result.unwrap();
        assert_eq!(response, json!({"result": "success"}));
    }

    #[tokio::test]
    async fn test_universal_api_handler_invalid_route() {
        let registry = Arc::new(ServiceRegistry::new());
        let uri: Uri = "/invalid/route".parse().unwrap();
        let query_params = HashMap::new();

        let result = universal_api_handler(
            Method::GET,
            uri,
            Query(query_params),
            State(registry),
            None,
        ).await;

        assert!(result.is_err());
        let (status, Json(error)) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(error.get("error").is_some());
    }

    #[tokio::test]
    async fn test_universal_api_handler_service_not_found() {
        let registry = Arc::new(ServiceRegistry::new());
        let uri: Uri = "/api/v2/nonexistent/resource".parse().unwrap();
        let query_params = HashMap::new();

        let result = universal_api_handler(
            Method::GET,
            uri,
            Query(query_params),
            State(registry),
            None,
        ).await;

        assert!(result.is_err());
        let (status, Json(error)) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(error.get("error").is_some());
    }

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        let Json(response) = result;
        assert_eq!(response.get("status"), Some(&json!("ok")));
        assert!(response.get("timestamp").is_some());
        assert_eq!(response.get("service"), Some(&json!("dreamfactory-api")));
    }

    #[tokio::test]
    async fn test_api_info() {
        let result = api_info().await;
        let Json(response) = result;
        assert_eq!(response.get("name"), Some(&json!("DreamFactory API")));
        assert!(response.get("version").is_some());
        assert!(response.get("supported_versions").is_some());
    }
}