use crate::{
    AuthError, Result, AuthContext, UserService, RoleService, PermissionService, 
    ApiKeyService, SessionService, CreateUserRequest, UpdateUserRequest,
    CreateRoleRequest, UpdateRoleRequest, CreatePermissionRequest, UpdatePermissionRequest,
    CreateApiKeyRequest, UpdateApiKeyRequest
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
    Json as JsonBody,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub type UserServiceState = Arc<UserService>;
pub type RoleServiceState = Arc<RoleService>;
pub type PermissionServiceState = Arc<PermissionService>;
pub type ApiKeyServiceState = Arc<ApiKeyService>;
pub type SessionServiceState = Arc<SessionService>;

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub total: Option<i64>,
    pub limit: i64,
    pub offset: i64,
}

// User Management Handlers
pub async fn admin_list_users(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<crate::User>>> {
    if !auth_context.has_permission("user", "list") {
        return Err(AuthError::PermissionDenied);
    }

    let users = user_service.list_users(query.limit, query.offset).await?;
    
    Ok(Json(ListResponse {
        data: users,
        total: None, // Could add count query if needed
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
    }))
}

pub async fn admin_create_user(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    JsonBody(request): JsonBody<CreateUserRequest>,
) -> Result<(StatusCode, Json<crate::User>)> {
    if !auth_context.has_permission("user", "create") {
        return Err(AuthError::PermissionDenied);
    }

    let user = user_service.create_user(request, Some(auth_context.user_id)).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn admin_get_user(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<crate::UserProfile>> {
    if !auth_context.has_permission("user", "read") {
        return Err(AuthError::PermissionDenied);
    }

    let profile = user_service.get_user_profile(user_id).await?;
    Ok(Json(profile))
}

pub async fn admin_update_user(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    Path(user_id): Path<Uuid>,
    JsonBody(request): JsonBody<UpdateUserRequest>,
) -> Result<Json<crate::User>> {
    if !auth_context.has_permission("user", "update") {
        return Err(AuthError::PermissionDenied);
    }

    let user = user_service.update_user(user_id, request, Some(auth_context.user_id)).await?;
    Ok(Json(user))
}

pub async fn admin_delete_user(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    if !auth_context.has_permission("user", "delete") {
        return Err(AuthError::PermissionDenied);
    }

    // Prevent users from deleting themselves
    if user_id == auth_context.user_id {
        return Err(AuthError::Validation("Cannot delete your own account".to_string()));
    }

    user_service.delete_user(user_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User deleted successfully"
    })))
}

// Role Management Handlers
pub async fn admin_list_roles(
    Extension(auth_context): Extension<AuthContext>,
    State(role_service): State<RoleServiceState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<crate::Role>>> {
    if !auth_context.has_permission("role", "list") {
        return Err(AuthError::PermissionDenied);
    }

    let roles = role_service.list_roles(query.limit, query.offset).await?;
    
    Ok(Json(ListResponse {
        data: roles,
        total: None,
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
    }))
}

pub async fn admin_create_role(
    Extension(auth_context): Extension<AuthContext>,
    State(role_service): State<RoleServiceState>,
    JsonBody(request): JsonBody<CreateRoleRequest>,
) -> Result<(StatusCode, Json<crate::Role>)> {
    if !auth_context.has_permission("role", "create") {
        return Err(AuthError::PermissionDenied);
    }

    let role = role_service.create_role(request, Some(auth_context.user_id)).await?;
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn admin_get_role(
    Extension(auth_context): Extension<AuthContext>,
    State(role_service): State<RoleServiceState>,
    Path(role_id): Path<Uuid>,
) -> Result<Json<crate::Role>> {
    if !auth_context.has_permission("role", "read") {
        return Err(AuthError::PermissionDenied);
    }

    let role = role_service.get_role_by_id(role_id).await?;
    Ok(Json(role))
}

pub async fn admin_update_role(
    Extension(auth_context): Extension<AuthContext>,
    State(role_service): State<RoleServiceState>,
    Path(role_id): Path<Uuid>,
    JsonBody(request): JsonBody<UpdateRoleRequest>,
) -> Result<Json<crate::Role>> {
    if !auth_context.has_permission("role", "update") {
        return Err(AuthError::PermissionDenied);
    }

    let role = role_service.update_role(role_id, request, Some(auth_context.user_id)).await?;
    Ok(Json(role))
}

pub async fn admin_delete_role(
    Extension(auth_context): Extension<AuthContext>,
    State(role_service): State<RoleServiceState>,
    Path(role_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    if !auth_context.has_permission("role", "delete") {
        return Err(AuthError::PermissionDenied);
    }

    role_service.delete_role(role_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Role deleted successfully"
    })))
}

