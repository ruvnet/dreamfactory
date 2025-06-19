use thiserror::Error;

/// Main error type for the DreamFactory framework
#[derive(Error, Debug)]
pub enum DfError {
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

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Service error in '{service}': {message}")]
    Service { service: String, message: String },

    #[error("Registry error: {message}")]
    Registry { message: String },

    #[error("Plugin error in '{plugin}': {message}")]
    Plugin { plugin: String, message: String },
}

impl DfError {
    /// Create a service-specific error
    pub fn service<S: Into<String>>(service: &str, message: S) -> Self {
        Self::Service {
            service: service.to_string(),
            message: message.into(),
        }
    }

    /// Create a registry error
    pub fn registry<S: Into<String>>(message: S) -> Self {
        Self::Registry {
            message: message.into(),
        }
    }

    /// Create a plugin-specific error
    pub fn plugin<S: Into<String>>(plugin: &str, message: S) -> Self {
        Self::Plugin {
            plugin: plugin.to_string(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
}

/// Result type alias for DreamFactory operations
pub type DfResult<T> = Result<T, DfError>;

