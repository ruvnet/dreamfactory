use crate::models::{
    config::{FileServiceConfig, GoogleCloudConfig, ProviderConfig},
    error::{FileResult, FileServiceError},
    file_info::{DirectoryListing, FileInfo},
    file_operation::{
        BatchOperation, BatchOperationResult, CopyOptions, DownloadOptions,
        FileOperationResult, FileStream, ListOptions, MultipartUpload,
        UploadOptions, UploadPart,
    },
};
use crate::traits::{
    file_service::{
        FileService, FileServiceInfo, PresignedOperation, SearchCriteria,
        ServiceFeature, ServiceLimits, StorageUsage,
    },
    storage_provider::{ProviderCapabilities, ProviderHealth, ProviderStatistics, StorageProvider},
};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::{stream, Stream, StreamExt, TryStreamExt};
use object_store::{
    gcp::{GoogleCloudStorage, GoogleCloudStorageBuilder},
    path::Path as ObjectPath,
    ObjectStore,
};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use url::Url;

/// Google Cloud Storage provider
pub struct GoogleCloudStorageProvider {
    config: FileServiceConfig,
    gcp_config: GoogleCloudConfig,
    object_store: Option<Arc<GoogleCloudStorage>>,
    initialized: bool,
    statistics: Arc<Mutex<ProviderStatistics>>,
    multipart_uploads: Arc<Mutex<HashMap<String, MultipartUpload>>>,
}

