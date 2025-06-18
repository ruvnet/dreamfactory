use crate::{AuthError, AuthContext};
use axum::{
    extract::{Request, Path},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension,
};
use std::collections::HashMap;

/// RBAC middleware that checks if the authenticated user has the required permission
pub async fn rbac_middleware(
    resource: String,
    action: String,
) -> impl Fn(
    Extension<AuthContext>,
    Request,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, StatusCode>> + Send>> + Clone {
    move |Extension(auth_context): Extension<AuthContext>, request: Request, next: Next| {
        let resource = resource.clone();
        let action = action.clone();
        
        Box::pin(async move {
            if auth_context.has_permission(&resource, &action) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        })
    }
}

/// Create RBAC middleware for a specific resource and action
pub fn require_permission(resource: &str, action: &str) -> impl Fn(Extension<AuthContext>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, StatusCode>> + Send>> + Clone {
    let resource = resource.to_string();
    let action = action.to_string();
    
    move |Extension(auth_context): Extension<AuthContext>, request: Request, next: Next| {
        let resource = resource.clone();
        let action = action.clone();
        
        Box::pin(async move {
            if auth_context.has_permission(&resource, &action) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        })
    }
}

/// Middleware that requires admin role
pub async fn require_admin(
    Extension(auth_context): Extension<AuthContext>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    if auth_context.is_admin {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Middleware that requires specific role
pub fn require_role(role: &str) -> impl Fn(Extension<AuthContext>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, StatusCode>> + Send>> + Clone {
    let required_role = role.to_string();
    
    move |Extension(auth_context): Extension<AuthContext>, request: Request, next: Next| {
        let required_role = required_role.clone();
        
        Box::pin(async move {
            if auth_context.has_role(&required_role) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        })
    }
}

/// Dynamic RBAC middleware that extracts resource from path parameters
pub async fn dynamic_rbac_middleware(
    Extension(auth_context): Extension<AuthContext>,
    Path(params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Extract resource and action from the request
    let method = request.method().to_string().to_lowercase();
    let path = request.uri().path();
    
    let (resource, action) = determine_resource_action(&path, &method, &params);
    
    if auth_context.has_permission(&resource, &action) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Middleware for resource ownership - ensures user can only access their own resources
pub async fn require_ownership_or_admin(
    Extension(auth_context): Extension<AuthContext>,
    Path(params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Admin can access everything
    if auth_context.is_admin {
        return Ok(next.run(request).await);
    }
    
    // Check if the resource belongs to the current user
    if let Some(user_id) = params.get("user_id") {
        if user_id == &auth_context.user_id.to_string() {
            return Ok(next.run(request).await);
        }
    }
    
    // Check if the resource ID matches the current user
    if let Some(id) = params.get("id") {
        if id == &auth_context.user_id.to_string() {
            return Ok(next.run(request).await);
        }
    }
    
    Err(StatusCode::FORBIDDEN)
}

/// Helper function to determine resource and action from request
fn determine_resource_action(
    path: &str,
    method: &str,
    params: &HashMap<String, String>,
) -> (String, String) {
    // Extract base resource from path
    let resource = if path.contains("/api/v2/user") {
        "user".to_string()
    } else if path.contains("/api/v2/system") {
        "system".to_string()
    } else if path.contains("/api/v2/admin") {
        "admin".to_string()
    } else {
        // Default or extract from path segments
        path.split('/').nth(3).unwrap_or("api").to_string()
    };
    
    // Determine action from HTTP method
    let action = match method {
        "get" => {
            if path.ends_with("s") || !params.contains_key("id") {
                "list".to_string()
            } else {
                "read".to_string()
            }
        }
        "post" => "create".to_string(),
        "put" | "patch" => "update".to_string(),
        "delete" => "delete".to_string(),
        _ => "execute".to_string(),
    };
    
    (resource, action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_determine_resource_action() {
        let mut params = HashMap::new();
        
        // Test list users
        let (resource, action) = determine_resource_action("/api/v2/user", "get", &params);
        assert_eq!(resource, "user");
        assert_eq!(action, "list");
        
        // Test get specific user
        params.insert("id".to_string(), "123".to_string());
        let (resource, action) = determine_resource_action("/api/v2/user/123", "get", &params);
        assert_eq!(resource, "user");
        assert_eq!(action, "read");
        
        // Test create user
        let (resource, action) = determine_resource_action("/api/v2/user", "post", &HashMap::new());
        assert_eq!(resource, "user");
        assert_eq!(action, "create");
        
        // Test update user
        let (resource, action) = determine_resource_action("/api/v2/user/123", "put", &params);
        assert_eq!(resource, "user");
        assert_eq!(action, "update");
        
        // Test delete user
        let (resource, action) = determine_resource_action("/api/v2/user/123", "delete", &params);
        assert_eq!(resource, "user");
        assert_eq!(action, "delete");
    }

    #[test]
    fn test_auth_context_permissions() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let roles = vec!["user".to_string()];
        let permissions = vec!["user.read".to_string(), "user.update".to_string()];
        
        let auth_context = AuthContext::new(
            user_id,
            "test@example.com".to_string(),
            session_id,
            roles,
            permissions,
        );
        
        assert!(auth_context.has_permission("user", "read"));
        assert!(auth_context.has_permission("user", "update"));
        assert!(!auth_context.has_permission("user", "delete"));
        assert!(!auth_context.has_permission("admin", "read"));
    }

    #[test]
    fn test_admin_permissions() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let roles = vec!["admin".to_string()];
        let permissions = vec!["*".to_string()];
        
        let auth_context = AuthContext::new(
            user_id,
            "admin@example.com".to_string(),
            session_id,
            roles,
            permissions,
        );
        
        assert!(auth_context.is_admin);
        assert!(auth_context.has_permission("user", "read"));
        assert!(auth_context.has_permission("user", "delete"));
        assert!(auth_context.has_permission("system", "admin"));
    }
}