use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64, // seconds
    pub refresh_token_expiration: i64, // seconds
    pub database_url: String,
    pub password_pepper: String,
    pub api_key_length: usize,
    pub session_timeout: i64, // seconds
    pub max_login_attempts: u32,
    pub lockout_duration: i64, // seconds
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_change_me".to_string()),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            refresh_token_expiration: env::var("REFRESH_TOKEN_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .unwrap_or(604800),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:dreamfactory.db".to_string()),
            password_pepper: env::var("PASSWORD_PEPPER")
                .unwrap_or_else(|_| "dreamfactory_pepper".to_string()),
            api_key_length: env::var("API_KEY_LENGTH")
                .unwrap_or_else(|_| "32".to_string())
                .parse()
                .unwrap_or(32),
            session_timeout: env::var("SESSION_TIMEOUT")
                .unwrap_or_else(|_| "1800".to_string()) // 30 minutes
                .parse()
                .unwrap_or(1800),
            max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            lockout_duration: env::var("LOCKOUT_DURATION")
                .unwrap_or_else(|_| "900".to_string()) // 15 minutes
                .parse()
                .unwrap_or(900),
        }
    }
}

impl AuthConfig {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Self::default()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters long".to_string());
        }
        
        if self.jwt_expiration <= 0 {
            return Err("JWT expiration must be positive".to_string());
        }
        
        if self.password_pepper.is_empty() {
            return Err("Password pepper cannot be empty".to_string());
        }
        
        Ok(())
    }
}