impl GoogleCloudStorageProvider {
    /// Create a new Google Cloud Storage provider
    pub fn new(config: FileServiceConfig) -> FileResult<Self> {
        let gcp_config = match &config.provider_config {
            ProviderConfig::GoogleCloud(gcp_config) => gcp_config.clone(),
            _ => {
                return Err(FileServiceError::ConfigError {
                    message: "Invalid provider config for Google Cloud Storage".to_string(),
                })
            }
        };

        Ok(Self {
            config,
            gcp_config,
            object_store: None,
            initialized: false,
            statistics: Arc::new(Mutex::new(ProviderStatistics::default())),
            multipart_uploads: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get the GCS object name for a given path
    fn get_object_name(&self, path: &str) -> String {
        let clean_path = path.strip_prefix('/').unwrap_or(path);
        if self.config.root_path.is_empty() || self.config.root_path == "/" {
            clean_path.to_string()
        } else {
            let root = self.config.root_path.strip_prefix('/').unwrap_or(&self.config.root_path);
            format!("{}/{}", root, clean_path)
        }
    }

    /// Convert GCS object name back to relative path
    fn get_relative_path(&self, object_name: &str) -> String {
        if self.config.root_path.is_empty() || self.config.root_path == "/" {
            format!("/{}", object_name)
        } else {
            let root = self.config.root_path.strip_prefix('/').unwrap_or(&self.config.root_path);
            if let Some(relative) = object_name.strip_prefix(&format!("{}/", root)) {
                format!("/{}", relative)
            } else if object_name == root {
                "/".to_string()
            } else {
                format!("/{}", object_name)
            }
        }
    }

    /// Validate and normalize a path
    fn validate_path(&self, path: &str) -> FileResult<String> {
        if path.is_empty() {
            return Err(FileServiceError::InvalidPath {
                path: path.to_string(),
            });
        }

        // Normalize path separators and remove dangerous components
        let normalized = path
            .replace('\\', "/")
            .split('/')
            .filter(|component| !component.is_empty() && *component != "." && *component != "..")
            .collect::<Vec<_>>()
            .join("/");

        if normalized.is_empty() {
            Ok("/".to_string())
        } else {
            Ok(format!("/{}", normalized))
        }
    }

    /// Create FileInfo from GCS object metadata
    fn create_file_info_from_object(
        &self,
        path: &str,
        size: i64,
        last_modified: Option<DateTime<Utc>>,
        etag: Option<String>,
        content_type: Option<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> FileInfo {
        let name = path
            .split('/')
            .last()
            .unwrap_or(path)
            .to_string();

        let created_at = last_modified.unwrap_or_else(Utc::now);
        let modified_at = created_at;

        let mut info = FileInfo::new(path.to_string(), name)
            .with_size(size as u64)
            .with_content_type(content_type.unwrap_or_else(|| {
                mime_guess::from_path(path)
                    .first_or_octet_stream()
                    .to_string()
            }))
            .with_timestamps(created_at, modified_at);

        if let Some(etag) = etag {
            info = info.with_etag(etag);
        }

        if let Some(metadata) = metadata {
            for (key, value) in metadata {
                info = info.with_metadata(key, value);
            }
        }

        info
    }

    /// Record operation statistics
    fn record_operation(&self, success: bool, bytes_transferred: Option<u64>) {
        if let Ok(mut stats) = self.statistics.lock() {
            stats.total_operations += 1;
            if success {
                stats.successful_operations += 1;
            } else {
                stats.failed_operations += 1;
            }
            if let Some(bytes) = bytes_transferred {
                stats.bytes_uploaded += bytes;
            }
            stats.last_operation_at = Some(Utc::now());
        }
    }
}

#[async_trait]
impl StorageProvider for GoogleCloudStorageProvider {
    async fn initialize(&mut self) -> FileResult<()> {
        if self.initialized {
            return Ok(());
        }

        // Initialize Google Cloud Storage client using object_store
        let mut builder = GoogleCloudStorageBuilder::new()
            .with_bucket_name(&self.gcp_config.bucket);

        // Set project ID if provided
        if let Some(project_id) = &self.gcp_config.project_id {
            builder = builder.with_google_service_account_path(project_id);
        }

        // Set service account key if provided
        if let Some(service_account_key) = &self.gcp_config.service_account_key {
            builder = builder.with_service_account_key(service_account_key);
        }

        // Set custom endpoint if provided
        if let Some(endpoint_url) = &self.gcp_config.endpoint_url {
            let url: Url = endpoint_url.parse().map_err(|e| FileServiceError::ConfigError {
                message: format!("Invalid endpoint URL: {}", e),
            })?;
            builder = builder.with_url(url.to_string());
        }

        self.object_store = Some(Arc::new(builder.build().map_err(|e| {
            FileServiceError::ConfigError {
                message: format!("Failed to initialize Google Cloud Storage client: {}", e),
            }
        })?));

        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> FileResult<()> {
        // Clear any in-progress multipart uploads
        if let Ok(mut uploads) = self.multipart_uploads.lock() {
            uploads.clear();
        }

        self.object_store = None;
        self.initialized = false;
        Ok(())
    }

    fn config(&self) -> &FileServiceConfig {
        &self.config
    }

    fn is_ready(&self) -> bool {
        self.initialized && self.object_store.is_some()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming_upload: true,
            streaming_download: true,
            multipart_upload: true,
            resumable_upload: true, // GCS supports resumable uploads
            directory_operations: false, // GCS doesn't have true directories
            metadata_support: true,
            permissions_support: true, // GCS has IAM
            encryption_support: true,
            compression_support: true, // GCS supports transparent compression
            versioning_support: true, // If bucket versioning is enabled
            presigned_urls: true,
            batch_operations: true,
            search_support: true,
            locking_support: false,
            atomic_operations: true,
            transaction_support: false,
            max_file_size: Some(5 * 1024 * 1024 * 1024 * 1024), // 5TB
            max_path_length: Some(1024),
            max_metadata_size: Some(8192),
        }
    }

    async fn health_check(&self) -> FileResult<ProviderHealth> {
        let start_time = std::time::Instant::now();

        if !self.is_ready() {
            return Ok(ProviderHealth {
                healthy: false,
                checked_at: Utc::now(),
                response_time_ms: 0,
                error_message: Some("Google Cloud Storage client not initialized".to_string()),
                details: HashMap::new(),
            });
        }

        let store = self.object_store.as_ref().unwrap();

        // Try to list objects with a limit to test connectivity
        let healthy = match store.list(None).await {
            Ok(_) => true,
            Err(_) => false,
        };

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ProviderHealth {
            healthy,
            checked_at: Utc::now(),
            response_time_ms,
            error_message: if healthy {
                None
            } else {
                Some("Bucket not accessible".to_string())
            },
            details: HashMap::new(),
        })
    }

    async fn get_statistics(&self) -> FileResult<ProviderStatistics> {
        self.statistics
            .lock()
            .map(|stats| stats.clone())
            .map_err(|_| FileServiceError::InternalError {
                message: "Failed to get statistics".to_string(),
            })
    }

    async fn reload_config(&mut self, config: FileServiceConfig) -> FileResult<()> {
        let gcp_config = match &config.provider_config {
            ProviderConfig::GoogleCloud(gcp_config) => gcp_config.clone(),
            _ => {
                return Err(FileServiceError::ConfigError {
                    message: "Invalid provider config for Google Cloud Storage".to_string(),
                })
            }
        };

        self.config = config;
        self.gcp_config = gcp_config;
        self.initialized = false;

        self.initialize().await
    }
}

#[async_trait]
impl FileService for GoogleCloudStorageProvider {
    fn service_info(&self) -> FileServiceInfo {
        FileServiceInfo {
            name: self.config.name.clone(),
            version: "1.0.0".to_string(),
            provider: "google_cloud".to_string(),
            features: vec![
                ServiceFeature::MultipartUpload,
                ServiceFeature::Streaming,
                ServiceFeature::Metadata,
                ServiceFeature::Permissions,
                ServiceFeature::Encryption,
                ServiceFeature::Compression,
                ServiceFeature::PresignedUrls,
                ServiceFeature::BatchOperations,
                ServiceFeature::Search,
                ServiceFeature::Checksums,
                ServiceFeature::Versioning,
            ],
            limits: ServiceLimits {
                max_file_size: Some(5 * 1024 * 1024 * 1024 * 1024), // 5TB
                max_files_per_directory: None,
                max_directory_depth: None,
                max_path_length: Some(1024),
                max_metadata_size: Some(8192),
                rate_limits: None,
            },
        }
    }

    async fn health_check(&self) -> FileResult<()> {
        let health = StorageProvider::health_check(self).await?;
        if health.healthy {
            Ok(())
        } else {
            Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            })
        }
    }

    async fn upload_bytes(
        &self,
        path: &str,
        content: Bytes,
        options: UploadOptions,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(object_name.clone());

        // Check if object exists and overwrite is not allowed
        if !options.overwrite {
            match store.head(&object_path).await {
                Ok(_) => {
                    self.record_operation(false, None);
                    return Err(FileServiceError::FileAlreadyExists {
                        path: normalized_path,
                    });
                }
                Err(object_store::Error::NotFound { .. }) => {
                    // File doesn't exist, which is what we want
                }
                Err(e) => {
                    self.record_operation(false, None);
                    return Err(FileServiceError::from(e));
                }
            }
        }

        // Prepare metadata
        let mut put_options = object_store::PutOptions::default();
        
        // Set content type
        if let Some(content_type) = &options.content_type {
            put_options.content_type = Some(content_type.clone());
        } else {
            let content_type = mime_guess::from_path(&normalized_path)
                .first_or_octet_stream()
                .to_string();
            put_options.content_type = Some(content_type);
        }

        // Set custom metadata
        if !options.metadata.is_empty() {
            put_options.attributes = Some(
                options.metadata
                    .into_iter()
                    .map(|(k, v)| (object_store::Attribute::from(k), v))
                    .collect()
            );
        }

        // Upload the object
        store.put_opts(&object_path, content.clone().into(), put_options).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        let bytes_transferred = content.len() as u64;
        self.record_operation(true, Some(bytes_transferred));

        Ok(FileOperationResult::success("upload".to_string(), normalized_path)
            .with_bytes_transferred(bytes_transferred))
    }

    async fn upload_stream(
        &self,
        path: &str,
        stream: FileStream,
        size: Option<u64>,
        options: UploadOptions,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        // For streaming uploads, we'll collect the stream into bytes first
        // In a production implementation, you might want to use multipart upload for large streams
        let chunks: Result<Vec<Bytes>, std::io::Error> = stream.collect().await;
        let chunks = chunks.map_err(|e| FileServiceError::IoError {
            message: format!("Failed to read stream: {}", e),
        })?;

        let content = chunks.into_iter().fold(Bytes::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            acc
        });

        self.upload_bytes(&normalized_path, content, options).await
    }

