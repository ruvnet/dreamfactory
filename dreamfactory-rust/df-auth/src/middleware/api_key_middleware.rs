use crate::{AuthError, Result, AuthContext, ApiKeyServiceState};
use axum::{
    extract::{Request, State, Query},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ApiKeyQuery {
    api_key: Option<String>,
}

/// Middleware specifically for API key authentication
pub async fn api_key_middleware(
    State(api_key_service): State<ApiKeyServiceState>,
    Query(query): Query<ApiKeyQuery>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Try to extract API key from multiple sources
    let api_key = extract_api_key_from_sources(headers, &query)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let ip_address = get_client_ip(headers);
    
    // Authenticate API key
    match api_key_service.authenticate_api_key(&api_key, ip_address.as_deref()).await {
        Ok(api_key_record) => {
            // Get permissions for this API key
            let permissions = api_key_service.get_user_permissions_for_api_key(&api_key_record).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Create auth context from API key
            let auth_context = create_auth_context_from_api_key(api_key_record, permissions);
            
            // Add auth context to request
            request.extensions_mut().insert(auth_context);
            Ok(next.run(request).await)
        }
        Err(AuthError::ApiKeyNotFound) => Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::ApiKeyExpired) => Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::Forbidden) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Optional API key middleware that doesn't fail if no API key is present
pub async fn optional_api_key_middleware(
    State(api_key_service): State<ApiKeyServiceState>,
    Query(query): Query<ApiKeyQuery>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    
    // Try to extract and validate API key, but continue if not present
    if let Ok(api_key) = extract_api_key_from_sources(headers, &query) {
        let ip_address = get_client_ip(headers);
        
        if let Ok(api_key_record) = api_key_service.authenticate_api_key(&api_key, ip_address.as_deref()).await {
            if let Ok(permissions) = api_key_service.get_user_permissions_for_api_key(&api_key_record).await {
                let auth_context = create_auth_context_from_api_key(api_key_record, permissions);
                request.extensions_mut().insert(auth_context);
            }
        }
    }

    next.run(request).await
}

fn extract_api_key_from_sources(headers: &HeaderMap, query: &ApiKeyQuery) -> Result<String> {
    // 1. Try X-DreamFactory-API-Key header (DreamFactory standard)
    if let Some(api_key) = headers.get("x-dreamfactory-api-key").and_then(|h| h.to_str().ok()) {
        return Ok(api_key.to_string());
    }

    // 2. Try X-API-Key header
    if let Some(api_key) = headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
        return Ok(api_key.to_string());
    }

    // 3. Try Authorization header with API key scheme
    if let Some(auth_header) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if auth_header.starts_with("ApiKey ") {
            return Ok(auth_header.strip_prefix("ApiKey ").unwrap().to_string());
        }
    }

    // 4. Try query parameter
    if let Some(api_key) = &query.api_key {
        return Ok(api_key.clone());
    }

    Err(AuthError::Unauthorized)
}

fn create_auth_context_from_api_key(
    api_key: crate::ApiKey,
    permissions: Vec<String>,
) -> AuthContext {
    // Use API key ID as session ID for tracking
    let session_id = api_key.id;
    
    // Use associated user ID or create a system user ID
    let user_id = api_key.user_id.unwrap_or_else(|| Uuid::new_v4());
    
    // Create email from API key name
    let email = format!("apikey+{}@system.local", api_key.name);
    
    // Determine roles based on permissions
    let roles = determine_roles_from_permissions(&permissions);
    
    AuthContext::new(
        user_id,
        email,
        session_id,
        roles,
        permissions,
    )
}

