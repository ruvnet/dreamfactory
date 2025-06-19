use crate::{AuthError, Result, AuthContext, AuthServiceState};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(auth_service): State<AuthServiceState>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Extract token from Authorization header
    let token = match extract_bearer_token(headers) {
        Ok(token) => token,
        Err(_) => {
            // Try API key authentication as fallback
            if let Ok(api_key) = extract_api_key(headers) {
                return handle_api_key_auth(auth_service, api_key, request, next).await;
            }
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Validate session token
    match auth_service.validate_session(&token).await {
        Ok(auth_context) => {
            // Add auth context to request extensions
            request.extensions_mut().insert(auth_context);
            Ok(next.run(request).await)
        }
        Err(AuthError::TokenExpired) => Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::InvalidToken) => Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::SessionNotFound) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn optional_auth_middleware(
    State(auth_service): State<AuthServiceState>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    
    // Try to extract and validate token, but don't fail if not present
    if let Ok(token) = extract_bearer_token(headers) {
        if let Ok(auth_context) = auth_service.validate_session(&token).await {
            request.extensions_mut().insert(auth_context);
        }
    } else if let Ok(api_key) = extract_api_key(headers) {
        // Try API key authentication
        if let Ok(auth_context) = validate_api_key(&auth_service, &api_key, get_client_ip(headers).as_deref()).await {
            request.extensions_mut().insert(auth_context);
        }
    }

    next.run(request).await
}

async fn handle_api_key_auth(
    auth_service: AuthServiceState,
    api_key: String,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    let ip_address = get_client_ip(request.headers());
    
    match validate_api_key(&auth_service, &api_key, ip_address.as_deref()).await {
        Ok(auth_context) => {
            request.extensions_mut().insert(auth_context);
            Ok(next.run(request).await)
        }
        Err(AuthError::ApiKeyNotFound) => Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::ApiKeyExpired) => Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::Forbidden) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn validate_api_key(
    _auth_service: &AuthServiceState,
    _api_key: &str,
    _ip_address: Option<&str>,
) -> Result<AuthContext> {
    // This would need access to the API key service
    // For now, we'll create a simplified version
    // In a real implementation, you'd inject the API key service
    Err(AuthError::ApiKeyNotFound) // Placeholder
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::Unauthorized);
    }

    Ok(auth_header.strip_prefix("Bearer ").unwrap().to_string())
}

fn extract_api_key(headers: &HeaderMap) -> Result<String> {
    // Try X-DreamFactory-API-Key header first (DreamFactory standard)
    if let Some(api_key) = headers.get("x-dreamfactory-api-key").and_then(|h| h.to_str().ok()) {
        return Ok(api_key.to_string());
    }

    // Try X-API-Key header as fallback
    if let Some(api_key) = headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
        return Ok(api_key.to_string());
    }

    // Try query parameter (less secure, but sometimes needed)
    // This would need to be handled in the route handlers since middleware doesn't have access to query params

    Err(AuthError::Unauthorized)
}

fn get_client_ip(headers: &HeaderMap) -> Option<String> {
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "x-client-ip",
        "cf-connecting-ip",
    ];

    for header_name in &ip_headers {
        if let Some(value) = headers.get(*header_name).and_then(|h| h.to_str().ok()) {
            let ip = value.split(',').next().unwrap_or(value).trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, HeaderMap};

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer test_token"));

        let token = extract_bearer_token(&headers).unwrap();
        assert_eq!(token, "test_token");
    }

    #[test]
    fn test_extract_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert("x-dreamfactory-api-key", HeaderValue::from_static("test_api_key"));

        let api_key = extract_api_key(&headers).unwrap();
        assert_eq!(api_key, "test_api_key");
    }

    #[test]
    fn test_extract_api_key_fallback() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("fallback_key"));

        let api_key = extract_api_key(&headers).unwrap();
        assert_eq!(api_key, "fallback_key");
    }
}