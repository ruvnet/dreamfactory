use crate::{
    AuthError, Result, AuthContext, LoginRequest, RegisterRequest,
    AuthResponse, LogoutResponse, RefreshTokenRequest,
    UserService, SessionService, JwtService, AuthConfig
};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct AuthService {
    user_service: UserService,
    session_service: SessionService,
    jwt_service: JwtService,
    config: AuthConfig,
}

impl AuthService {
    pub fn new(db: Pool<Sqlite>, config: AuthConfig) -> Self {
        let user_service = UserService::new(db.clone(), config.clone());
        let session_service = SessionService::new(db, config.clone());
        let jwt_service = JwtService::new(&config);

        Self {
            user_service,
            session_service,
            jwt_service,
            config,
        }
    }

    pub async fn login(
        &self,
        request: LoginRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResponse> {
        // Authenticate user
        let user = self.user_service.authenticate_user(&request.email, &request.password).await?;

        // Get user roles and permissions
        let roles = self.user_service.get_user_roles(user.id).await?;
        let permissions = self.user_service.get_user_permissions(user.id).await?;

        // Create session
        let session = self.session_service.create_session(
            user.id,
            roles.clone(),
            permissions.clone(),
            ip_address,
            user_agent,
            request.remember_me.unwrap_or(false),
        ).await?;

        Ok(AuthResponse {
            success: true,
            session_token: session.session_token,
            session_id: session.session_id.to_string(),
            user_id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            expires_in: self.config.jwt_expiration,
            refresh_token: Some(session.refresh_token),
        })
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        // Create user
        let create_request = crate::CreateUserRequest {
            email: request.email.clone(),
            username: request.username,
            password: request.password,
            first_name: request.first_name,
            last_name: request.last_name,
            role_id: None, // Default role will be assigned later
        };

        let user = self.user_service.create_user(create_request, None).await?;

        // Assign default user role (assuming role with name "user" exists)
        if let Ok(roles) = sqlx::query("SELECT id FROM roles WHERE name = 'user' AND is_active = TRUE")
            .fetch_all(self.user_service.db())
            .await 
        {
            if let Some(role) = roles.first() {
                let role_id: String = role.get("id");
                if let Ok(role_uuid) = Uuid::parse_str(&role_id) {
                    let _ = self.user_service.assign_role_to_user(user.id, role_uuid, None).await;
                }
            }
        }

        // Auto-login after registration
        let _login_request = LoginRequest {
            email: request.email,
            password: String::new(), // We don't need to re-verify password
            remember_me: Some(false),
        };

        // Get user roles and permissions for session
        let roles = self.user_service.get_user_roles(user.id).await?;
        let permissions = self.user_service.get_user_permissions(user.id).await?;

        // Create session
        let session = self.session_service.create_session(
            user.id,
            roles,
            permissions,
            None,
            None,
            false,
        ).await?;

        Ok(AuthResponse {
            success: true,
            session_token: session.session_token,
            session_id: session.session_id.to_string(),
            user_id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            expires_in: self.config.jwt_expiration,
            refresh_token: Some(session.refresh_token),
        })
    }

    pub async fn logout(&self, session_token: &str) -> Result<LogoutResponse> {
        // Validate token to get session info
        let claims = self.jwt_service.validate_token(session_token)?;
        let session_id = Uuid::parse_str(&claims.jti)
            .map_err(|_| AuthError::InvalidToken)?;

        // Deactivate session
        self.session_service.deactivate_session(session_id).await?;

        Ok(LogoutResponse {
            success: true,
            message: "Logged out successfully".to_string(),
        })
    }

    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> Result<AuthResponse> {
        // Validate and refresh session
        let session_response = self.session_service.refresh_session(&request.refresh_token).await?;

        // Get user info
        let user = self.user_service.get_user_by_id(session_response.user_id).await?;

        Ok(AuthResponse {
            success: true,
            session_token: session_response.session_token,
            session_id: session_response.session_id.to_string(),
            user_id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            expires_in: session_response.expires_in,
            refresh_token: Some(session_response.refresh_token),
        })
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<AuthContext> {
        // Validate JWT token
        let claims = self.jwt_service.validate_token(session_token)?;

        // Check if session is still active
        let session_id = Uuid::parse_str(&claims.jti)
            .map_err(|_| AuthError::InvalidToken)?;

        let session = self.session_service.get_session(session_id).await?;
        if !session.is_active || session.is_expired() {
            return Err(AuthError::SessionNotFound);
        }

        // Update session activity
        self.session_service.update_session_activity(session_id).await?;

        // Create auth context
        AuthContext::from_claims(&claims)
            .map_err(|_| AuthError::InvalidToken)
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // Verify current password
        if !self.user_service.verify_password(user_id, current_password).await? {
            return Err(AuthError::InvalidCredentials);
        }

        // Update password
        let update_request = crate::UpdateUserRequest {
            password: Some(new_password.to_string()),
            ..Default::default()
        };

        self.user_service.update_user(user_id, update_request, Some(user_id)).await?;

        // Deactivate all sessions for this user (force re-login)
        self.session_service.deactivate_user_sessions(user_id).await?;

        Ok(())
    }

    pub async fn get_user_profile(&self, user_id: Uuid) -> Result<crate::UserProfile> {
        self.user_service.get_user_profile(user_id).await
    }
}

impl Default for crate::UpdateUserRequest {
    fn default() -> Self {
        Self {
            email: None,
            username: None,
            password: None,
            first_name: None,
            last_name: None,
            is_active: None,
            is_verified: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthConfig;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // Create tables (simplified for testing)
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
        .execute(&pool)
        .await
        .unwrap();

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
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_register_and_login() {
        let db = setup_test_db().await;
        let config = AuthConfig::default();
        let auth_service = AuthService::new(db, config);

        // Register user
        let register_request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: Some("testuser".to_string()),
            password: "TestPassword123!".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
        };

        let register_response = auth_service.register(register_request).await.unwrap();
        assert!(register_response.success);
        assert!(!register_response.session_token.is_empty());

        // Login with registered credentials
        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "TestPassword123!".to_string(),
            remember_me: Some(false),
        };

        let login_response = auth_service.login(login_request, None, None).await.unwrap();
        assert!(login_response.success);
        assert_eq!(login_response.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_invalid_login() {
        let db = setup_test_db().await;
        let config = AuthConfig::default();
        let auth_service = AuthService::new(db, config);

        let login_request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "wrongpassword".to_string(),
            remember_me: Some(false),
        };

        let result = auth_service.login(login_request, None, None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::UserNotFound));
    }
}