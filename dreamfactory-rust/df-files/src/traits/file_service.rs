use crate::models::{
    error::{FileResult, FileServiceError},
    file_info::{DirectoryListing, FileInfo},
    file_operation::{
        BatchOperation, BatchOperationResult, CopyOptions, DownloadOptions, FileOperationResult,
        FileStream, ListOptions, MultipartUpload, UploadOptions, UploadPart,
    },
};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;

/// Core file service trait that all storage providers must implement
#[async_trait]
pub trait FileService: Send + Sync {
    /// Get service information
    fn service_info(&self) -> FileServiceInfo;

    /// Check if the service is healthy and accessible
    async fn health_check(&self) -> FileResult<()>;

    // File Operations
    
    /// Upload a file from bytes
    async fn upload_bytes(
        &self,
        path: &str,
        content: Bytes,
        options: UploadOptions,
    ) -> FileResult<FileOperationResult>;

    /// Upload a file from a stream
    async fn upload_stream(
        &self,
        path: &str,
        stream: FileStream,
        size: Option<u64>,
        options: UploadOptions,
    ) -> FileResult<FileOperationResult>;

    /// Download a file as bytes
    async fn download_bytes(
        &self,
        path: &str,
        options: DownloadOptions,
    ) -> FileResult<(Bytes, FileInfo)>;

    /// Download a file as a stream
    async fn download_stream(
        &self,
        path: &str,
        options: DownloadOptions,
    ) -> FileResult<(FileStream, FileInfo)>;

    /// Get file metadata without downloading content
    async fn get_file_info(&self, path: &str) -> FileResult<FileInfo>;

    /// Check if a file exists
    async fn exists(&self, path: &str) -> FileResult<bool>;

    /// Delete a file
    async fn delete_file(&self, path: &str) -> FileResult<FileOperationResult>;

