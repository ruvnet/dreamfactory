use thiserror::Error;

#[derive(Error, Debug)]
pub enum DreamFactoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Authentication error: {0}")] 
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Internal error: {0}")]
    Internal(String),
}