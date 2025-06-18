use crate::models::error::FileResult;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

/// Stream type for file content
pub type FileStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

/// File upload options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadOptions {
    /// Content type override
    pub content_type: Option<String>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
    /// Whether to overwrite existing files
    pub overwrite: bool,
    /// Whether to create parent directories
    pub create_parents: bool,
    /// File permissions (Unix-style octal)
    pub permissions: Option<u32>,
    /// Checksum to verify
    pub checksum: Option<String>,
    /// Checksum algorithm
    pub checksum_algorithm: Option<String>,
    /// Whether to enable server-side encryption
    pub encrypt: bool,
    /// Compression to apply
    pub compression: Option<String>,
    /// Cache control headers
    pub cache_control: Option<String>,
    /// Content encoding
    pub content_encoding: Option<String>,
    /// Content disposition
    pub content_disposition: Option<String>,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            content_type: None,
            metadata: HashMap::new(),
            overwrite: false,
            create_parents: true,
            permissions: None,
            checksum: None,
            checksum_algorithm: None,
            encrypt: false,
            compression: None,
            cache_control: None,
            content_encoding: None,
            content_disposition: None,
        }
    }
}

impl UploadOptions {
    /// Create new upload options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Enable overwrite
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// Set permissions
    pub fn with_permissions(mut self, permissions: u32) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Set checksum
    pub fn with_checksum(mut self, checksum: String, algorithm: String) -> Self {
        self.checksum = Some(checksum);
        self.checksum_algorithm = Some(algorithm);
        self
    }

    /// Enable encryption
    pub fn with_encryption(mut self, encrypt: bool) -> Self {
        self.encrypt = encrypt;
        self
    }

    /// Set compression
    pub fn with_compression(mut self, compression: String) -> Self {
        self.compression = Some(compression);
        self
    }
}

/// File download options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    /// Byte range to download (start, end)
    pub range: Option<(u64, Option<u64>)>,
    /// Expected ETag for conditional download
    pub if_match: Option<String>,
    /// Expected modified date for conditional download
    pub if_modified_since: Option<DateTime<Utc>>,
    /// Whether to include metadata in response
    pub include_metadata: bool,
    /// Whether to verify checksum
    pub verify_checksum: bool,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            range: None,
            if_match: None,
            if_modified_since: None,
            include_metadata: true,
            verify_checksum: false,
        }
    }
}

impl DownloadOptions {
    /// Create new download options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set byte range
    pub fn with_range(mut self, start: u64, end: Option<u64>) -> Self {
        self.range = Some((start, end));
        self
    }

    /// Set conditional ETag
    pub fn with_if_match(mut self, etag: String) -> Self {
        self.if_match = Some(etag);
        self
    }

    /// Set conditional modified date
    pub fn with_if_modified_since(mut self, date: DateTime<Utc>) -> Self {
        self.if_modified_since = Some(date);
        self
    }

    /// Enable checksum verification
    pub fn with_checksum_verification(mut self, verify: bool) -> Self {
        self.verify_checksum = verify;
        self
    }
}

/// Directory listing options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOptions {
    /// Whether to list recursively
    pub recursive: bool,
    /// Include hidden files (starting with .)
    pub include_hidden: bool,
    /// File pattern to match (glob-style)
    pub pattern: Option<String>,
    /// Maximum number of entries to return
    pub limit: Option<usize>,
    /// Continuation token for pagination
    pub continuation_token: Option<String>,
    /// Sort order
    pub sort_by: SortBy,
    /// Sort direction
    pub sort_order: SortOrder,
    /// Include file metadata
    pub include_metadata: bool,
    /// Include directory sizes (may be expensive)
    pub include_directory_sizes: bool,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            include_hidden: false,
            pattern: None,
            limit: None,
            continuation_token: None,
            sort_by: SortBy::Name,
            sort_order: SortOrder::Ascending,
            include_metadata: true,
            include_directory_sizes: false,
        }
    }
}

/// Sort criteria for directory listings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Created,
    Type,
}

/// Sort order
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Copy/move operation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyOptions {
    /// Whether to overwrite existing files
    pub overwrite: bool,
    /// Whether to preserve metadata
    pub preserve_metadata: bool,
    /// Whether to preserve permissions
    pub preserve_permissions: bool,
    /// Whether to copy recursively (for directories)
    pub recursive: bool,
    /// Whether to create parent directories
    pub create_parents: bool,
    /// Follow symbolic links
    pub follow_symlinks: bool,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            preserve_metadata: true,
            preserve_permissions: true,
            recursive: true,
            create_parents: true,
            follow_symlinks: false,
        }
    }
}

/// Multipart upload information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUpload {
    /// Upload ID
    pub upload_id: String,
    /// File path
    pub path: String,
    /// Upload initiated timestamp
    pub initiated_at: DateTime<Utc>,
    /// Total file size (if known)
    pub total_size: Option<u64>,
    /// Uploaded parts
    pub parts: Vec<UploadPart>,
    /// Upload metadata
    pub metadata: HashMap<String, String>,
}

