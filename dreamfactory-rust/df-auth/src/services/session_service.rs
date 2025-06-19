use crate::{
    AuthError, Result, Session, SessionResponse, JwtService, AuthConfig
};
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionService {
    db: Pool<Sqlite>,
    jwt_service: JwtService,
    config: AuthConfig,
}

impl SessionService {
    pub fn new(db: Pool<Sqlite>, config: AuthConfig) -> Self {
        let jwt_service = JwtService::new(&config);
        Self {
            db,
            jwt_service,
            config,
        }
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        roles: Vec<String>,
        permissions: Vec<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        remember_me: bool,
    ) -> Result<SessionResponse> {
        let session_id = Uuid::new_v4();
        
        // Generate JWT token
        let token = self.jwt_service.generate_token(
            user_id,
            "".to_string(), // Email will be filled from user data
            session_id,
            roles,
            permissions,
        )?;

        // Generate refresh token
        let refresh_token = self.generate_refresh_token();
        
        // Determine expiration times
        let jwt_expiration = self.config.jwt_expiration;
        let refresh_expiration = if remember_me {
            self.config.refresh_token_expiration
        } else {
            self.config.session_timeout
        };

        // Create session record
        let session = Session::new(
            user_id,
            token.clone(),
            refresh_token.clone(),
            jwt_expiration,
            refresh_expiration,
            ip_address,
            user_agent,
        );

        // Insert into database
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, user_id, token, refresh_token, expires_at, refresh_expires_at,
                is_active, ip_address, user_agent, created_date, last_activity
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(session.id)
        .bind(session.user_id)
        .bind(&session.token)
        .bind(&session.refresh_token)
        .bind(session.expires_at)
        .bind(session.refresh_expires_at)
        .bind(session.is_active)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(session.created_date)
        .bind(session.last_activity)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        // Get user email for response
        let user_email: (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT email, first_name, last_name FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(SessionResponse {
            session_token: token,
            session_id,
            user_id,
            email: user_email.0,
            first_name: user_email.1,
            last_name: user_email.2,
            expires_in: jwt_expiration,
            refresh_token,
        })
    }

    pub async fn get_session(&self, session_id: Uuid) -> Result<Session> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::SessionNotFound)?;

        Ok(session)
    }

    pub async fn update_session_activity(&self, session_id: Uuid) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE sessions SET last_activity = ? WHERE id = ?"
        )
        .bind(now)
        .bind(session_id)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn deactivate_session(&self, session_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE sessions SET is_active = FALSE WHERE id = ?"
        )
        .bind(session_id)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn deactivate_user_sessions(&self, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE sessions SET is_active = FALSE WHERE user_id = ?"
        )
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn refresh_session(&self, refresh_token: &str) -> Result<SessionResponse> {
        // Find session by refresh token
        let mut session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE refresh_token = ? AND is_active = TRUE"
        )
        .bind(refresh_token)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::SessionNotFound)?;

        // Check if refresh token is expired
        if session.is_refresh_expired() {
            self.deactivate_session(session.id).await?;
            return Err(AuthError::TokenExpired);
        }

        // Get user info and permissions
        let user: (String,) = sqlx::query_as(
            "SELECT email FROM users WHERE id = ?"
        )
        .bind(session.user_id)
        .fetch_one(&self.db)
        .await
        .map_err(AuthError::Database)?;

        let roles = self.get_user_roles(session.user_id).await?;
        let permissions = self.get_user_permissions(session.user_id).await?;

        // Generate new JWT token
        let new_token = self.jwt_service.generate_token(
            session.user_id,
            user.0.clone(),
            session.id,
            roles,
            permissions,
        )?;

        // Update session with new token and extend expiration
        let now = Utc::now();
        session.token = new_token.clone();
        session.expires_at = now + chrono::Duration::seconds(self.config.jwt_expiration);
        session.last_activity = now;

        sqlx::query(
            "UPDATE sessions SET token = ?, expires_at = ?, last_activity = ? WHERE id = ?"
        )
        .bind(&session.token)
        .bind(session.expires_at)
        .bind(session.last_activity)
        .bind(session.id)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        // Get user details for response
        let user_details: (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT email, first_name, last_name FROM users WHERE id = ?"
        )
        .bind(session.user_id)
        .fetch_one(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(SessionResponse {
            session_token: new_token,
            session_id: session.id,
            user_id: session.user_id,
            email: user_details.0,
            first_name: user_details.1,
            last_name: user_details.2,
            expires_in: self.config.jwt_expiration,
            refresh_token: session.refresh_token,
        })
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let now = Utc::now();
        let result = sqlx::query(
            "DELETE FROM sessions WHERE expires_at < ? OR refresh_expires_at < ?"
        )
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(result.rows_affected())
    }

    pub async fn get_active_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE user_id = ? AND is_active = TRUE ORDER BY last_activity DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(sessions)
    }

    fn generate_refresh_token(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                 abcdefghijklmnopqrstuvwxyz\
                                 0123456789";
        
        let mut rng = rand::thread_rng();
        (0..64) // Longer refresh token
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT r.name
            FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = ? AND r.is_active = TRUE
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(rows.into_iter().map(|row| row.0).collect())
    }

    async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT p.resource || '.' || p.action as permission
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = ? AND p.is_active = TRUE
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(rows.into_iter().map(|row| row.0).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthConfig;

    #[tokio::test]
    async fn test_generate_refresh_token() {
        let db = sqlx::SqlitePool::connect(":memory:").await.unwrap();
        let config = AuthConfig::default();
        let session_service = SessionService::new(db, config);
        
        let token1 = session_service.generate_refresh_token();
        let token2 = session_service.generate_refresh_token();
        
        assert_eq!(token1.len(), 64);
        assert_eq!(token2.len(), 64);
        assert_ne!(token1, token2); // Should be different
    }
}