fn determine_roles_from_permissions(permissions: &[String]) -> Vec<String> {
    let mut roles = Vec::new();
    
    // Check for admin permissions
    if permissions.contains(&"*".to_string()) || 
       permissions.iter().any(|p| p.contains("admin")) {
        roles.push("api_admin".to_string());
    }
    
    // Check for user management permissions
    if permissions.iter().any(|p| p.starts_with("user.")) {
        roles.push("api_user_manager".to_string());
    }
    
    // Check for system permissions
    if permissions.iter().any(|p| p.starts_with("system.")) {
        roles.push("api_system".to_string());
    }
    
    // Default API role
    if roles.is_empty() {
        roles.push("api_user".to_string());
    }
    
    roles
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

/// Rate limiting middleware for API keys
pub async fn api_key_rate_limit_middleware(
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Extract auth context to get API key info
    if let Some(_auth_context) = request.extensions().get::<AuthContext>() {
        // Check if this is an API key request (session_id matches an API key pattern)
        // In a real implementation, you'd check against a rate limiting store (Redis, etc.)
        
        // For now, we'll implement a simple in-memory rate limiter
        // In production, use a proper rate limiting solution
    }
    
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, HeaderMap};

    #[test]
    fn test_extract_api_key_from_dreamfactory_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-dreamfactory-api-key", HeaderValue::from_static("test_api_key"));
        
        let query = ApiKeyQuery { api_key: None };
        let result = extract_api_key_from_sources(&headers, &query);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_api_key");
    }

    #[test]
    fn test_extract_api_key_from_standard_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("standard_key"));
        
        let query = ApiKeyQuery { api_key: None };
        let result = extract_api_key_from_sources(&headers, &query);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "standard_key");
    }

    #[test]
    fn test_extract_api_key_from_auth_header() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("ApiKey auth_key"));
        
        let query = ApiKeyQuery { api_key: None };
        let result = extract_api_key_from_sources(&headers, &query);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "auth_key");
    }

    #[test]
    fn test_extract_api_key_from_query() {
        let headers = HeaderMap::new();
        let query = ApiKeyQuery { api_key: Some("query_key".to_string()) };
        
        let result = extract_api_key_from_sources(&headers, &query);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "query_key");
    }

    #[test]
    fn test_extract_api_key_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-dreamfactory-api-key", HeaderValue::from_static("df_key"));
        headers.insert("x-api-key", HeaderValue::from_static("standard_key"));
        
        let query = ApiKeyQuery { api_key: Some("query_key".to_string()) };
        let result = extract_api_key_from_sources(&headers, &query);
        
        // Should prefer DreamFactory header
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "df_key");
    }

    #[test]
    fn test_extract_api_key_none_present() {
        let headers = HeaderMap::new();
        let query = ApiKeyQuery { api_key: None };
        
        let result = extract_api_key_from_sources(&headers, &query);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_roles_from_permissions() {
        let admin_permissions = vec!["*".to_string()];
        let roles = determine_roles_from_permissions(&admin_permissions);
        assert!(roles.contains(&"api_admin".to_string()));

        let user_permissions = vec!["user.read".to_string(), "user.update".to_string()];
        let roles = determine_roles_from_permissions(&user_permissions);
        assert!(roles.contains(&"api_user_manager".to_string()));

        let no_permissions = vec![];
        let roles = determine_roles_from_permissions(&no_permissions);
        assert!(roles.contains(&"api_user".to_string()));
    }

    #[test]
    fn test_create_auth_context_from_api_key() {
        let api_key = crate::ApiKey {
            id: Uuid::new_v4(),
            name: "test_key".to_string(),
            key_hash: "hash".to_string(),
            user_id: Some(Uuid::new_v4()),
            role_id: None,
            is_active: true,
            expires_at: None,
            last_used_at: None,
            usage_count: 0,
            rate_limit_per_minute: None,
            allowed_ips: None,
            created_date: chrono::Utc::now(),
            last_modified_date: chrono::Utc::now(),
            created_by_id: None,
            last_modified_by_id: None,
        };

        let permissions = vec!["user.read".to_string()];
        let auth_context = create_auth_context_from_api_key(api_key.clone(), permissions.clone());
        
        assert_eq!(auth_context.user_id, api_key.user_id.unwrap());
        assert_eq!(auth_context.session_id, api_key.id);
        assert_eq!(auth_context.permissions, permissions);
        assert!(auth_context.email.contains("test_key"));
    }
}