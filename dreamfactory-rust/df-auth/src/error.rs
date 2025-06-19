use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Password hashing error: {0}")]
    PasswordHash(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Access denied: insufficient permissions")]
    Forbidden,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("API key not found")]
    ApiKeyNotFound,

    #[error("API key expired")]
    ApiKeyExpired,

    #[error("Role not found")]
    RoleNotFound,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AuthError::PasswordHash(format!("{:?}", err))
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::Unauthorized | AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Authentication failed")
            }
            AuthError::Forbidden | AuthError::PermissionDenied => {
                (StatusCode::FORBIDDEN, "Access denied")
            }
            AuthError::UserNotFound | AuthError::SessionNotFound | AuthError::ApiKeyNotFound => {
                (StatusCode::NOT_FOUND, "Resource not found")
            }
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthError::Validation(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            AuthError::InvalidToken | AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token")
            }
            AuthError::ApiKeyExpired => (StatusCode::UNAUTHORIZED, "API key expired"),
            AuthError::RoleNotFound => (StatusCode::NOT_FOUND, "Role not found"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": error_message,
                "details": self.to_string()
            }
        }));

        (status, body).into_response()
    }
}