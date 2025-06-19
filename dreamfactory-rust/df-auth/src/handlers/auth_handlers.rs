use crate::{
    AuthError, Result, LoginRequest, RegisterRequest, RefreshTokenRequest,
    AuthResponse, LogoutResponse, ChangePasswordRequest, AuthContext, AuthServiceState
};
use axum::{
    extract::{State, Extension},
    http::{HeaderMap, StatusCode},
    response::Json,
    Json as JsonBody,
};

pub async fn login(
    State(auth_service): State<AuthServiceState>,
    headers: HeaderMap,
    JsonBody(request): JsonBody<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let ip_address = get_client_ip(&headers);
    let user_agent = get_user_agent(&headers);

    let response = auth_service.login(request, ip_address, user_agent).await?;
    Ok(Json(response))
}

pub async fn register(
    State(auth_service): State<AuthServiceState>,
    JsonBody(request): JsonBody<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>)> {
    let response = auth_service.register(request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn logout(
    State(auth_service): State<AuthServiceState>,
    headers: HeaderMap,
) -> Result<Json<LogoutResponse>> {
    let token = extract_bearer_token(&headers)?;
    let response = auth_service.logout(&token).await?;
    Ok(Json(response))
}

pub async fn refresh_token(
    State(auth_service): State<AuthServiceState>,
    JsonBody(request): JsonBody<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>> {
    let response = auth_service.refresh_token(request).await?;
    Ok(Json(response))
}

pub async fn change_password(
    State(auth_service): State<AuthServiceState>,
    Extension(auth_context): Extension<AuthContext>,
    JsonBody(request): JsonBody<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    auth_service.change_password(
        auth_context.user_id,
        &request.current_password,
        &request.new_password,
    ).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully"
    })))
}

pub async fn get_profile(
    State(auth_service): State<AuthServiceState>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<crate::UserProfile>> {
    let profile = auth_service.get_user_profile(auth_context.user_id).await?;
    Ok(Json(profile))
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "df-auth",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Helper functions
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

fn get_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try different headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "x-client-ip",
        "cf-connecting-ip",
    ];

    for header_name in &ip_headers {
        if let Some(value) = headers.get(*header_name).and_then(|h| h.to_str().ok()) {
            // For X-Forwarded-For, take the first IP
            let ip = value.split(',').next().unwrap_or(value).trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    None
}

fn get_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, HeaderMap};

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer test_token_123"));

        let token = extract_bearer_token(&headers).unwrap();
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_extract_bearer_token_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Invalid token"));

        let result = extract_bearer_token(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_client_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.1, 10.0.0.1"));

        let ip = get_client_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_get_user_agent() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Test Agent)"));

        let user_agent = get_user_agent(&headers);
        assert_eq!(user_agent, Some("Mozilla/5.0 (Test Agent)".to_string()));
    }
}