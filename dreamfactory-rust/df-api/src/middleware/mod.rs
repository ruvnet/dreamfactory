use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{info, warn};

/// Authentication middleware for API requests
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authentication token from headers
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "));

    // For now, we'll allow all requests through
    // In a real implementation, this would validate JWT tokens
    if let Some(token) = token {
        info!("Request with token: {}...", &token[..token.len().min(10)]);
    } else {
        info!("Request without authentication token");
    }

    Ok(next.run(request).await)
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client IP or identifier
    let client_id = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown");

    // For now, just log the request
    // In a real implementation, this would implement actual rate limiting
    info!("Request from client: {}", client_id);

    Ok(next.run(request).await)
}

/// Request logging middleware
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    info!("Incoming request: {} {}", method, uri);

    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();

    info!(
        "Request completed: {} {} - {} ({:?})",
        method, uri, status, duration
    );

    response
}

/// Error handling middleware
pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;
    
    // Log errors based on status code
    match response.status() {
        StatusCode::INTERNAL_SERVER_ERROR => {
            warn!("Internal server error occurred");
        }
        StatusCode::BAD_REQUEST => {
            warn!("Bad request received");
        }
        StatusCode::NOT_FOUND => {
            info!("Resource not found");
        }
        _ => {}
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
        response::Response,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "test response"
    }

    #[tokio::test]
    async fn test_auth_middleware_with_token() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(auth_middleware));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("Authorization", "Bearer test-token-123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_without_token() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(auth_middleware));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(rate_limit_middleware));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("x-forwarded-for", "192.168.1.1")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_logging_middleware() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(request_logging_middleware));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_error_handling_middleware() {
        async fn error_handler() -> StatusCode {
            StatusCode::INTERNAL_SERVER_ERROR
        }

        let app = Router::new()
            .route("/error", get(error_handler))
            .layer(middleware::from_fn(error_handling_middleware));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/error")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}