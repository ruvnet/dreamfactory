use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_date: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_token: String,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub expires_in: i64,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

impl Session {
    pub fn new(
        user_id: Uuid,
        token: String,
        refresh_token: String,
        jwt_expiration: i64,
        refresh_expiration: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            token,
            refresh_token,
            expires_at: now + chrono::Duration::seconds(jwt_expiration),
            refresh_expires_at: now + chrono::Duration::seconds(refresh_expiration),
            is_active: true,
            ip_address,
            user_agent,
            created_date: now,
            last_activity: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_refresh_expired(&self) -> bool {
        Utc::now() > self.refresh_expires_at
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}