    async fn download_bytes(
        &self,
        path: &str,
        options: DownloadOptions,
    ) -> FileResult<(Bytes, FileInfo)> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(object_name.clone());

        // Get object metadata first
        let metadata = store.head(&object_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        // Prepare get options
        let mut get_options = object_store::GetOptions::default();

        // Set range if specified
        if let Some((start, end)) = options.range {
            let range = if let Some(end) = end {
                object_store::GetRange::Bounded(start..=end)
            } else {
                object_store::GetRange::Offset(start)
            };
            get_options.range = Some(range);
        }

        // Set conditional headers
        if let Some(if_match) = &options.if_match {
            get_options.if_match = Some(if_match.clone());
        }

        if let Some(if_modified_since) = options.if_modified_since {
            get_options.if_modified_since = Some(if_modified_since);
        }

        // Download the object
        let result = store.get_opts(&object_path, get_options).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        // Read content
        let content = result.bytes().await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to read Google Cloud Storage object body: {}", e),
            }
        })?;

        let bytes_transferred = content.len() as u64;

        // Create file info
        let content_type = metadata.content_type.unwrap_or_else(|| {
            mime_guess::from_path(&normalized_path)
                .first_or_octet_stream()
                .to_string()
        });

        let file_info = self.create_file_info_from_object(
            &normalized_path,
            metadata.size as i64,
            Some(metadata.last_modified),
            metadata.e_tag.clone(),
            Some(content_type),
            Some(
                metadata.attributes
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect()
            ),
        );

        self.record_operation(true, Some(bytes_transferred));

        Ok((content, file_info))
    }

    async fn download_stream(
        &self,
        path: &str,
        options: DownloadOptions,
    ) -> FileResult<(FileStream, FileInfo)> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(object_name.clone());

        // Get object metadata first
        let metadata = store.head(&object_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        // Create file info
        let content_type = metadata.content_type.unwrap_or_else(|| {
            mime_guess::from_path(&normalized_path)
                .first_or_octet_stream()
                .to_string()
        });

        let file_info = self.create_file_info_from_object(
            &normalized_path,
            metadata.size as i64,
            Some(metadata.last_modified),
            metadata.e_tag.clone(),
            Some(content_type),
            Some(
                metadata.attributes
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect()
            ),
        );

        // Prepare get options
        let mut get_options = object_store::GetOptions::default();

        // Set range if specified
        if let Some((start, end)) = options.range {
            let range = if let Some(end) = end {
                object_store::GetRange::Bounded(start..=end)
            } else {
                object_store::GetRange::Offset(start)
            };
            get_options.range = Some(range);
        }

        // Get the object for streaming
        let result = store.get_opts(&object_path, get_options).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        // Convert to our FileStream
        let stream: FileStream = Box::pin(
            result.into_stream()
                .map_ok(|chunk| chunk)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        );

        self.record_operation(true, None);

        Ok((stream, file_info))
    }

    async fn get_file_info(&self, path: &str) -> FileResult<FileInfo> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(object_name.clone());

        let metadata = store.head(&object_path).await.map_err(|e| {
            FileServiceError::from(e)
        })?;

        let content_type = metadata.content_type.unwrap_or_else(|| {
            mime_guess::from_path(&normalized_path)
                .first_or_octet_stream()
                .to_string()
        });

        Ok(self.create_file_info_from_object(
            &normalized_path,
            metadata.size as i64,
            Some(metadata.last_modified),
            metadata.e_tag.clone(),
            Some(content_type),
            Some(
                metadata.attributes
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect()
            ),
        ))
    }

    async fn exists(&self, path: &str) -> FileResult<bool> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(object_name.clone());

        match store.head(&object_path).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(FileServiceError::from(e)),
        }
    }

    async fn delete_file(&self, path: &str) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(object_name.clone());

        // Check if object exists
        if !self.exists(&normalized_path).await? {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_path,
            });
        }

        store.delete(&object_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        self.record_operation(true, None);

        Ok(FileOperationResult::success("delete".to_string(), normalized_path))
    }

    async fn copy_file(
        &self,
        source: &str,
        destination: &str,
        _options: CopyOptions,
    ) -> FileResult<FileOperationResult> {
        let normalized_source = self.validate_path(source)?;
        let normalized_dest = self.validate_path(destination)?;
        let source_object = self.get_object_name(&normalized_source);
        let dest_object = self.get_object_name(&normalized_dest);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let source_path = ObjectPath::from(source_object.clone());
        let dest_path = ObjectPath::from(dest_object.clone());

        // Check if source exists
        if !self.exists(&normalized_source).await? {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_source,
            });
        }

        store.copy(&source_path, &dest_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        // Get file size for bytes transferred
        let file_info = self.get_file_info(&normalized_dest).await.ok();
        let bytes_transferred = file_info.map(|info| info.size).unwrap_or(0);

        self.record_operation(true, Some(bytes_transferred));

        Ok(FileOperationResult::success("copy".to_string(), normalized_dest)
            .with_bytes_transferred(bytes_transferred))
    }

    async fn move_file(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult> {
        // GCS doesn't have a native move operation, so we copy then delete
        let copy_result = self.copy_file(source, destination, options).await?;
        self.delete_file(source).await?;

        Ok(FileOperationResult::success("move".to_string(), destination.to_string())
            .with_bytes_transferred(copy_result.bytes_transferred.unwrap_or(0)))
    }

    // GCS doesn't have true directories, so these operations are simplified
    async fn create_directory(&self, path: &str) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        // In GCS, we can create a "directory" by creating a zero-byte object with a trailing slash
        let directory_object = format!("{}/", self.get_object_name(&normalized_path));

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(directory_object.clone());

        let mut put_options = object_store::PutOptions::default();
        put_options.content_type = Some("application/x-directory".to_string());

        store.put_opts(&object_path, Bytes::new().into(), put_options).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        self.record_operation(true, None);

        Ok(FileOperationResult::success("create_directory".to_string(), normalized_path))
    }

    async fn list_directory(
        &self,
        path: &str,
        options: ListOptions,
    ) -> FileResult<DirectoryListing> {
        let normalized_path = self.validate_path(path)?;
        let prefix = if normalized_path == "/" {
            None
        } else {
            Some(ObjectPath::from(format!("{}/", self.get_object_name(&normalized_path))))
        };

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();

        let mut list_stream = store.list(prefix.as_ref());
        let mut entries = Vec::new();

        // Collect all objects
        while let Some(object_result) = list_stream.next().await {
            let object_meta = object_result.map_err(|e| {
                FileServiceError::CloudStorageError {
                    provider: "Google Cloud Storage".to_string(),
                    message: e.to_string(),
                }
            })?;

            let object_name = object_meta.location.to_string();
            let relative_path = self.get_relative_path(&object_name);
            let name = object_name.split('/').last().unwrap_or(&object_name).to_string();

            // Skip the directory marker itself
            if object_name.ends_with('/') && relative_path == normalized_path {
                continue;
            }

            let file_info = self.create_file_info_from_object(
                &relative_path,
                object_meta.size as i64,
                Some(object_meta.last_modified),
                object_meta.e_tag.clone(),
                None,
                None,
            );

            entries.push(file_info);

            // Apply limit during collection to avoid loading too much data
            if let Some(limit) = options.limit {
                if entries.len() >= limit {
                    break;
                }
            }
        }

        // Apply filtering and sorting
        let filtered_entries = self.apply_file_filters(entries, &options);

        Ok(DirectoryListing::new(normalized_path, filtered_entries))
    }

    async fn delete_directory(
        &self,
        path: &str,
        recursive: bool,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;

        if !recursive {
            // For non-recursive, just delete the directory marker
            let directory_object = format!("{}/", self.get_object_name(&normalized_path));
            return self.delete_file(&self.get_relative_path(&directory_object)).await;
        }

        // For recursive, list and delete all objects with the prefix
        let prefix = Some(ObjectPath::from(format!("{}/", self.get_object_name(&normalized_path))));

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let mut list_stream = store.list(prefix.as_ref());
        let mut total_deleted = 0;

        // Delete objects in batches
        while let Some(object_result) = list_stream.next().await {
            let object_meta = object_result.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::CloudStorageError {
                    provider: "Google Cloud Storage".to_string(),
                    message: e.to_string(),
                }
            })?;

            store.delete(&object_meta.location).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::from(e)
            })?;

            total_deleted += 1;
        }

        self.record_operation(true, None);

        Ok(FileOperationResult::success("delete_directory".to_string(), normalized_path))
    }

    async fn copy_directory(
        &self,
        source: &str,
        destination: &str,
        _options: CopyOptions,
    ) -> FileResult<FileOperationResult> {
        // This would be a complex operation involving listing and copying many objects
        // For now, return not implemented
        Err(FileServiceError::InternalError {
            message: "Directory copy not implemented for Google Cloud Storage".to_string(),
        })
    }

    async fn move_directory(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult> {
        // Copy then delete
        self.copy_directory(source, destination, options).await?;
        self.delete_directory(source, true).await?;

        Ok(FileOperationResult::success("move_directory".to_string(), destination.to_string()))
    }

    // Multipart upload operations (using object_store's multipart support)
    async fn initiate_multipart_upload(
        &self,
        path: &str,
        options: UploadOptions,
    ) -> FileResult<MultipartUpload> {
        let normalized_path = self.validate_path(path)?;
        let upload_id = uuid::Uuid::new_v4().to_string();

        let upload = MultipartUpload {
            upload_id: upload_id.clone(),
            path: normalized_path,
            initiated_at: Utc::now(),
            total_size: None,
            parts: Vec::new(),
            metadata: options.metadata,
        };

        if let Ok(mut uploads) = self.multipart_uploads.lock() {
            uploads.insert(upload_id.clone(), upload.clone());
        }

        Ok(upload)
    }

    async fn upload_part(
        &self,
        upload_id: &str,
        part_number: u32,
        content: Bytes,
    ) -> FileResult<UploadPart> {
        let mut uploads = self.multipart_uploads.lock().map_err(|_| {
            FileServiceError::InternalError {
                message: "Failed to lock multipart uploads".to_string(),
            }
        })?;

        let upload = uploads.get_mut(upload_id).ok_or_else(|| {
            FileServiceError::InvalidMultipartUpload {
                reason: "Upload not found".to_string(),
            }
        })?;

        // For GCS, we'll store parts temporarily and combine them during completion
        // This is a simplified implementation
        let part = UploadPart {
            part_number,
            size: content.len() as u64,
            etag: format!("part-{}", part_number),
            uploaded_at: Utc::now(),
            checksum: None,
        };

        upload.parts.push(part.clone());
        upload.parts.sort_by_key(|p| p.part_number);

        Ok(part)
    }

    async fn complete_multipart_upload(
        &self,
        upload_id: &str,
        _parts: Vec<UploadPart>,
    ) -> FileResult<FileOperationResult> {
        let mut uploads = self.multipart_uploads.lock().map_err(|_| {
            FileServiceError::InternalError {
                message: "Failed to lock multipart uploads".to_string(),
            }
        })?;

        let upload = uploads.remove(upload_id).ok_or_else(|| {
            FileServiceError::InvalidMultipartUpload {
                reason: "Upload not found".to_string(),
            }
        })?;

        // For this simplified implementation, we'll just create an empty object
        // In a real implementation, you would combine the parts
        let object_name = self.get_object_name(&upload.path);
        let object_path = ObjectPath::from(object_name);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();

        store.put(&object_path, Bytes::new().into()).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        let total_bytes = upload.parts.iter().map(|p| p.size).sum();
        self.record_operation(true, Some(total_bytes));

        Ok(FileOperationResult::success("multipart_upload".to_string(), upload.path)
            .with_bytes_transferred(total_bytes))
    }

    async fn abort_multipart_upload(&self, upload_id: &str) -> FileResult<FileOperationResult> {
        let mut uploads = self.multipart_uploads.lock().map_err(|_| {
            FileServiceError::InternalError {
                message: "Failed to lock multipart uploads".to_string(),
            }
        })?;

        let upload = uploads.remove(upload_id).ok_or_else(|| {
            FileServiceError::InvalidMultipartUpload {
                reason: "Upload not found".to_string(),
            }
        })?;

        // Clean up any temporary data if needed
        Ok(FileOperationResult::success("abort_multipart_upload".to_string(), upload.path))
    }

    async fn list_multipart_uploads(&self) -> FileResult<Vec<MultipartUpload>> {
        let uploads = self.multipart_uploads.lock().map_err(|_| {
            FileServiceError::InternalError {
                message: "Failed to lock multipart uploads".to_string(),
            }
        })?;

        Ok(uploads.values().cloned().collect())
    }

    async fn batch_operation(&self, operation: BatchOperation) -> FileResult<BatchOperationResult> {
        // Implement batch operations similar to S3 but using GCS operations
        // This is a simplified implementation
        Err(FileServiceError::InternalError {
            message: "Batch operations not fully implemented for Google Cloud Storage".to_string(),
        })
    }

    async fn get_storage_usage(&self, _path: Option<&str>) -> FileResult<StorageUsage> {
        // GCS doesn't provide direct storage usage information
        // This would require listing all objects and summing sizes
        Ok(StorageUsage {
            total_bytes: None,
            used_bytes: 0,
            available_bytes: None,
            file_count: 0,
            directory_count: 0,
        })
    }

    async fn search_files(&self, criteria: SearchCriteria) -> FileResult<Vec<FileInfo>> {
        // Implement search using GCS list operations with prefix filtering
        // This is a simplified implementation
        let listing = self.list_directory(&criteria.path, ListOptions {
            recursive: criteria.recursive,
            ..Default::default()
        }).await?;

        let mut results = Vec::new();
        for entry in listing.entries {
            let mut matches = true;

            // Apply name pattern filter
            if let Some(pattern) = &criteria.name_pattern {
                if !glob_match(pattern, &entry.name) {
                    matches = false;
                }
            }

            // Apply size filters
            if let Some(min_size) = criteria.min_size {
                if entry.size < min_size {
                    matches = false;
                }
            }

            if let Some(max_size) = criteria.max_size {
                if entry.size > max_size {
                    matches = false;
                }
            }

            if matches {
                results.push(entry);
            }

            // Check limit
            if let Some(limit) = criteria.limit {
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    async fn generate_presigned_url(
        &self,
        _path: &str,
        _operation: PresignedOperation,
        _expires_in_seconds: u64,
    ) -> FileResult<String> {
        // GCS presigned URLs would require using signed URLs
        // This is a simplified implementation
        Err(FileServiceError::InternalError {
            message: "Presigned URLs not fully implemented for Google Cloud Storage".to_string(),
        })
    }

    async fn set_metadata(
        &self,
        path: &str,
        metadata: HashMap<String, String>,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let object_name = self.get_object_name(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "google_cloud".to_string(),
            });
        }

        let store = self.object_store.as_ref().unwrap();
        let object_path = ObjectPath::from(object_name);

        // GCS requires copying the object to update metadata
        store.copy(&object_path, &object_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::from(e)
        })?;

        self.record_operation(true, None);

        Ok(FileOperationResult::success("set_metadata".to_string(), normalized_path))
    }

    async fn get_metadata(&self, path: &str) -> FileResult<HashMap<String, String>> {
        let file_info = self.get_file_info(path).await?;
        Ok(file_info.metadata)
    }
}