    /// Copy a file
    async fn copy_file(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult>;

    /// Move/rename a file
    async fn move_file(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult>;

    // Directory Operations

    /// Create a directory
    async fn create_directory(&self, path: &str) -> FileResult<FileOperationResult>;

    /// List directory contents
    async fn list_directory(
        &self,
        path: &str,
        options: ListOptions,
    ) -> FileResult<DirectoryListing>;

    /// Delete a directory (and optionally its contents)
    async fn delete_directory(
        &self,
        path: &str,
        recursive: bool,
    ) -> FileResult<FileOperationResult>;

    /// Copy a directory
    async fn copy_directory(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult>;

    /// Move/rename a directory
    async fn move_directory(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult>;

    // Multipart Upload Operations

    /// Initiate a multipart upload
    async fn initiate_multipart_upload(
        &self,
        path: &str,
        options: UploadOptions,
    ) -> FileResult<MultipartUpload>;

    /// Upload a part of a multipart upload
    async fn upload_part(
        &self,
        upload_id: &str,
        part_number: u32,
        content: Bytes,
    ) -> FileResult<UploadPart>;

    /// Complete a multipart upload
    async fn complete_multipart_upload(
        &self,
        upload_id: &str,
        parts: Vec<UploadPart>,
    ) -> FileResult<FileOperationResult>;

    /// Abort a multipart upload
    async fn abort_multipart_upload(&self, upload_id: &str) -> FileResult<FileOperationResult>;

    /// List active multipart uploads
    async fn list_multipart_uploads(&self) -> FileResult<Vec<MultipartUpload>>;

    // Batch Operations

    /// Execute multiple file operations in a batch
    async fn batch_operation(&self, operation: BatchOperation) -> FileResult<BatchOperationResult>;

    // Utility Operations

    /// Get storage usage statistics
    async fn get_storage_usage(&self, path: Option<&str>) -> FileResult<StorageUsage>;

    /// Search for files matching criteria
    async fn search_files(&self, criteria: SearchCriteria) -> FileResult<Vec<FileInfo>>;

    /// Generate a pre-signed URL for file access (if supported)
    async fn generate_presigned_url(
        &self,
        path: &str,
        operation: PresignedOperation,
        expires_in_seconds: u64,
    ) -> FileResult<String>;

    /// Set file metadata
    async fn set_metadata(
        &self,
        path: &str,
        metadata: HashMap<String, String>,
    ) -> FileResult<FileOperationResult>;

    /// Get file metadata
    async fn get_metadata(&self, path: &str) -> FileResult<HashMap<String, String>>;
}

/// File service information
#[derive(Debug, Clone)]
pub struct FileServiceInfo {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Storage provider type
    pub provider: String,
    /// Supported features
    pub features: Vec<ServiceFeature>,
    /// Service limits
    pub limits: ServiceLimits,
}

/// Service features that may or may not be supported
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceFeature {
    MultipartUpload,
    Streaming,
    DirectoryOperations,
    Metadata,
    Permissions,
    Encryption,
    Compression,
    Versioning,
    PresignedUrls,
    BatchOperations,
    Search,
    Checksums,
}

/// Service limits and constraints
#[derive(Debug, Clone)]
pub struct ServiceLimits {
    /// Maximum file size in bytes
    pub max_file_size: Option<u64>,
    /// Maximum number of files per directory
    pub max_files_per_directory: Option<u64>,
    /// Maximum directory depth
    pub max_directory_depth: Option<u32>,
    /// Maximum path length
    pub max_path_length: Option<usize>,
    /// Maximum metadata size
    pub max_metadata_size: Option<usize>,
    /// Rate limits (operations per second)
    pub rate_limits: Option<HashMap<String, u32>>,
}

/// Storage usage information
#[derive(Debug, Clone)]
pub struct StorageUsage {
    /// Total storage space in bytes
    pub total_bytes: Option<u64>,
    /// Used storage space in bytes
    pub used_bytes: u64,
    /// Available storage space in bytes
    pub available_bytes: Option<u64>,
    /// Number of files
    pub file_count: u64,
    /// Number of directories
    pub directory_count: u64,
}

/// File search criteria
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    /// Search path (starting point)
    pub path: String,
    /// Search recursively
    pub recursive: bool,
    /// File name pattern (glob-style)
    pub name_pattern: Option<String>,
    /// Content type pattern
    pub content_type_pattern: Option<String>,
    /// Minimum file size
    pub min_size: Option<u64>,
    /// Maximum file size
    pub max_size: Option<u64>,
    /// Modified after date
    pub modified_after: Option<chrono::DateTime<chrono::Utc>>,
    /// Modified before date
    pub modified_before: Option<chrono::DateTime<chrono::Utc>>,
    /// Metadata filters
    pub metadata_filters: HashMap<String, String>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Pre-signed URL operations
#[derive(Debug, Clone, PartialEq)]
pub enum PresignedOperation {
    Read,
    Write,
    Delete,
}

/// Trait for storage providers that support streaming
#[async_trait]
pub trait StreamingFileService: FileService {
    /// Create a resumable upload session
    async fn create_resumable_upload(&self, path: &str) -> FileResult<String>;

    /// Resume an upload session
    async fn resume_upload(
        &self,
        session_id: &str,
        offset: u64,
        content: Bytes,
    ) -> FileResult<FileOperationResult>;

    /// Get upload session status
    async fn get_upload_status(&self, session_id: &str) -> FileResult<UploadStatus>;
}

/// Upload session status
#[derive(Debug, Clone)]
pub struct UploadStatus {
    /// Session ID
    pub session_id: String,
    /// File path
    pub path: String,
    /// Total file size (if known)
    pub total_size: Option<u64>,
    /// Bytes uploaded so far
    pub uploaded_bytes: u64,
    /// Session expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Session status
    pub status: UploadSessionStatus,
}

/// Upload session status
#[derive(Debug, Clone, PartialEq)]
pub enum UploadSessionStatus {
    Active,
    Completed,
    Expired,
    Cancelled,
    Failed,
}

/// Trait for storage providers that support advanced features
#[async_trait]
pub trait AdvancedFileService: FileService {
    /// Create a snapshot/version of a file
    async fn create_snapshot(&self, path: &str) -> FileResult<String>;

    /// List snapshots/versions of a file
    async fn list_snapshots(&self, path: &str) -> FileResult<Vec<FileSnapshot>>;

    /// Restore a file from a snapshot
    async fn restore_snapshot(&self, path: &str, snapshot_id: &str) -> FileResult<FileOperationResult>;

    /// Delete a snapshot
    async fn delete_snapshot(&self, path: &str, snapshot_id: &str) -> FileResult<FileOperationResult>;

    /// Lock a file for exclusive access
    async fn lock_file(&self, path: &str, duration_seconds: u64) -> FileResult<String>;

    /// Unlock a file
    async fn unlock_file(&self, path: &str, lock_id: &str) -> FileResult<FileOperationResult>;

    /// Check if a file is locked
    async fn is_file_locked(&self, path: &str) -> FileResult<Option<FileLock>>;
}

/// File snapshot information
#[derive(Debug, Clone)]
pub struct FileSnapshot {
    /// Snapshot ID
    pub id: String,
    /// File path
    pub path: String,
    /// Snapshot creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// File size at time of snapshot
    pub size: u64,
    /// Snapshot metadata
    pub metadata: HashMap<String, String>,
}

/// File lock information
#[derive(Debug, Clone)]
pub struct FileLock {
    /// Lock ID
    pub id: String,
    /// File path
    pub path: String,
    /// Lock owner
    pub owner: String,
    /// Lock acquired time
    pub acquired_at: chrono::DateTime<chrono::Utc>,
    /// Lock expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info() {
        let info = FileServiceInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            provider: "local".to_string(),
            features: vec![ServiceFeature::MultipartUpload, ServiceFeature::Streaming],
            limits: ServiceLimits {
                max_file_size: Some(1024 * 1024 * 1024),
                max_files_per_directory: None,
                max_directory_depth: Some(20),
                max_path_length: Some(1024),
                max_metadata_size: Some(4096),
                rate_limits: None,
            },
        };

        assert_eq!(info.name, "test");
        assert!(info.features.contains(&ServiceFeature::MultipartUpload));
        assert_eq!(info.limits.max_file_size, Some(1024 * 1024 * 1024));
    }

    #[test]
    fn test_storage_usage() {
        let usage = StorageUsage {
            total_bytes: Some(1024 * 1024 * 1024),
            used_bytes: 512 * 1024 * 1024,
            available_bytes: Some(512 * 1024 * 1024),
            file_count: 100,
            directory_count: 10,
        };

        assert_eq!(usage.total_bytes, Some(1024 * 1024 * 1024));
        assert_eq!(usage.used_bytes, 512 * 1024 * 1024);
    }

    #[test]
    fn test_search_criteria() {
        let criteria = SearchCriteria {
            path: "/documents".to_string(),
            recursive: true,
            name_pattern: Some("*.pdf".to_string()),
            content_type_pattern: Some("application/pdf".to_string()),
            min_size: Some(1024),
            max_size: Some(10 * 1024 * 1024),
            modified_after: None,
            modified_before: None,
            metadata_filters: HashMap::new(),
            limit: Some(100),
        };

        assert_eq!(criteria.path, "/documents");
        assert!(criteria.recursive);
        assert_eq!(criteria.name_pattern.as_ref().unwrap(), "*.pdf");
    }

    #[test]
    fn test_upload_status() {
        let status = UploadStatus {
            session_id: "session123".to_string(),
            path: "large_file.bin".to_string(),
            total_size: Some(1024 * 1024),
            uploaded_bytes: 512 * 1024,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            status: UploadSessionStatus::Active,
        };

        assert_eq!(status.session_id, "session123");
        assert_eq!(status.status, UploadSessionStatus::Active);
        assert_eq!(status.uploaded_bytes, 512 * 1024);
    }
}