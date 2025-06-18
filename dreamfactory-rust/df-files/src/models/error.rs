use thiserror::Error;

/// File service error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum FileServiceError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("File already exists: {path}")]
    FileAlreadyExists { path: String },

    #[error("Directory already exists: {path}")]
    DirectoryAlreadyExists { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Invalid file path: {path}")]
    InvalidPath { path: String },

    #[error("File too large: {size} bytes, maximum allowed: {max_size} bytes")]
    FileTooLarge { size: u64, max_size: u64 },

    #[error("Insufficient storage space: required {required}, available {available}")]
    InsufficientSpace { required: u64, available: u64 },

    #[error("Unsupported file type: {mime_type}")]
    UnsupportedFileType { mime_type: String },

    #[error("Invalid multipart upload: {reason}")]
    InvalidMultipartUpload { reason: String },

    #[error("Upload incomplete: part {part} missing")]
    IncompleteUpload { part: u32 },

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Cloud storage error: {provider} - {message}")]
    CloudStorageError { provider: String, message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },

    #[error("Authorization failed: {path}")]
    AuthorizationError { path: String },

    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Timeout occurred during operation: {operation}")]
    TimeoutError { operation: String },

    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl FileServiceError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            FileServiceError::FileNotFound { .. } => 404,
            FileServiceError::DirectoryNotFound { .. } => 404,
            FileServiceError::FileAlreadyExists { .. } => 409,
            FileServiceError::DirectoryAlreadyExists { .. } => 409,
            FileServiceError::PermissionDenied { .. } => 403,
            FileServiceError::InvalidPath { .. } => 400,
            FileServiceError::FileTooLarge { .. } => 413,
            FileServiceError::InsufficientSpace { .. } => 507,
            FileServiceError::UnsupportedFileType { .. } => 415,
            FileServiceError::InvalidMultipartUpload { .. } => 400,
            FileServiceError::IncompleteUpload { .. } => 400,
            FileServiceError::ChecksumMismatch { .. } => 400,
            FileServiceError::AuthenticationError { .. } => 401,
            FileServiceError::AuthorizationError { .. } => 403,
            FileServiceError::ServiceUnavailable { .. } => 503,
            FileServiceError::TimeoutError { .. } => 408,
            FileServiceError::CloudStorageError { .. } => 502,
            FileServiceError::NetworkError { .. } => 502,
            FileServiceError::ConfigError { .. } => 500,
            FileServiceError::IoError { .. } => 500,
            FileServiceError::SerializationError { .. } => 500,
            FileServiceError::InternalError { .. } => 500,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            FileServiceError::NetworkError { .. }
                | FileServiceError::ServiceUnavailable { .. }
                | FileServiceError::TimeoutError { .. }
                | FileServiceError::InsufficientSpace { .. }
        )
    }

    /// Check if this error should be retried
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            FileServiceError::NetworkError { .. }
                | FileServiceError::ServiceUnavailable { .. }
                | FileServiceError::TimeoutError { .. }
        )
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for FileServiceError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => FileServiceError::FileNotFound {
                path: "unknown".to_string(),
            },
            std::io::ErrorKind::PermissionDenied => FileServiceError::PermissionDenied {
                path: "unknown".to_string(),
            },
            std::io::ErrorKind::AlreadyExists => FileServiceError::FileAlreadyExists {
                path: "unknown".to_string(),
            },
            _ => FileServiceError::IoError {
                message: error.to_string(),
            },
        }
    }
}

/// Convert from object_store::Error
impl From<object_store::Error> for FileServiceError {
    fn from(error: object_store::Error) -> Self {
        match error {
            object_store::Error::NotFound { path, .. } => FileServiceError::FileNotFound {
                path: path.to_string(),
            },
            object_store::Error::AlreadyExists { path, .. } => FileServiceError::FileAlreadyExists {
                path: path.to_string(),
            },
            object_store::Error::Unauthenticated { .. } => FileServiceError::AuthenticationError {
                message: error.to_string(),
            },
            _ => FileServiceError::CloudStorageError {
                provider: "object_store".to_string(),
                message: error.to_string(),
            },
        }
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for FileServiceError {
    fn from(error: serde_json::Error) -> Self {
        FileServiceError::SerializationError {
            message: error.to_string(),
        }
    }
}

/// Result type for file operations
pub type FileResult<T> = Result<T, FileServiceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            FileServiceError::FileNotFound {
                path: "test".to_string()
            }
            .status_code(),
            404
        );
        assert_eq!(
            FileServiceError::PermissionDenied {
                path: "test".to_string()
            }
            .status_code(),
            403
        );
        assert_eq!(
            FileServiceError::FileTooLarge {
                size: 1000,
                max_size: 500
            }
            .status_code(),
            413
        );
    }

    #[test]
    fn test_error_recoverability() {
        assert!(FileServiceError::NetworkError {
            message: "test".to_string()
        }
        .is_recoverable());
        assert!(!FileServiceError::FileNotFound {
            path: "test".to_string()
        }
        .is_recoverable());
    }

    #[test]
    fn test_error_retry_logic() {
        assert!(FileServiceError::ServiceUnavailable {
            service: "S3".to_string()
        }
        .should_retry());
        assert!(!FileServiceError::PermissionDenied {
            path: "test".to_string()
        }
        .should_retry());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let file_error: FileServiceError = io_error.into();
        assert!(matches!(file_error, FileServiceError::FileNotFound { .. }));
    }
}