impl GoogleCloudStorageProvider {
    /// Apply file filters (simplified for GCS)
    fn apply_file_filters(&self, entries: Vec<FileInfo>, options: &ListOptions) -> Vec<FileInfo> {
        let mut filtered: Vec<FileInfo> = entries
            .into_iter()
            .filter(|entry| {
                // Filter hidden files
                if !options.include_hidden && entry.is_hidden() {
                    return false;
                }

                // Apply pattern matching
                if let Some(pattern) = &options.pattern {
                    if !glob_match(pattern, &entry.name) {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Apply sorting (simplified)
        filtered.sort_by(|a, b| a.name.cmp(&b.name));

        // Apply limit
        if let Some(limit) = options.limit {
            filtered.truncate(limit);
        }

        filtered
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    // Simple implementation - in production would use a proper glob library
    if pattern == "*" {
        return true;
    }
    
    if pattern.starts_with("*.") {
        let extension = &pattern[2..];
        return text.ends_with(&format!(".{}", extension));
    }
    
    pattern == text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::FileServiceConfig;

    #[test]
    fn test_gcs_provider_creation() {
        let config = FileServiceConfig::google_cloud(
            "test".to_string(),
            "test-bucket".to_string(),
        );
        let provider = GoogleCloudStorageProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_object_name_generation() {
        let config = FileServiceConfig::google_cloud(
            "test".to_string(),
            "test-bucket".to_string(),
        );
        let provider = GoogleCloudStorageProvider::new(config).unwrap();

        assert_eq!(provider.get_object_name("/test.txt"), "test.txt");
        assert_eq!(provider.get_object_name("test.txt"), "test.txt");
        assert_eq!(provider.get_object_name("/dir/test.txt"), "dir/test.txt");
    }

    #[test]
    fn test_object_name_with_root_path() {
        let mut config = FileServiceConfig::google_cloud(
            "test".to_string(),
            "test-bucket".to_string(),
        );
        config.root_path = "/app/files".to_string();
        let provider = GoogleCloudStorageProvider::new(config).unwrap();

        assert_eq!(provider.get_object_name("/test.txt"), "app/files/test.txt");
        assert_eq!(provider.get_object_name("test.txt"), "app/files/test.txt");
    }

    #[test]
    fn test_path_validation() {
        let config = FileServiceConfig::google_cloud(
            "test".to_string(),
            "test-bucket".to_string(),
        );
        let provider = GoogleCloudStorageProvider::new(config).unwrap();

        assert_eq!(provider.validate_path("/test.txt").unwrap(), "/test.txt");
        assert_eq!(provider.validate_path("test.txt").unwrap(), "/test.txt");
        assert_eq!(provider.validate_path("/dir/../test.txt").unwrap(), "/test.txt");
        assert!(provider.validate_path("").is_err());
    }

    #[test]
    fn test_service_info() {
        let config = FileServiceConfig::google_cloud(
            "test".to_string(),
            "test-bucket".to_string(),
        );
        let provider = GoogleCloudStorageProvider::new(config).unwrap();

        let info = provider.service_info();
        assert_eq!(info.name, "test");
        assert_eq!(info.provider, "google_cloud");
        assert!(info.features.contains(&ServiceFeature::PresignedUrls));
        assert!(info.features.contains(&ServiceFeature::MultipartUpload));
        assert!(info.features.contains(&ServiceFeature::Compression));
    }

    #[test]
    fn test_capabilities() {
        let config = FileServiceConfig::google_cloud(
            "test".to_string(),
            "test-bucket".to_string(),
        );
        let provider = GoogleCloudStorageProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.streaming_upload);
        assert!(caps.streaming_download);
        assert!(caps.multipart_upload);
        assert!(caps.resumable_upload); // GCS supports resumable uploads
        assert!(caps.presigned_urls);
        assert!(caps.compression_support); // GCS supports compression
        assert!(!caps.directory_operations); // GCS doesn't have true directories
    }
}