use df_auth::*;
use serde_json::json;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

static DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

async fn get_test_db() -> &'static Pool<Sqlite> {
    DB_POOL.get_or_init(|| async {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        setup_test_schema(&pool).await.unwrap();
        pool
    }).await
}

async fn setup_test_schema(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Create all necessary tables
    sqlx::query(
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            username TEXT,
            password_hash TEXT NOT NULL,
            first_name TEXT,
            last_name TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            is_verified BOOLEAN NOT NULL DEFAULT FALSE,
            last_login_date TEXT,
            created_date TEXT NOT NULL,
            last_modified_date TEXT NOT NULL,
            created_by_id TEXT,
            last_modified_by_id TEXT,
            login_attempts INTEGER NOT NULL DEFAULT 0,
            locked_until TEXT
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            refresh_expires_at TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            ip_address TEXT,
            user_agent TEXT,
            created_date TEXT NOT NULL,
            last_activity TEXT NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE roles (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_date TEXT NOT NULL,
            last_modified_date TEXT NOT NULL,
            created_by_id TEXT,
            last_modified_by_id TEXT
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE permissions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            resource TEXT NOT NULL,
            action TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_date TEXT NOT NULL,
            last_modified_date TEXT NOT NULL,
            created_by_id TEXT,
            last_modified_by_id TEXT
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE user_roles (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            created_date TEXT NOT NULL,
            created_by_id TEXT
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE role_permissions (
            id TEXT PRIMARY KEY,
            role_id TEXT NOT NULL,
            permission_id TEXT NOT NULL,
            created_date TEXT NOT NULL,
            created_by_id TEXT
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_test_auth_service() -> Arc<AuthService> {
    let db = get_test_db().await.clone();
    let config = AuthConfig::default();
    Arc::new(AuthService::new(db, config))
}

#[tokio::test]
async fn test_user_registration_and_login() {
    let auth_service = create_test_auth_service().await;

    // Test user registration
    let register_request = RegisterRequest {
        email: "test@example.com".to_string(),
        username: Some("testuser".to_string()),
        password: "TestPassword123!".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
    };

    let register_response = auth_service.register(register_request).await;
    assert!(register_response.is_ok());
    
    let register_result = register_response.unwrap();
    assert!(register_result.success);
    assert_eq!(register_result.email, "test@example.com");
    assert!(!register_result.session_token.is_empty());

    // Test user login with same credentials
    let login_request = LoginRequest {
        email: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
        remember_me: Some(false),
    };

    let login_response = auth_service.login(login_request, None, None).await;
    assert!(login_response.is_ok());
    
    let login_result = login_response.unwrap();
    assert!(login_result.success);
    assert_eq!(login_result.email, "test@example.com");
    assert!(!login_result.session_token.is_empty());
}

#[tokio::test]
async fn test_invalid_login_credentials() {
    let auth_service = create_test_auth_service().await;

    // Test login with non-existent user
    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "wrongpassword".to_string(),
        remember_me: Some(false),
    };

    let login_response = auth_service.login(login_request, None, None).await;
    assert!(login_response.is_err());
    assert!(matches!(login_response.unwrap_err(), AuthError::UserNotFound));
}

#[tokio::test]
async fn test_login_with_wrong_password() {
    let auth_service = create_test_auth_service().await;

    // First register a user
    let register_request = RegisterRequest {
        email: "test2@example.com".to_string(),
        username: None,
        password: "CorrectPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    auth_service.register(register_request).await.unwrap();

    // Try to login with wrong password
    let login_request = LoginRequest {
        email: "test2@example.com".to_string(),
        password: "WrongPassword123!".to_string(),
        remember_me: Some(false),
    };

    let login_response = auth_service.login(login_request, None, None).await;
    assert!(login_response.is_err());
    assert!(matches!(login_response.unwrap_err(), AuthError::InvalidCredentials));
}

#[tokio::test]
async fn test_session_validation() {
    let auth_service = create_test_auth_service().await;

    // Register and login user
    let register_request = RegisterRequest {
        email: "test3@example.com".to_string(),
        username: None,
        password: "TestPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    let auth_response = auth_service.register(register_request).await.unwrap();
    
    // Validate the session token
    let auth_context = auth_service.validate_session(&auth_response.session_token).await;
    assert!(auth_context.is_ok());
    
    let context = auth_context.unwrap();
    assert_eq!(context.email, "test3@example.com");
    assert_eq!(context.user_id.to_string(), auth_response.user_id);
}

#[tokio::test]
async fn test_invalid_session_token() {
    let auth_service = create_test_auth_service().await;

    let result = auth_service.validate_session("invalid.token.here").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
}

#[tokio::test]
async fn test_refresh_token() {
    let auth_service = create_test_auth_service().await;

    // Register user
    let register_request = RegisterRequest {
        email: "test4@example.com".to_string(),
        username: None,
        password: "TestPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    let auth_response = auth_service.register(register_request).await.unwrap();
    let refresh_token = auth_response.refresh_token.unwrap();

    // Use refresh token to get new session
    let refresh_request = RefreshTokenRequest {
        refresh_token,
    };

    let refresh_response = auth_service.refresh_token(refresh_request).await;
    assert!(refresh_response.is_ok());
    
    let new_auth = refresh_response.unwrap();
    assert!(new_auth.success);
    assert_eq!(new_auth.email, "test4@example.com");
    assert!(!new_auth.session_token.is_empty());
    assert_ne!(new_auth.session_token, auth_response.session_token); // Should be different
}

#[tokio::test]
async fn test_logout() {
    let auth_service = create_test_auth_service().await;

    // Register and login user
    let register_request = RegisterRequest {
        email: "test5@example.com".to_string(),
        username: None,
        password: "TestPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    let auth_response = auth_service.register(register_request).await.unwrap();
    
    // Logout
    let logout_response = auth_service.logout(&auth_response.session_token).await;
    assert!(logout_response.is_ok());
    
    let logout_result = logout_response.unwrap();
    assert!(logout_result.success);

    // Try to validate the token after logout - should fail
    let validation_result = auth_service.validate_session(&auth_response.session_token).await;
    assert!(validation_result.is_err());
    assert!(matches!(validation_result.unwrap_err(), AuthError::SessionNotFound));
}

#[tokio::test]
async fn test_password_validation() {
    let auth_service = create_test_auth_service().await;

    // Test weak password
    let register_request = RegisterRequest {
        email: "weakpass@example.com".to_string(),
        username: None,
        password: "weak".to_string(), // Too short
        first_name: None,
        last_name: None,
    };

    let result = auth_service.register(register_request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::Validation(_)));
}

#[tokio::test]
async fn test_duplicate_email_registration() {
    let auth_service = create_test_auth_service().await;

    // Register first user
    let register_request1 = RegisterRequest {
        email: "duplicate@example.com".to_string(),
        username: None,
        password: "TestPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    let result1 = auth_service.register(register_request1).await;
    assert!(result1.is_ok());

    // Try to register second user with same email
    let register_request2 = RegisterRequest {
        email: "duplicate@example.com".to_string(),
        username: None,
        password: "AnotherPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    let result2 = auth_service.register(register_request2).await;
    assert!(result2.is_err());
    assert!(matches!(result2.unwrap_err(), AuthError::UserAlreadyExists));
}

#[tokio::test]
async fn test_get_user_profile() {
    let auth_service = create_test_auth_service().await;

    // Register user
    let register_request = RegisterRequest {
        email: "profile@example.com".to_string(),
        username: Some("profileuser".to_string()),
        password: "TestPassword123!".to_string(),
        first_name: Some("Profile".to_string()),
        last_name: Some("User".to_string()),
    };

    let auth_response = auth_service.register(register_request).await.unwrap();
    let user_id = Uuid::parse_str(&auth_response.user_id).unwrap();

    // Get user profile
    let profile = auth_service.get_user_profile(user_id).await;
    assert!(profile.is_ok());
    
    let profile_data = profile.unwrap();
    assert_eq!(profile_data.email, "profile@example.com");
    assert_eq!(profile_data.username, Some("profileuser".to_string()));
    assert_eq!(profile_data.first_name, Some("Profile".to_string()));
    assert_eq!(profile_data.last_name, Some("User".to_string()));
}

#[tokio::test]
async fn test_change_password() {
    let auth_service = create_test_auth_service().await;

    // Register user
    let register_request = RegisterRequest {
        email: "changepass@example.com".to_string(),
        username: None,
        password: "OldPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    let auth_response = auth_service.register(register_request).await.unwrap();
    let user_id = Uuid::parse_str(&auth_response.user_id).unwrap();

    // Change password
    let change_result = auth_service.change_password(
        user_id,
        "OldPassword123!",
        "NewPassword123!",
    ).await;
    assert!(change_result.is_ok());

    // Try to login with old password - should fail
    let login_old = LoginRequest {
        email: "changepass@example.com".to_string(),
        password: "OldPassword123!".to_string(),
        remember_me: None,
    };

    let old_login_result = auth_service.login(login_old, None, None).await;
    assert!(old_login_result.is_err());

    // Login with new password - should succeed
    let login_new = LoginRequest {
        email: "changepass@example.com".to_string(),
        password: "NewPassword123!".to_string(),
        remember_me: None,
    };

    let new_login_result = auth_service.login(login_new, None, None).await;
    assert!(new_login_result.is_ok());
}