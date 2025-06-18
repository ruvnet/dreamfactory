use crate::{AuthError, Result, AuthContext, UserService, UpdateUserRequest};
use axum::{
    extract::{Extension, Path, State},
    response::Json,
    Json as JsonBody,
};
use std::sync::Arc;
use uuid::Uuid;

pub type UserServiceState = Arc<UserService>;

pub async fn get_current_user_profile(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
) -> Result<Json<crate::UserProfile>> {
    let profile = user_service.get_user_profile(auth_context.user_id).await?;
    Ok(Json(profile))
}

pub async fn update_current_user_profile(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    JsonBody(request): JsonBody<UpdateUserRequest>,
) -> Result<Json<crate::UserProfile>> {
    // Users can only update their own profile
    let updated_user = user_service.update_user(
        auth_context.user_id,
        request,
        Some(auth_context.user_id),
    ).await?;

    let profile = user_service.get_user_profile(updated_user.id).await?;
    Ok(Json(profile))
}

pub async fn change_current_user_password(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    JsonBody(request): JsonBody<crate::ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    // Verify current password
    if !user_service.verify_password(auth_context.user_id, &request.current_password).await? {
        return Err(AuthError::InvalidCredentials);
    }

    // Update to new password
    let update_request = UpdateUserRequest {
        password: Some(request.new_password),
        ..Default::default()
    };

    user_service.update_user(
        auth_context.user_id,
        update_request,
        Some(auth_context.user_id),
    ).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully"
    })))
}

pub async fn get_user_sessions(
    Extension(auth_context): Extension<AuthContext>,
    State(session_service): State<Arc<crate::SessionService>>,
) -> Result<Json<Vec<crate::Session>>> {
    // Users can only view their own sessions
    let sessions = session_service.get_active_sessions(auth_context.user_id).await?;
    Ok(Json(sessions))
}

pub async fn terminate_user_session(
    Extension(auth_context): Extension<AuthContext>,
    State(session_service): State<Arc<crate::SessionService>>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    // Get the session to verify ownership
    let session = session_service.get_session(session_id).await?;
    
    // Users can only terminate their own sessions
    if session.user_id != auth_context.user_id {
        return Err(AuthError::Forbidden);
    }

    session_service.deactivate_session(session_id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Session terminated successfully"
    })))
}

pub async fn get_user_api_keys(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<Arc<crate::ApiKeyService>>,
) -> Result<Json<Vec<crate::ApiKey>>> {
    // Users can only view their own API keys
    let api_keys = api_key_service.list_api_keys(Some(auth_context.user_id), None, None).await?;
    Ok(Json(api_keys))
}

pub async fn create_user_api_key(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<Arc<crate::ApiKeyService>>,
    JsonBody(mut request): JsonBody<crate::CreateApiKeyRequest>,
) -> Result<Json<crate::ApiKeyResponse>> {
    // Force the API key to be associated with the current user
    request.user_id = Some(auth_context.user_id);

    let api_key = api_key_service.create_api_key(request, Some(auth_context.user_id)).await?;
    Ok(Json(api_key))
}

pub async fn update_user_api_key(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<Arc<crate::ApiKeyService>>,
    Path(api_key_id): Path<Uuid>,
    JsonBody(request): JsonBody<crate::UpdateApiKeyRequest>,
) -> Result<Json<crate::ApiKey>> {
    // Get the API key to verify ownership
    let existing_key = api_key_service.get_api_key_by_id(api_key_id).await?;
    
    // Users can only update their own API keys
    if existing_key.user_id != Some(auth_context.user_id) {
        return Err(AuthError::Forbidden);
    }

    let updated_key = api_key_service.update_api_key(
        api_key_id,
        request,
        Some(auth_context.user_id),
    ).await?;

    Ok(Json(updated_key))
}

pub async fn delete_user_api_key(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<Arc<crate::ApiKeyService>>,
    Path(api_key_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    // Get the API key to verify ownership
    let existing_key = api_key_service.get_api_key_by_id(api_key_id).await?;
    
    // Users can only delete their own API keys
    if existing_key.user_id != Some(auth_context.user_id) {
        return Err(AuthError::Forbidden);
    }

    api_key_service.delete_api_key(api_key_id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "API key deleted successfully"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthConfig, AuthContext};
    use uuid::Uuid;

    fn create_test_auth_context() -> AuthContext {
        AuthContext::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            Uuid::new_v4(),
            vec!["user".to_string()],
            vec!["user.read".to_string(), "user.update".to_string()],
        )
    }

    #[tokio::test]
    async fn test_auth_context_creation() {
        let context = create_test_auth_context();
        assert_eq!(context.email, "test@example.com");
        assert!(context.has_role("user"));
        assert!(context.has_permission("user", "read"));
        assert!(context.has_permission("user", "update"));
        assert!(!context.has_permission("admin", "read"));
    }
}