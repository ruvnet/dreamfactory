//! # DreamFactory Files Module
//! 
//! This module provides comprehensive file services for DreamFactory, including:
//! - Local filesystem operations
//! - Cloud storage integration (S3, Azure, Google Cloud)
//! - File upload/download with streaming support
//! - Directory operations and listing
//! - File metadata management
//! - Multipart upload support
//! - Security and permission handling

pub mod models;
pub mod traits;
pub mod services;
pub mod storage;
pub mod handlers;
pub mod utils;

// Re-export commonly used types
pub use models::{
    FileInfo, FileServiceError, FileResult,
    FileServiceConfig, StorageProvider as StorageProviderType,
    UploadOptions, DownloadOptions, CopyOptions, ListOptions,
    FileOperationResult, BatchOperation, BatchOperationResult,
    MultipartUpload, UploadPart
};
pub use traits::{
    FileService, StorageProvider, ProviderCapabilities, ProviderHealth, ProviderStatistics,
    StorageUsage, FileServiceInfo
};
pub use services::FileServiceFactory;

// Re-export storage providers specifically
pub use storage::{
    local::LocalStorageProvider,
    s3::S3StorageProvider,
    azure::AzureBlobStorageProvider,
    google_cloud::GoogleCloudStorageProvider,
};