/// Individual part of a multipart upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPart {
    /// Part number (1-based)
    pub part_number: u32,
    /// Part size in bytes
    pub size: u64,
    /// Part ETag
    pub etag: String,
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
    /// Part checksum
    pub checksum: Option<String>,
}

/// File operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    /// Operation that was performed
    pub operation: String,
    /// File path
    pub path: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional message
    pub message: Option<String>,
    /// Bytes transferred (for upload/download)
    pub bytes_transferred: Option<u64>,
    /// Operation duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl FileOperationResult {
    /// Create a successful operation result
    pub fn success(operation: String, path: String) -> Self {
        Self {
            operation,
            path,
            success: true,
            timestamp: Utc::now(),
            message: None,
            bytes_transferred: None,
            duration_ms: None,
        }
    }

    /// Create a failed operation result
    pub fn failure(operation: String, path: String, message: String) -> Self {
        Self {
            operation,
            path,
            success: false,
            timestamp: Utc::now(),
            message: Some(message),
            bytes_transferred: None,
            duration_ms: None,
        }
    }

    /// Set bytes transferred
    pub fn with_bytes_transferred(mut self, bytes: u64) -> Self {
        self.bytes_transferred = Some(bytes);
        self
    }

    /// Set operation duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Batch operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    /// Operation type
    pub operation: BatchOperationType,
    /// Source paths
    pub sources: Vec<String>,
    /// Destination path (for copy/move operations)
    pub destination: Option<String>,
    /// Operation options
    pub options: HashMap<String, String>,
}

/// Batch operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperationType {
    Delete,
    Copy,
    Move,
    Archive,
    Extract,
}

/// Batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult {
    /// Total operations
    pub total: usize,
    /// Successful operations
    pub successful: usize,
    /// Failed operations
    pub failed: usize,
    /// Individual results
    pub results: Vec<FileOperationResult>,
    /// Overall operation timestamp
    pub timestamp: DateTime<Utc>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_options_builder() {
        let options = UploadOptions::new()
            .with_content_type("text/plain".to_string())
            .with_metadata("author".to_string(), "test".to_string())
            .with_overwrite(true)
            .with_permissions(644)
            .with_encryption(true);

        assert_eq!(options.content_type.as_ref().unwrap(), "text/plain");
        assert_eq!(options.metadata.get("author").unwrap(), "test");
        assert!(options.overwrite);
        assert_eq!(options.permissions, Some(644));
        assert!(options.encrypt);
    }

    #[test]
    fn test_download_options_builder() {
        let options = DownloadOptions::new()
            .with_range(0, Some(1023))
            .with_if_match("etag123".to_string())
            .with_checksum_verification(true);

        assert_eq!(options.range, Some((0, Some(1023))));
        assert_eq!(options.if_match.as_ref().unwrap(), "etag123");
        assert!(options.verify_checksum);
    }

    #[test]
    fn test_list_options_defaults() {
        let options = ListOptions::default();
        assert!(!options.recursive);
        assert!(!options.include_hidden);
        assert!(matches!(options.sort_by, SortBy::Name));
        assert!(matches!(options.sort_order, SortOrder::Ascending));
    }

    #[test]
    fn test_file_operation_result() {
        let result = FileOperationResult::success("upload".to_string(), "test.txt".to_string())
            .with_bytes_transferred(1024)
            .with_duration(500);

        assert!(result.success);
        assert_eq!(result.operation, "upload");
        assert_eq!(result.path, "test.txt");
        assert_eq!(result.bytes_transferred, Some(1024));
        assert_eq!(result.duration_ms, Some(500));
    }

    #[test]
    fn test_multipart_upload() {
        let upload = MultipartUpload {
            upload_id: "test123".to_string(),
            path: "large_file.bin".to_string(),
            initiated_at: Utc::now(),
            total_size: Some(10485760), // 10MB
            parts: vec![UploadPart {
                part_number: 1,
                size: 5242880, // 5MB
                etag: "part1_etag".to_string(),
                uploaded_at: Utc::now(),
                checksum: Some("checksum1".to_string()),
            }],
            metadata: HashMap::new(),
        };

        assert_eq!(upload.upload_id, "test123");
        assert_eq!(upload.parts.len(), 1);
        assert_eq!(upload.parts[0].part_number, 1);
    }

    #[test]
    fn test_batch_operation() {
        let batch_op = BatchOperation {
            operation: BatchOperationType::Delete,
            sources: vec!["file1.txt".to_string(), "file2.txt".to_string()],
            destination: None,
            options: HashMap::new(),
        };

        assert!(matches!(batch_op.operation, BatchOperationType::Delete));
        assert_eq!(batch_op.sources.len(), 2);
        assert!(batch_op.destination.is_none());
    }

    #[test]
    fn test_copy_options() {
        let options = CopyOptions::default();
        assert!(!options.overwrite);
        assert!(options.preserve_metadata);
        assert!(options.recursive);
        assert!(options.create_parents);
    }
}