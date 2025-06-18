use crate::models::{
    config::{FileServiceConfig, S3Config, ProviderConfig},
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
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    error::SdkError,
    operation::{
        get_object::GetObjectError,
        head_object::HeadObjectError,
        put_object::PutObjectError,
    },
    primitives::{ByteStream, SdkBody},
    types::{CompletedMultipartUpload, CompletedPart, ObjectCannedAcl},
    Client as S3Client,
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::{stream, Stream, StreamExt, TryStreamExt};
use object_store::{
    aws::{AmazonS3, AmazonS3Builder},
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

/// AWS S3 storage provider
pub struct S3StorageProvider {
    config: FileServiceConfig,
    s3_config: S3Config,
    client: Option<S3Client>,
    object_store: Option<Arc<AmazonS3>>,
    initialized: bool,
    statistics: Arc<Mutex<ProviderStatistics>>,
    multipart_uploads: Arc<Mutex<HashMap<String, MultipartUpload>>>,
}

impl S3StorageProvider {
    /// Create a new S3 storage provider
    pub fn new(config: FileServiceConfig) -> FileResult<Self> {
        let s3_config = match &config.provider_config {
            ProviderConfig::S3(s3_config) => s3_config.clone(),
            _ => {
                return Err(FileServiceError::ConfigError {
                    message: "Invalid provider config for S3 storage".to_string(),
                })
            }
        };

        Ok(Self {
            config,
            s3_config,
            client: None,
            object_store: None,
            initialized: false,
            statistics: Arc::new(Mutex::new(ProviderStatistics::default())),
            multipart_uploads: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get the S3 key for a given path
    fn get_s3_key(&self, path: &str) -> String {
        let clean_path = path.strip_prefix('/').unwrap_or(path);
        if self.config.root_path.is_empty() || self.config.root_path == "/" {
            clean_path.to_string()
        } else {
            let root = self.config.root_path.strip_prefix('/').unwrap_or(&self.config.root_path);
            format!("{}/{}", root, clean_path)
        }
    }

    /// Convert S3 key back to relative path
    fn get_relative_path(&self, s3_key: &str) -> String {
        if self.config.root_path.is_empty() || self.config.root_path == "/" {
            format!("/{}", s3_key)
        } else {
            let root = self.config.root_path.strip_prefix('/').unwrap_or(&self.config.root_path);
            if let Some(relative) = s3_key.strip_prefix(&format!("{}/", root)) {
                format!("/{}", relative)
            } else if s3_key == root {
                "/".to_string()
            } else {
                format!("/{}", s3_key)
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

    /// Create FileInfo from S3 object metadata
    fn create_file_info_from_s3_object(
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

    /// Convert S3 SDK error to FileServiceError
    fn map_s3_error(&self, error: SdkError<impl std::error::Error + Send + Sync + 'static>, key: &str) -> FileServiceError {
        match error {
            SdkError::ServiceError(service_error) => {
                let status_code = service_error.raw().status().as_u16();
                match status_code {
                    404 => FileServiceError::FileNotFound {
                        path: self.get_relative_path(key),
                    },
                    403 => FileServiceError::PermissionDenied {
                        path: self.get_relative_path(key),
                    },
                    _ => FileServiceError::CloudStorageError {
                        provider: "S3".to_string(),
                        message: service_error.to_string(),
                    },
                }
            }
            SdkError::TimeoutError(_) => FileServiceError::TimeoutError {
                operation: "S3 operation".to_string(),
            },
            SdkError::DispatchFailure(_) => FileServiceError::NetworkError {
                message: error.to_string(),
            },
            _ => FileServiceError::CloudStorageError {
                provider: "S3".to_string(),
                message: error.to_string(),
            },
        }
    }
}

#[async_trait]
impl StorageProvider for S3StorageProvider {
    async fn initialize(&mut self) -> FileResult<()> {
        if self.initialized {
            return Ok(());
        }

        // Initialize AWS config
        let mut aws_config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(self.s3_config.region.clone()));

        // Set credentials if provided
        if let (Some(access_key), Some(secret_key)) = (
            &self.s3_config.access_key_id,
            &self.s3_config.secret_access_key,
        ) {
            let credentials = aws_config::environment::credentials::EnvironmentVariableCredentialsProvider::default()
                .try_into()
                .map_err(|e| FileServiceError::ConfigError {
                    message: format!("Failed to create credentials: {}", e),
                })?;

            aws_config_builder = aws_config_builder.credentials_provider(credentials);
        }

        let aws_config = aws_config_builder.load().await;

        // Create S3 client
        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);

        if let Some(endpoint_url) = &self.s3_config.endpoint_url {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint_url);
        }

        if self.s3_config.path_style {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let s3_config = s3_config_builder.build();
        self.client = Some(S3Client::from_conf(s3_config));

        // Initialize object_store
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(&self.s3_config.bucket)
            .with_region(&self.s3_config.region);

        if let (Some(access_key), Some(secret_key)) = (
            &self.s3_config.access_key_id,
            &self.s3_config.secret_access_key,
        ) {
            builder = builder
                .with_access_key_id(access_key)
                .with_secret_access_key(secret_key);
        }

        if let Some(session_token) = &self.s3_config.session_token {
            builder = builder.with_token(session_token);
        }

        if let Some(endpoint_url) = &self.s3_config.endpoint_url {
            let url: Url = endpoint_url.parse().map_err(|e| FileServiceError::ConfigError {
                message: format!("Invalid endpoint URL: {}", e),
            })?;
            builder = builder.with_endpoint(url.to_string());
        }

        self.object_store = Some(Arc::new(builder.build().map_err(|e| {
            FileServiceError::ConfigError {
                message: format!("Failed to initialize S3 client: {}", e),
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

        self.client = None;
        self.object_store = None;
        self.initialized = false;
        Ok(())
    }

    fn config(&self) -> &FileServiceConfig {
        &self.config
    }

    fn is_ready(&self) -> bool {
        self.initialized && self.client.is_some() && self.object_store.is_some()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming_upload: true,
            streaming_download: true,
            multipart_upload: true,
            resumable_upload: false,
            directory_operations: false, // S3 doesn't have true directories
            metadata_support: true,
            permissions_support: false, // ACLs are separate
            encryption_support: true,
            compression_support: false,
            versioning_support: true, // If bucket versioning is enabled
            presigned_urls: true,
            batch_operations: true,
            search_support: true,
            locking_support: false,
            atomic_operations: true,
            transaction_support: false,
            max_file_size: Some(5 * 1024 * 1024 * 1024 * 1024), // 5TB
            max_path_length: Some(1024),
            max_metadata_size: Some(2048),
        }
    }

    async fn health_check(&self) -> FileResult<ProviderHealth> {
        let start_time = std::time::Instant::now();

        if !self.is_ready() {
            return Ok(ProviderHealth {
                healthy: false,
                checked_at: Utc::now(),
                response_time_ms: 0,
                error_message: Some("S3 client not initialized".to_string()),
                details: HashMap::new(),
            });
        }

        let client = self.client.as_ref().unwrap();

        let healthy = match client
            .head_bucket()
            .bucket(&self.s3_config.bucket)
            .send()
            .await
        {
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
        let s3_config = match &config.provider_config {
            ProviderConfig::S3(s3_config) => s3_config.clone(),
            _ => {
                return Err(FileServiceError::ConfigError {
                    message: "Invalid provider config for S3 storage".to_string(),
                })
            }
        };

        self.config = config;
        self.s3_config = s3_config;
        self.initialized = false;

        self.initialize().await
    }
}

#[async_trait]
impl FileService for S3StorageProvider {
    fn service_info(&self) -> FileServiceInfo {
        FileServiceInfo {
            name: self.config.name.clone(),
            version: "1.0.0".to_string(),
            provider: "s3".to_string(),
            features: vec![
                ServiceFeature::MultipartUpload,
                ServiceFeature::Streaming,
                ServiceFeature::Metadata,
                ServiceFeature::Encryption,
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
                max_metadata_size: Some(2048),
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
                service: "s3".to_string(),
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
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        // Check if object exists and overwrite is not allowed
        if !options.overwrite {
            match client
                .head_object()
                .bucket(&self.s3_config.bucket)
                .key(&s3_key)
                .send()
                .await
            {
                Ok(_) => {
                    self.record_operation(false, None);
                    return Err(FileServiceError::FileAlreadyExists {
                        path: normalized_path,
                    });
                }
                Err(SdkError::ServiceError(service_error)) => {
                    if service_error.raw().status().as_u16() != 404 {
                        self.record_operation(false, None);
                        return Err(self.map_s3_error(SdkError::ServiceError(service_error), &s3_key));
                    }
                }
                Err(e) => {
                    self.record_operation(false, None);
                    return Err(self.map_s3_error(e, &s3_key));
                }
            }
        }

        let mut put_request = client
            .put_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .body(ByteStream::from(content.clone()));

        // Set content type
        if let Some(content_type) = &options.content_type {
            put_request = put_request.content_type(content_type);
        } else {
            let content_type = mime_guess::from_path(&normalized_path)
                .first_or_octet_stream()
                .to_string();
            put_request = put_request.content_type(content_type);
        }

        // Set metadata
        for (key, value) in &options.metadata {
            put_request = put_request.metadata(key, value);
        }

        // Set server-side encryption
        if options.encrypt || self.s3_config.server_side_encryption.is_some() {
            if let Some(sse) = &self.s3_config.server_side_encryption {
                put_request = put_request.server_side_encryption(
                    aws_sdk_s3::types::ServerSideEncryption::from(sse.as_str())
                );
            }
        }

        // Set storage class
        if let Some(storage_class) = &self.s3_config.storage_class {
            put_request = put_request.storage_class(
                aws_sdk_s3::types::StorageClass::from(storage_class.as_str())
            );
        }

        // Set KMS key if specified
        if let Some(kms_key_id) = &self.s3_config.kms_key_id {
            put_request = put_request.ssekms_key_id(kms_key_id);
        }

        // Execute upload
        let result = put_request.send().await.map_err(|e| {
            self.record_operation(false, None);
            self.map_s3_error(e, &s3_key)
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
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
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
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        let mut get_request = client
            .get_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key);

        // Set range if specified
        if let Some((start, end)) = options.range {
            let range = if let Some(end) = end {
                format!("bytes={}-{}", start, end)
            } else {
                format!("bytes={}-", start)
            };
            get_request = get_request.range(range);
        }

        // Set conditional headers
        if let Some(if_match) = &options.if_match {
            get_request = get_request.if_match(if_match);
        }

        if let Some(if_modified_since) = options.if_modified_since {
            get_request = get_request.if_modified_since(
                aws_smithy_types::DateTime::from_secs(if_modified_since.timestamp())
            );
        }

        let result = get_request.send().await.map_err(|e| {
            self.record_operation(false, None);
            self.map_s3_error(e, &s3_key)
        })?;

        // Get object metadata
        let content_length = result.content_length().unwrap_or(0);
        let content_type = result.content_type().unwrap_or("application/octet-stream").to_string();
        let etag = result.e_tag().map(|s| s.to_string());
        let last_modified = result.last_modified().map(|dt| {
            DateTime::from_timestamp(dt.secs(), 0).unwrap_or_else(Utc::now)
        });

        // Collect metadata
        let metadata = result.metadata().cloned().unwrap_or_default();

        // Read content
        let body = result.body.collect().await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to read S3 object body: {}", e),
            }
        })?;

        let content = Bytes::from(body.into_bytes());
        let bytes_transferred = content.len() as u64;

        // Create file info
        let file_info = self.create_file_info_from_s3_object(
            &normalized_path,
            content_length,
            last_modified,
            etag,
            Some(content_type),
            Some(metadata),
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
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        // First get object metadata
        let head_result = client
            .head_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .send()
            .await
            .map_err(|e| {
                self.record_operation(false, None);
                self.map_s3_error(e, &s3_key)
            })?;

        let content_length = head_result.content_length().unwrap_or(0);
        let content_type = head_result.content_type().unwrap_or("application/octet-stream").to_string();
        let etag = head_result.e_tag().map(|s| s.to_string());
        let last_modified = head_result.last_modified().map(|dt| {
            DateTime::from_timestamp(dt.secs(), 0).unwrap_or_else(Utc::now)
        });
        let metadata = head_result.metadata().cloned().unwrap_or_default();

        // Create file info
        let file_info = self.create_file_info_from_s3_object(
            &normalized_path,
            content_length,
            last_modified,
            etag,
            Some(content_type),
            Some(metadata),
        );

        // Now get the object for streaming
        let mut get_request = client
            .get_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key);

        // Set range if specified
        if let Some((start, end)) = options.range {
            let range = if let Some(end) = end {
                format!("bytes={}-{}", start, end)
            } else {
                format!("bytes={}-", start)
            };
            get_request = get_request.range(range);
        }

        let result = get_request.send().await.map_err(|e| {
            self.record_operation(false, None);
            self.map_s3_error(e, &s3_key)
        })?;

        // Convert S3 body to our FileStream
        let stream: FileStream = Box::pin(
            result.body
                .map_ok(|chunk| Bytes::from(chunk.into_bytes()))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        );

        self.record_operation(true, None);

        Ok((stream, file_info))
    }

    async fn get_file_info(&self, path: &str) -> FileResult<FileInfo> {
        let normalized_path = self.validate_path(path)?;
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        let result = client
            .head_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .send()
            .await
            .map_err(|e| self.map_s3_error(e, &s3_key))?;

        let content_length = result.content_length().unwrap_or(0);
        let content_type = result.content_type().unwrap_or("application/octet-stream").to_string();
        let etag = result.e_tag().map(|s| s.to_string());
        let last_modified = result.last_modified().map(|dt| {
            DateTime::from_timestamp(dt.secs(), 0).unwrap_or_else(Utc::now)
        });
        let metadata = result.metadata().cloned().unwrap_or_default();

        Ok(self.create_file_info_from_s3_object(
            &normalized_path,
            content_length,
            last_modified,
            etag,
            Some(content_type),
            Some(metadata),
        ))
    }

    async fn exists(&self, path: &str) -> FileResult<bool> {
        let normalized_path = self.validate_path(path)?;
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        match client
            .head_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(SdkError::ServiceError(service_error)) => {
                if service_error.raw().status().as_u16() == 404 {
                    Ok(false)
                } else {
                    Err(self.map_s3_error(SdkError::ServiceError(service_error), &s3_key))
                }
            }
            Err(e) => Err(self.map_s3_error(e, &s3_key)),
        }
    }

    async fn delete_file(&self, path: &str) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        // Check if object exists
        if !self.exists(&normalized_path).await? {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_path,
            });
        }

        client
            .delete_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .send()
            .await
            .map_err(|e| {
                self.record_operation(false, None);
                self.map_s3_error(e, &s3_key)
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
        let source_key = self.get_s3_key(&normalized_source);
        let dest_key = self.get_s3_key(&normalized_dest);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        // Check if source exists
        if !self.exists(&normalized_source).await? {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_source,
            });
        }

        let copy_source = format!("{}/{}", self.s3_config.bucket, source_key);

        let result = client
            .copy_object()
            .bucket(&self.s3_config.bucket)
            .key(&dest_key)
            .copy_source(&copy_source)
            .send()
            .await
            .map_err(|e| {
                self.record_operation(false, None);
                self.map_s3_error(e, &dest_key)
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
        // S3 doesn't have a native move operation, so we copy then delete
        let copy_result = self.copy_file(source, destination, options).await?;
        self.delete_file(source).await?;

        Ok(FileOperationResult::success("move".to_string(), destination.to_string())
            .with_bytes_transferred(copy_result.bytes_transferred.unwrap_or(0)))
    }

    // S3 doesn't have true directories, so these operations are simplified
    async fn create_directory(&self, path: &str) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        // In S3, we can create a "directory" by creating a zero-byte object with a trailing slash
        let directory_key = format!("{}/", self.get_s3_key(&normalized_path));

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        client
            .put_object()
            .bucket(&self.s3_config.bucket)
            .key(&directory_key)
            .body(ByteStream::from(Bytes::new()))
            .content_type("application/x-directory")
            .send()
            .await
            .map_err(|e| {
                self.record_operation(false, None);
                self.map_s3_error(e, &directory_key)
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
            String::new()
        } else {
            format!("{}/", self.get_s3_key(&normalized_path))
        };

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        let mut list_request = client
            .list_objects_v2()
            .bucket(&self.s3_config.bucket)
            .prefix(&prefix);

        if !options.recursive {
            list_request = list_request.delimiter("/");
        }

        if let Some(limit) = options.limit {
            list_request = list_request.max_keys(limit as i32);
        }

        if let Some(token) = &options.continuation_token {
            list_request = list_request.continuation_token(token);
        }

        let result = list_request.send().await.map_err(|e| {
            FileServiceError::CloudStorageError {
                provider: "S3".to_string(),
                message: e.to_string(),
            }
        })?;

        let mut entries = Vec::new();

        // Add objects
        if let Some(objects) = result.contents() {
            for object in objects {
                if let Some(key) = object.key() {
                    // Skip the directory marker itself
                    if key.ends_with('/') && key.len() == prefix.len() + 1 {
                        continue;
                    }

                    let relative_path = self.get_relative_path(key);
                    let name = key.split('/').last().unwrap_or(key).to_string();

                    let file_info = self.create_file_info_from_s3_object(
                        &relative_path,
                        object.size().unwrap_or(0),
                        object.last_modified().map(|dt| {
                            DateTime::from_timestamp(dt.secs(), 0).unwrap_or_else(Utc::now)
                        }),
                        object.e_tag().map(|s| s.to_string()),
                        None,
                        None,
                    );

                    entries.push(file_info);
                }
            }
        }

        // Add common prefixes (subdirectories)
        if let Some(prefixes) = result.common_prefixes() {
            for prefix_info in prefixes {
                if let Some(prefix_key) = prefix_info.prefix() {
                    let relative_path = self.get_relative_path(prefix_key.trim_end_matches('/'));
                    let name = prefix_key
                        .trim_end_matches('/')
                        .split('/')
                        .last()
                        .unwrap_or(prefix_key)
                        .to_string();

                    let mut file_info = FileInfo::directory(relative_path, name);
                    file_info.created_at = Utc::now();
                    file_info.modified_at = Utc::now();

                    entries.push(file_info);
                }
            }
        }

        // Apply filtering and sorting (simplified for S3)
        let filtered_entries = self.apply_file_filters(entries, &options);

        let has_more = result.is_truncated().unwrap_or(false);
        let continuation_token = result.next_continuation_token().map(|s| s.to_string());

        Ok(DirectoryListing::paginated(
            normalized_path,
            filtered_entries,
            has_more,
            continuation_token,
        ))
    }

    async fn delete_directory(
        &self,
        path: &str,
        recursive: bool,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;

        if !recursive {
            // For non-recursive, just delete the directory marker
            let directory_key = format!("{}/", self.get_s3_key(&normalized_path));
            return self.delete_file(&self.get_relative_path(&directory_key)).await;
        }

        // For recursive, list and delete all objects with the prefix
        let prefix = format!("{}/", self.get_s3_key(&normalized_path));

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        // List all objects with the prefix
        let mut continuation_token = None;
        let mut total_deleted = 0;

        loop {
            let mut list_request = client
                .list_objects_v2()
                .bucket(&self.s3_config.bucket)
                .prefix(&prefix)
                .max_keys(1000);

            if let Some(token) = &continuation_token {
                list_request = list_request.continuation_token(token);
            }

            let result = list_request.send().await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::CloudStorageError {
                    provider: "S3".to_string(),
                    message: e.to_string(),
                }
            })?;

            if let Some(objects) = result.contents() {
                if objects.is_empty() {
                    break;
                }

                // Delete objects in batches
                let keys: Vec<_> = objects
                    .iter()
                    .filter_map(|obj| obj.key())
                    .collect();

                for chunk in keys.chunks(1000) {
                    let delete_objects: Vec<_> = chunk
                        .iter()
                        .map(|key| {
                            aws_sdk_s3::types::ObjectIdentifier::builder()
                                .key(*key)
                                .build()
                                .unwrap()
                        })
                        .collect();

                    let delete_request = aws_sdk_s3::types::Delete::builder()
                        .set_objects(Some(delete_objects))
                        .build()
                        .unwrap();

                    client
                        .delete_objects()
                        .bucket(&self.s3_config.bucket)
                        .delete(delete_request)
                        .send()
                        .await
                        .map_err(|e| {
                            self.record_operation(false, None);
                            FileServiceError::CloudStorageError {
                                provider: "S3".to_string(),
                                message: e.to_string(),
                            }
                        })?;

                    total_deleted += chunk.len();
                }
            }

            if !result.is_truncated().unwrap_or(false) {
                break;
            }

            continuation_token = result.next_continuation_token().map(|s| s.to_string());
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
            message: "Directory copy not implemented for S3".to_string(),
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

    // Multipart upload operations (using S3's native multipart upload)
    async fn initiate_multipart_upload(
        &self,
        path: &str,
        options: UploadOptions,
    ) -> FileResult<MultipartUpload> {
        let normalized_path = self.validate_path(path)?;
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        let mut request = client
            .create_multipart_upload()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key);

        // Set content type
        if let Some(content_type) = &options.content_type {
            request = request.content_type(content_type);
        }

        // Set metadata
        for (key, value) in &options.metadata {
            request = request.metadata(key, value);
        }

        let result = request.send().await.map_err(|e| {
            self.map_s3_error(e, &s3_key)
        })?;

        let upload_id = result.upload_id().unwrap().to_string();

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
        let uploads = self.multipart_uploads.lock().map_err(|_| {
            FileServiceError::InternalError {
                message: "Failed to lock multipart uploads".to_string(),
            }
        })?;

        let upload = uploads.get(upload_id).ok_or_else(|| {
            FileServiceError::InvalidMultipartUpload {
                reason: "Upload not found".to_string(),
            }
        })?;

        let s3_key = self.get_s3_key(&upload.path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        let result = client
            .upload_part()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .upload_id(upload_id)
            .part_number(part_number as i32)
            .body(ByteStream::from(content.clone()))
            .send()
            .await
            .map_err(|e| self.map_s3_error(e, &s3_key))?;

        let etag = result.e_tag().unwrap().to_string();

        let part = UploadPart {
            part_number,
            size: content.len() as u64,
            etag,
            uploaded_at: Utc::now(),
            checksum: None,
        };

        // Update the upload record
        drop(uploads);
        if let Ok(mut uploads) = self.multipart_uploads.lock() {
            if let Some(upload) = uploads.get_mut(upload_id) {
                upload.parts.push(part.clone());
                upload.parts.sort_by_key(|p| p.part_number);
            }
        }

        Ok(part)
    }

    async fn complete_multipart_upload(
        &self,
        upload_id: &str,
        parts: Vec<UploadPart>,
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

        let s3_key = self.get_s3_key(&upload.path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        // Create completed parts
        let completed_parts: Vec<_> = parts
            .iter()
            .map(|part| {
                CompletedPart::builder()
                    .part_number(part.part_number as i32)
                    .e_tag(&part.etag)
                    .build()
            })
            .collect();

        let completed_upload = CompletedMultipartUpload::builder()
            .set_parts(Some(completed_parts))
            .build();

        let result = client
            .complete_multipart_upload()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .upload_id(upload_id)
            .multipart_upload(completed_upload)
            .send()
            .await
            .map_err(|e| self.map_s3_error(e, &s3_key))?;

        let total_bytes = parts.iter().map(|p| p.size).sum();
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

        let s3_key = self.get_s3_key(&upload.path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        client
            .abort_multipart_upload()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .upload_id(upload_id)
            .send()
            .await
            .map_err(|e| self.map_s3_error(e, &s3_key))?;

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
        // Implement batch operations similar to local storage but using S3 operations
        // This is a simplified implementation
        Err(FileServiceError::InternalError {
            message: "Batch operations not fully implemented for S3".to_string(),
        })
    }

    async fn get_storage_usage(&self, _path: Option<&str>) -> FileResult<StorageUsage> {
        // S3 doesn't provide direct storage usage information
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
        // Implement search using S3 list operations with prefix filtering
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
        path: &str,
        operation: PresignedOperation,
        expires_in_seconds: u64,
    ) -> FileResult<String> {
        let normalized_path = self.validate_path(path)?;
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        let expires_in = std::time::Duration::from_secs(expires_in_seconds);

        let presigned_request = match operation {
            PresignedOperation::Read => {
                client
                    .get_object()
                    .bucket(&self.s3_config.bucket)
                    .key(&s3_key)
                    .presigned(
                        aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
                            .map_err(|e| FileServiceError::InternalError {
                                message: format!("Failed to create presigning config: {}", e),
                            })?
                    )
                    .await
                    .map_err(|e| FileServiceError::InternalError {
                        message: format!("Failed to create presigned URL: {}", e),
                    })?
            }
            PresignedOperation::Write => {
                client
                    .put_object()
                    .bucket(&self.s3_config.bucket)
                    .key(&s3_key)
                    .presigned(
                        aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
                            .map_err(|e| FileServiceError::InternalError {
                                message: format!("Failed to create presigning config: {}", e),
                            })?
                    )
                    .await
                    .map_err(|e| FileServiceError::InternalError {
                        message: format!("Failed to create presigned URL: {}", e),
                    })?
            }
            PresignedOperation::Delete => {
                client
                    .delete_object()
                    .bucket(&self.s3_config.bucket)
                    .key(&s3_key)
                    .presigned(
                        aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
                            .map_err(|e| FileServiceError::InternalError {
                                message: format!("Failed to create presigning config: {}", e),
                            })?
                    )
                    .await
                    .map_err(|e| FileServiceError::InternalError {
                        message: format!("Failed to create presigned URL: {}", e),
                    })?
            }
        };

        Ok(presigned_request.uri().to_string())
    }

    async fn set_metadata(
        &self,
        path: &str,
        metadata: HashMap<String, String>,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let s3_key = self.get_s3_key(&normalized_path);

        if !self.is_ready() {
            return Err(FileServiceError::ServiceUnavailable {
                service: "s3".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();

        // S3 requires copying the object to update metadata
        let copy_source = format!("{}/{}", self.s3_config.bucket, s3_key);

        let mut request = client
            .copy_object()
            .bucket(&self.s3_config.bucket)
            .key(&s3_key)
            .copy_source(&copy_source)
            .metadata_directive(aws_sdk_s3::types::MetadataDirective::Replace);

        for (key, value) in metadata {
            request = request.metadata(key, value);
        }

        request.send().await.map_err(|e| {
            self.record_operation(false, None);
            self.map_s3_error(e, &s3_key)
        })?;

        self.record_operation(true, None);

        Ok(FileOperationResult::success("set_metadata".to_string(), normalized_path))
    }

    async fn get_metadata(&self, path: &str) -> FileResult<HashMap<String, String>> {
        let file_info = self.get_file_info(path).await?;
        Ok(file_info.metadata)
    }
}

impl S3StorageProvider {
    /// Apply file filters (simplified for S3)
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
    fn test_s3_provider_creation() {
        let config = FileServiceConfig::s3(
            "test".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        );
        let provider = S3StorageProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_s3_key_generation() {
        let config = FileServiceConfig::s3(
            "test".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        );
        let provider = S3StorageProvider::new(config).unwrap();

        assert_eq!(provider.get_s3_key("/test.txt"), "test.txt");
        assert_eq!(provider.get_s3_key("test.txt"), "test.txt");
        assert_eq!(provider.get_s3_key("/dir/test.txt"), "dir/test.txt");
    }

    #[test]
    fn test_s3_key_with_root_path() {
        let mut config = FileServiceConfig::s3(
            "test".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        );
        config.root_path = "/app/files".to_string();
        let provider = S3StorageProvider::new(config).unwrap();

        assert_eq!(provider.get_s3_key("/test.txt"), "app/files/test.txt");
        assert_eq!(provider.get_s3_key("test.txt"), "app/files/test.txt");
    }

    #[test]
    fn test_path_validation() {
        let config = FileServiceConfig::s3(
            "test".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        );
        let provider = S3StorageProvider::new(config).unwrap();

        assert_eq!(provider.validate_path("/test.txt").unwrap(), "/test.txt");
        assert_eq!(provider.validate_path("test.txt").unwrap(), "/test.txt");
        assert_eq!(provider.validate_path("/dir/../test.txt").unwrap(), "/test.txt");
        assert!(provider.validate_path("").is_err());
    }

    #[test]
    fn test_service_info() {
        let config = FileServiceConfig::s3(
            "test".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        );
        let provider = S3StorageProvider::new(config).unwrap();

        let info = provider.service_info();
        assert_eq!(info.name, "test");
        assert_eq!(info.provider, "s3");
        assert!(info.features.contains(&ServiceFeature::PresignedUrls));
        assert!(info.features.contains(&ServiceFeature::MultipartUpload));
    }

    #[test]
    fn test_capabilities() {
        let config = FileServiceConfig::s3(
            "test".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        );
        let provider = S3StorageProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.streaming_upload);
        assert!(caps.streaming_download);
        assert!(caps.multipart_upload);
        assert!(caps.presigned_urls);
        assert!(!caps.directory_operations); // S3 doesn't have true directories
    }
}