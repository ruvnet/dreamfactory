use df_auth::*;
use uuid::Uuid;

#[test]
fn test_jwt_service_creation() {
    let config = AuthConfig::default();
    let jwt_service = JwtService::new(&config);
    assert_eq!(jwt_service.get_expiration(), config.jwt_expiration);
}

#[test]
fn test_jwt_token_generation() {
    let config = AuthConfig::default();
    let jwt_service = JwtService::new(&config);
    
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["user".to_string()];
    let permissions = vec!["read".to_string()];

    let token = jwt_service.generate_token(
        user_id,
        email.clone(),
        session_id,
        roles.clone(),
        permissions.clone(),
    );

    assert!(token.is_ok());
    let token_str = token.unwrap();
    assert!(!token_str.is_empty());
    assert!(token_str.contains('.'), "JWT should contain dots");
}

#[test]
fn test_jwt_token_validation() {
    let config = AuthConfig::default();
    let jwt_service = JwtService::new(&config);
    
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["user".to_string()];
    let permissions = vec!["read".to_string()];

    // Generate token
    let token = jwt_service.generate_token(
        user_id,
        email.clone(),
        session_id,
        roles.clone(),
        permissions.clone(),
    ).unwrap();

    // Validate token
    let claims = jwt_service.validate_token(&token);
    assert!(claims.is_ok());
    
    let claims_data = claims.unwrap();
    assert_eq!(claims_data.sub, user_id.to_string());
    assert_eq!(claims_data.email, email);
    assert_eq!(claims_data.jti, session_id.to_string());
    assert_eq!(claims_data.roles, roles);
    assert_eq!(claims_data.permissions, permissions);
}

#[test]
fn test_jwt_invalid_token() {
    let config = AuthConfig::default();
    let jwt_service = JwtService::new(&config);
    
    let result = jwt_service.validate_token("invalid.token.here");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
}

#[test]
fn test_jwt_malformed_token() {
    let config = AuthConfig::default();
    let jwt_service = JwtService::new(&config);
    
    let result = jwt_service.validate_token("not.a.jwt");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
}

#[test]
fn test_jwt_empty_token() {
    let config = AuthConfig::default();
    let jwt_service = JwtService::new(&config);
    
    let result = jwt_service.validate_token("");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
}

#[test]
fn test_jwt_token_refresh() {
    let config = AuthConfig::default();
    let jwt_service = JwtService::new(&config);
    
    let user_id = Uuid::new_v4();
    let old_session_id = Uuid::new_v4();
    let new_session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["user".to_string()];
    let permissions = vec!["read".to_string()];

    // Generate original token
    let original_token = jwt_service.generate_token(
        user_id,
        email.clone(),
        old_session_id,
        roles.clone(),
        permissions.clone(),
    ).unwrap();

    // Refresh token
    let refreshed_token = jwt_service.refresh_token(&original_token, new_session_id);
    assert!(refreshed_token.is_ok());
    
    let new_token = refreshed_token.unwrap();
    assert_ne!(original_token, new_token);

    // Validate new token
    let claims = jwt_service.validate_token(&new_token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.jti, new_session_id.to_string());
    assert_eq!(claims.email, email);
}

#[test] 
fn test_jwt_claims_expiration_check() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["user".to_string()];
    let permissions = vec!["read".to_string()];

    // Create claims that expire immediately
    let claims = Claims::new(
        user_id,
        email,
        session_id,
        roles,
        permissions,
        -1, // Expired
    );

    assert!(claims.is_expired());
}

#[test]
fn test_jwt_claims_not_expired() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["user".to_string()];
    let permissions = vec!["read".to_string()];

    // Create claims that expire in the future
    let claims = Claims::new(
        user_id,
        email,
        session_id,
        roles,
        permissions,
        3600, // 1 hour from now
    );

    assert!(!claims.is_expired());
}

#[test]
fn test_auth_context_from_claims() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["admin".to_string()];
    let permissions = vec!["*".to_string()];

    let claims = Claims::new(
        user_id,
        email.clone(),
        session_id,
        roles.clone(),
        permissions.clone(),
        3600,
    );

    let auth_context = AuthContext::from_claims(&claims);
    assert!(auth_context.is_ok());
    
    let context = auth_context.unwrap();
    assert_eq!(context.user_id, user_id);
    assert_eq!(context.email, email);
    assert_eq!(context.session_id, session_id);
    assert_eq!(context.roles, roles);
    assert_eq!(context.permissions, permissions);
    assert!(context.is_admin);
}

#[test]
fn test_auth_context_permissions() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["user".to_string()];
    let permissions = vec!["user.read".to_string(), "user.update".to_string()];

    let auth_context = AuthContext::new(
        user_id,
        email,
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
fn test_auth_context_wildcard_permissions() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "admin@example.com".to_string();
    let roles = vec!["admin".to_string()];
    let permissions = vec!["*".to_string()];

    let auth_context = AuthContext::new(
        user_id,
        email,
        session_id,
        roles,
        permissions,
    );

    assert!(auth_context.has_permission("user", "read"));
    assert!(auth_context.has_permission("user", "delete"));
    assert!(auth_context.has_permission("admin", "read"));
    assert!(auth_context.has_permission("system", "admin"));
}

#[test]
fn test_auth_context_resource_wildcard() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "manager@example.com".to_string();
    let roles = vec!["manager".to_string()];
    let permissions = vec!["user.*".to_string()];

    let auth_context = AuthContext::new(
        user_id,
        email,
        session_id,
        roles,
        permissions,
    );

    assert!(auth_context.has_permission("user", "read"));
    assert!(auth_context.has_permission("user", "create"));
    assert!(auth_context.has_permission("user", "update"));
    assert!(auth_context.has_permission("user", "delete"));
    assert!(!auth_context.has_permission("admin", "read"));
}

#[test]
fn test_auth_context_role_checking() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let roles = vec!["user".to_string(), "moderator".to_string()];
    let permissions = vec![];

    let auth_context = AuthContext::new(
        user_id,
        email,
        session_id,
        roles,
        permissions,
    );

    assert!(auth_context.has_role("user"));
    assert!(auth_context.has_role("moderator"));
    assert!(auth_context.has_role("USER")); // Case insensitive
    assert!(!auth_context.has_role("admin"));
}