// Permission Management Handlers
pub async fn admin_list_permissions(
    Extension(auth_context): Extension<AuthContext>,
    State(permission_service): State<PermissionServiceState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<crate::Permission>>> {
    if !auth_context.has_permission("permission", "list") {
        return Err(AuthError::PermissionDenied);
    }

    let permissions = permission_service.list_permissions(query.limit, query.offset).await?;
    
    Ok(Json(ListResponse {
        data: permissions,
        total: None,
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
    }))
}

pub async fn admin_create_permission(
    Extension(auth_context): Extension<AuthContext>,
    State(permission_service): State<PermissionServiceState>,
    JsonBody(request): JsonBody<CreatePermissionRequest>,
) -> Result<(StatusCode, Json<crate::Permission>)> {
    if !auth_context.has_permission("permission", "create") {
        return Err(AuthError::PermissionDenied);
    }

    let permission = permission_service.create_permission(request, Some(auth_context.user_id)).await?;
    Ok((StatusCode::CREATED, Json(permission)))
}

// API Key Management Handlers
pub async fn admin_list_api_keys(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<ApiKeyServiceState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<crate::ApiKey>>> {
    if !auth_context.has_permission("api_key", "list") {
        return Err(AuthError::PermissionDenied);
    }

    let api_keys = api_key_service.list_api_keys(None, query.limit, query.offset).await?;
    
    Ok(Json(ListResponse {
        data: api_keys,
        total: None,
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
    }))
}

pub async fn admin_create_api_key(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<ApiKeyServiceState>,
    JsonBody(request): JsonBody<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<crate::ApiKeyResponse>)> {
    if !auth_context.has_permission("api_key", "create") {
        return Err(AuthError::PermissionDenied);
    }

    let api_key = api_key_service.create_api_key(request, Some(auth_context.user_id)).await?;
    Ok((StatusCode::CREATED, Json(api_key)))
}

pub async fn admin_get_api_key(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<ApiKeyServiceState>,
    Path(api_key_id): Path<Uuid>,
) -> Result<Json<crate::ApiKey>> {
    if !auth_context.has_permission("api_key", "read") {
        return Err(AuthError::PermissionDenied);
    }

    let api_key = api_key_service.get_api_key_by_id(api_key_id).await?;
    Ok(Json(api_key))
}

pub async fn admin_update_api_key(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<ApiKeyServiceState>,
    Path(api_key_id): Path<Uuid>,
    JsonBody(request): JsonBody<UpdateApiKeyRequest>,
) -> Result<Json<crate::ApiKey>> {
    if !auth_context.has_permission("api_key", "update") {
        return Err(AuthError::PermissionDenied);
    }

    let api_key = api_key_service.update_api_key(api_key_id, request, Some(auth_context.user_id)).await?;
    Ok(Json(api_key))
}

pub async fn admin_delete_api_key(
    Extension(auth_context): Extension<AuthContext>,
    State(api_key_service): State<ApiKeyServiceState>,
    Path(api_key_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    if !auth_context.has_permission("api_key", "delete") {
        return Err(AuthError::PermissionDenied);
    }

    api_key_service.delete_api_key(api_key_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "API key deleted successfully"
    })))
}

// Session Management Handlers
pub async fn admin_list_sessions(
    Extension(auth_context): Extension<AuthContext>,
    State(session_service): State<SessionServiceState>,
) -> Result<Json<Vec<crate::Session>>> {
    if !auth_context.has_permission("session", "list") {
        return Err(AuthError::PermissionDenied);
    }

    // This would need a list_all_sessions method in SessionService
    // For now, return empty list
    Ok(Json(vec![]))
}

pub async fn admin_terminate_session(
    Extension(auth_context): Extension<AuthContext>,
    State(session_service): State<SessionServiceState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    if !auth_context.has_permission("session", "delete") {
        return Err(AuthError::PermissionDenied);
    }

    session_service.deactivate_session(session_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Session terminated successfully"
    })))
}

pub async fn admin_get_user_sessions(
    Extension(auth_context): Extension<AuthContext>,
    State(session_service): State<SessionServiceState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<crate::Session>>> {
    if !auth_context.has_permission("session", "read") {
        return Err(AuthError::PermissionDenied);
    }

    let sessions = session_service.get_active_sessions(user_id).await?;
    Ok(Json(sessions))
}

pub async fn admin_terminate_user_sessions(
    Extension(auth_context): Extension<AuthContext>,
    State(session_service): State<SessionServiceState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    if !auth_context.has_permission("session", "delete") {
        return Err(AuthError::PermissionDenied);
    }

    session_service.deactivate_user_sessions(user_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "All user sessions terminated successfully"
    })))
}

// Role-User Assignment Handlers
#[derive(Deserialize)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

pub async fn admin_assign_role_to_user(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    Path(user_id): Path<Uuid>,
    JsonBody(request): JsonBody<AssignRoleRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth_context.has_permission("user", "update") {
        return Err(AuthError::PermissionDenied);
    }

    user_service.assign_role_to_user(user_id, request.role_id, Some(auth_context.user_id)).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Role assigned to user successfully"
    })))
}

pub async fn admin_remove_role_from_user(
    Extension(auth_context): Extension<AuthContext>,
    State(user_service): State<UserServiceState>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    if !auth_context.has_permission("user", "update") {
        return Err(AuthError::PermissionDenied);
    }

    user_service.remove_role_from_user(user_id, role_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Role removed from user successfully"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthContext;
    use uuid::Uuid;

    fn create_admin_auth_context() -> AuthContext {
        AuthContext::new(
            Uuid::new_v4(),
            "admin@example.com".to_string(),
            Uuid::new_v4(),
            vec!["admin".to_string()],
            vec!["*".to_string()],
        )
    }

    fn create_limited_auth_context() -> AuthContext {
        AuthContext::new(
            Uuid::new_v4(),
            "user@example.com".to_string(),
            Uuid::new_v4(),
            vec!["user".to_string()],
            vec!["user.read".to_string()],
        )
    }

    #[test]
    fn test_admin_context_permissions() {
        let admin_context = create_admin_auth_context();
        assert!(admin_context.is_admin);
        assert!(admin_context.has_permission("user", "create"));
        assert!(admin_context.has_permission("user", "delete"));
        assert!(admin_context.has_permission("role", "create"));
    }

    #[test]
    fn test_limited_context_permissions() {
        let user_context = create_limited_auth_context();
        assert!(!user_context.is_admin);
        assert!(user_context.has_permission("user", "read"));
        assert!(!user_context.has_permission("user", "create"));
        assert!(!user_context.has_permission("user", "delete"));
    }
}