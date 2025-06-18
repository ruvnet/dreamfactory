use crate::models::{
    config::{FileServiceConfig, LocalConfig, ProviderConfig, StorageProvider as ConfigStorageProvider},
    error::{FileResult, FileServiceError},
    file_info::{DirectoryListing, FileInfo},
    file_operation::{
        BatchOperation, BatchOperationResult, BatchOperationType, CopyOptions, DownloadOptions,
        FileOperationResult, FileStream, ListOptions, MultipartUpload, SortBy, SortOrder,
        UploadOptions, UploadPart,
    },
};
use crate::traits::{
    file_service::{
        AdvancedFileService, FileService, FileServiceInfo, PresignedOperation, SearchCriteria,
        ServiceFeature, ServiceLimits, StorageUsage, StreamingFileService,
    },
    storage_provider::{ProviderCapabilities, ProviderHealth, ProviderStatistics, StorageProvider},
};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::{stream, Stream, StreamExt, TryStreamExt};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use walkdir::WalkDir;

/// Local file system storage provider
pub struct LocalStorageProvider {
    config: FileServiceConfig,
    local_config: LocalConfig,
    root_path: PathBuf,
    initialized: bool,
    statistics: Arc<Mutex<ProviderStatistics>>,
    multipart_uploads: Arc<Mutex<HashMap<String, MultipartUpload>>>,
}

impl LocalStorageProvider {
    /// Create a new local storage provider
    pub fn new(config: FileServiceConfig) -> FileResult<Self> {
        let local_config = match &config.provider_config {
            ProviderConfig::Local(local_config) => local_config.clone(),
            _ => {
                return Err(FileServiceError::ConfigError {
                    message: "Invalid provider config for local storage".to_string(),
                })
            }
        };

        let root_path = local_config.root_directory.clone();

        Ok(Self {
            config,
            local_config,
            root_path,
            initialized: false,
            statistics: Arc::new(Mutex::new(ProviderStatistics::default())),
            multipart_uploads: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get the full file system path for a given relative path
    fn get_full_path(&self, path: &str) -> PathBuf {
        let clean_path = path.strip_prefix('/').unwrap_or(path);
        self.root_path.join(clean_path)
    }

    /// Convert a file system path back to a relative path
    fn get_relative_path(&self, full_path: &Path) -> FileResult<String> {
        let relative = full_path
            .strip_prefix(&self.root_path)
            .map_err(|_| FileServiceError::InvalidPath {
                path: full_path.to_string_lossy().to_string(),
            })?;

        Ok(format!("/{}", relative.to_string_lossy()))
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

    /// Create FileInfo from std::fs::Metadata
    async fn create_file_info(&self, path: &str, full_path: &Path) -> FileResult<FileInfo> {
        let metadata = fs::metadata(full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FileServiceError::FileNotFound {
                    path: path.to_string(),
                }
            } else {
                FileServiceError::IoError {
                    message: e.to_string(),
                }
            }
        })?;

        let name = full_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let created_at = metadata
            .created()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .into();
        let modified_at = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH).into();

        let content_type = if metadata.is_dir() {
            "inode/directory".to_string()
        } else {
            mime_guess::from_path(full_path)
                .first_or_octet_stream()
                .to_string()
        };

        let mut info = FileInfo::new(path.to_string(), name)
            .with_size(metadata.len())
            .with_content_type(content_type)
            .with_timestamps(created_at, modified_at);

        info.is_directory = metadata.is_dir();

        // Add Unix permissions if available
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            info = info.with_permissions(metadata.permissions().mode());
        }

        Ok(info)
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

    /// Apply file filters based on options
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

        // Apply sorting
        match options.sort_by {
            SortBy::Name => filtered.sort_by(|a, b| match options.sort_order {
                SortOrder::Ascending => a.name.cmp(&b.name),
                SortOrder::Descending => b.name.cmp(&a.name),
            }),
            SortBy::Size => filtered.sort_by(|a, b| match options.sort_order {
                SortOrder::Ascending => a.size.cmp(&b.size),
                SortOrder::Descending => b.size.cmp(&a.size),
            }),
            SortBy::Modified => filtered.sort_by(|a, b| match options.sort_order {
                SortOrder::Ascending => a.modified_at.cmp(&b.modified_at),
                SortOrder::Descending => b.modified_at.cmp(&a.modified_at),
            }),
            SortBy::Created => filtered.sort_by(|a, b| match options.sort_order {
                SortOrder::Ascending => a.created_at.cmp(&b.created_at),
                SortOrder::Descending => b.created_at.cmp(&a.created_at),
            }),
            SortBy::Type => filtered.sort_by(|a, b| match options.sort_order {
                SortOrder::Ascending => a.content_type.cmp(&b.content_type),
                SortOrder::Descending => b.content_type.cmp(&a.content_type),
            }),
        }

        // Apply limit
        if let Some(limit) = options.limit {
            filtered.truncate(limit);
        }

        filtered
    }
}

#[async_trait]
impl StorageProvider for LocalStorageProvider {
    async fn initialize(&mut self) -> FileResult<()> {
        if self.initialized {
            return Ok(());
        }

        // Create root directory if it doesn't exist and is configured to do so
        if self.local_config.create_root_directory && !self.root_path.exists() {
            fs::create_dir_all(&self.root_path).await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to create root directory: {}", e),
                }
            })?;
        }

        // Verify root directory exists and is accessible
        if !self.root_path.exists() {
            return Err(FileServiceError::DirectoryNotFound {
                path: self.root_path.to_string_lossy().to_string(),
            });
        }

        if !self.root_path.is_dir() {
            return Err(FileServiceError::InvalidPath {
                path: self.root_path.to_string_lossy().to_string(),
            });
        }

        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> FileResult<()> {
        // Clear any in-progress multipart uploads
        if let Ok(mut uploads) = self.multipart_uploads.lock() {
            uploads.clear();
        }

        self.initialized = false;
        Ok(())
    }

    fn config(&self) -> &FileServiceConfig {
        &self.config
    }

    fn is_ready(&self) -> bool {
        self.initialized && self.root_path.exists()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming_upload: true,
            streaming_download: true,
            multipart_upload: true,
            resumable_upload: false,
            directory_operations: true,
            metadata_support: true,
            permissions_support: cfg!(unix),
            encryption_support: false,
            compression_support: false,
            versioning_support: false,
            presigned_urls: false,
            batch_operations: true,
            search_support: true,
            locking_support: false,
            atomic_operations: true,
            transaction_support: false,
            max_file_size: None,
            max_path_length: Some(4096),
            max_metadata_size: Some(64 * 1024),
        }
    }

    async fn health_check(&self) -> FileResult<ProviderHealth> {
        let start_time = std::time::Instant::now();
        
        let healthy = self.is_ready() && {
            // Try to create a temporary file to test write access
            let test_path = self.root_path.join(".health_check");
            match fs::write(&test_path, b"health_check").await {
                Ok(_) => {
                    let _ = fs::remove_file(&test_path).await;
                    true
                }
                Err(_) => false,
            }
        };

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ProviderHealth {
            healthy,
            checked_at: Utc::now(),
            response_time_ms,
            error_message: if healthy {
                None
            } else {
                Some("Storage not accessible".to_string())
            },
            details: std::collections::HashMap::new(),
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
        let local_config = match &config.provider_config {
            ProviderConfig::Local(local_config) => local_config.clone(),
            _ => {
                return Err(FileServiceError::ConfigError {
                    message: "Invalid provider config for local storage".to_string(),
                })
            }
        };

        self.config = config;
        self.local_config = local_config;
        self.root_path = self.local_config.root_directory.clone();
        self.initialized = false;

        self.initialize().await
    }
}

#[async_trait]
impl FileService for LocalStorageProvider {
    fn service_info(&self) -> FileServiceInfo {
        FileServiceInfo {
            name: self.config.name.clone(),
            version: "1.0.0".to_string(),
            provider: "local".to_string(),
            features: vec![
                ServiceFeature::MultipartUpload,
                ServiceFeature::Streaming,
                ServiceFeature::DirectoryOperations,
                ServiceFeature::Metadata,
                ServiceFeature::BatchOperations,
                ServiceFeature::Search,
                ServiceFeature::Checksums,
            ],
            limits: ServiceLimits {
                max_file_size: self.config.max_file_size,
                max_files_per_directory: None,
                max_directory_depth: self.config.max_directory_depth.map(|d| d as u64),
                max_path_length: Some(4096),
                max_metadata_size: Some(64 * 1024),
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
                service: "local".to_string(),
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
        let full_path = self.get_full_path(&normalized_path);

        // Check if file exists and overwrite is not allowed
        if !options.overwrite && full_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileAlreadyExists {
                path: normalized_path,
            });
        }

        // Create parent directories if needed
        if options.create_parents {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    self.record_operation(false, None);
                    FileServiceError::IoError {
                        message: format!("Failed to create parent directories: {}", e),
                    }
                })?;
            }
        }

        // Write file content
        fs::write(&full_path, &content).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to write file: {}", e),
            }
        })?;

        // Set permissions if specified
        #[cfg(unix)]
        if let Some(permissions) = options.permissions {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(permissions);
            std::fs::set_permissions(&full_path, perms).map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to set permissions: {}", e),
                }
            })?;
        }

        let bytes_transferred = content.len() as u64;
        self.record_operation(true, Some(bytes_transferred));

        Ok(FileOperationResult::success("upload".to_string(), normalized_path)
            .with_bytes_transferred(bytes_transferred))
    }

    async fn upload_stream(
        &self,
        path: &str,
        mut stream: FileStream,
        _size: Option<u64>,
        options: UploadOptions,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);

        // Check if file exists and overwrite is not allowed
        if !options.overwrite && full_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileAlreadyExists {
                path: normalized_path,
            });
        }

        // Create parent directories if needed
        if options.create_parents {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    self.record_operation(false, None);
                    FileServiceError::IoError {
                        message: format!("Failed to create parent directories: {}", e),
                    }
                })?;
            }
        }

        // Create and write to file
        let mut file = File::create(&full_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to create file: {}", e),
            }
        })?;

        let mut bytes_transferred = 0u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                self.record_operation(false, Some(bytes_transferred));
                FileServiceError::IoError {
                    message: format!("Failed to read stream: {}", e),
                }
            })?;

            file.write_all(&chunk).await.map_err(|e| {
                self.record_operation(false, Some(bytes_transferred));
                FileServiceError::IoError {
                    message: format!("Failed to write to file: {}", e),
                }
            })?;

            bytes_transferred += chunk.len() as u64;
        }

        file.flush().await.map_err(|e| {
            self.record_operation(false, Some(bytes_transferred));
            FileServiceError::IoError {
                message: format!("Failed to flush file: {}", e),
            }
        })?;

        // Set permissions if specified
        #[cfg(unix)]
        if let Some(permissions) = options.permissions {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(permissions);
            std::fs::set_permissions(&full_path, perms).map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to set permissions: {}", e),
                }
            })?;
        }

        self.record_operation(true, Some(bytes_transferred));

        Ok(FileOperationResult::success("upload_stream".to_string(), normalized_path)
            .with_bytes_transferred(bytes_transferred))
    }

    async fn download_bytes(
        &self,
        path: &str,
        options: DownloadOptions,
    ) -> FileResult<(Bytes, FileInfo)> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);

        if !full_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_path,
            });
        }

        let file_info = self.create_file_info(&normalized_path, &full_path).await?;

        if file_info.is_directory {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_path,
            });
        }

        let content = if let Some((start, end)) = options.range {
            // Range download
            let mut file = File::open(&full_path).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::IoError {
                    message: format!("Failed to open file: {}", e),
                }
            })?;

            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();

            // Seek to start position
            use tokio::io::AsyncSeekExt;
            reader.seek(std::io::SeekFrom::Start(start)).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::IoError {
                    message: format!("Failed to seek: {}", e),
                }
            })?;

            // Read specified range
            let bytes_to_read = end.map(|e| (e - start + 1) as usize).unwrap_or(usize::MAX);
            reader.take(bytes_to_read as u64).read_to_end(&mut buffer).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::IoError {
                    message: format!("Failed to read file: {}", e),
                }
            })?;

            Bytes::from(buffer)
        } else {
            // Full file download
            let bytes = fs::read(&full_path).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::IoError {
                    message: format!("Failed to read file: {}", e),
                }
            })?;

            Bytes::from(bytes)
        };

        let bytes_transferred = content.len() as u64;
        self.record_operation(true, Some(bytes_transferred));

        Ok((content, file_info))
    }

    async fn download_stream(
        &self,
        path: &str,
        options: DownloadOptions,
    ) -> FileResult<(FileStream, FileInfo)> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);

        if !full_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_path,
            });
        }

        let file_info = self.create_file_info(&normalized_path, &full_path).await?;

        if file_info.is_directory {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_path,
            });
        }

        let file = File::open(&full_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to open file: {}", e),
            }
        })?;

        let stream: FileStream = if let Some((start, end)) = options.range {
            // Range stream
            let mut reader = BufReader::new(file);
            
            // Seek to start position
            use tokio::io::AsyncSeekExt;
            reader.seek(std::io::SeekFrom::Start(start)).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::IoError {
                    message: format!("Failed to seek: {}", e),
                }
            })?;

            let bytes_to_read = end.map(|e| e - start + 1).unwrap_or(u64::MAX);
            let limited_reader = reader.take(bytes_to_read);

            Box::pin(tokio_util::codec::FramedRead::new(
                limited_reader,
                tokio_util::codec::BytesCodec::new(),
            ).map_ok(|bytes| bytes.freeze()).map_err(std::io::Error::from))
        } else {
            // Full file stream
            let reader = BufReader::new(file);
            Box::pin(tokio_util::codec::FramedRead::new(
                reader,
                tokio_util::codec::BytesCodec::new(),
            ).map_ok(|bytes| bytes.freeze()).map_err(std::io::Error::from))
        };

        self.record_operation(true, None);

        Ok((stream, file_info))
    }

    async fn get_file_info(&self, path: &str) -> FileResult<FileInfo> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);

        if !full_path.exists() {
            return Err(FileServiceError::FileNotFound {
                path: normalized_path,
            });
        }

        self.create_file_info(&normalized_path, &full_path).await
    }

    async fn exists(&self, path: &str) -> FileResult<bool> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);
        Ok(full_path.exists())
    }

    async fn delete_file(&self, path: &str) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);

        if !full_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_path,
            });
        }

        if full_path.is_dir() {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_path,
            });
        }

        fs::remove_file(&full_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to delete file: {}", e),
            }
        })?;

        self.record_operation(true, None);

        Ok(FileOperationResult::success("delete".to_string(), normalized_path))
    }

    async fn copy_file(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult> {
        let normalized_source = self.validate_path(source)?;
        let normalized_dest = self.validate_path(destination)?;
        let source_path = self.get_full_path(&normalized_source);
        let dest_path = self.get_full_path(&normalized_dest);

        if !source_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_source,
            });
        }

        if source_path.is_dir() {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_source,
            });
        }

        if !options.overwrite && dest_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileAlreadyExists {
                path: normalized_dest,
            });
        }

        // Create parent directories if needed
        if options.create_parents {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    self.record_operation(false, None);
                    FileServiceError::IoError {
                        message: format!("Failed to create parent directories: {}", e),
                    }
                })?;
            }
        }

        fs::copy(&source_path, &dest_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to copy file: {}", e),
            }
        })?;

        // Preserve metadata if requested
        if options.preserve_metadata {
            let metadata = fs::metadata(&source_path).await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to read source metadata: {}", e),
                }
            })?;

            // Set modified time
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let accessed = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
            
            filetime::set_file_times(
                &dest_path,
                filetime::FileTime::from_system_time(accessed),
                filetime::FileTime::from_system_time(modified),
            ).map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to set file times: {}", e),
                }
            })?;

            // Set permissions if on Unix and requested
            #[cfg(unix)]
            if options.preserve_permissions {
                use std::os::unix::fs::PermissionsExt;
                let perms = metadata.permissions();
                std::fs::set_permissions(&dest_path, perms).map_err(|e| {
                    FileServiceError::IoError {
                        message: format!("Failed to set permissions: {}", e),
                    }
                })?;
            }
        }

        let bytes_transferred = fs::metadata(&dest_path).await
            .map(|m| m.len())
            .unwrap_or(0);

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
        let normalized_source = self.validate_path(source)?;
        let normalized_dest = self.validate_path(destination)?;
        let source_path = self.get_full_path(&normalized_source);
        let dest_path = self.get_full_path(&normalized_dest);

        if !source_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileNotFound {
                path: normalized_source,
            });
        }

        if source_path.is_dir() {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_source,
            });
        }

        if !options.overwrite && dest_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::FileAlreadyExists {
                path: normalized_dest,
            });
        }

        // Create parent directories if needed
        if options.create_parents {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    self.record_operation(false, None);
                    FileServiceError::IoError {
                        message: format!("Failed to create parent directories: {}", e),
                    }
                })?;
            }
        }

        let bytes_transferred = fs::metadata(&source_path).await
            .map(|m| m.len())
            .unwrap_or(0);

        fs::rename(&source_path, &dest_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to move file: {}", e),
            }
        })?;

        self.record_operation(true, Some(bytes_transferred));

        Ok(FileOperationResult::success("move".to_string(), normalized_dest)
            .with_bytes_transferred(bytes_transferred))
    }

    async fn create_directory(&self, path: &str) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);

        if full_path.exists() {
            if full_path.is_dir() {
                self.record_operation(true, None);
                return Ok(FileOperationResult::success("create_directory".to_string(), normalized_path));
            } else {
                self.record_operation(false, None);
                return Err(FileServiceError::FileAlreadyExists {
                    path: normalized_path,
                });
            }
        }

        fs::create_dir_all(&full_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to create directory: {}", e),
            }
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
        let full_path = self.get_full_path(&normalized_path);

        if !full_path.exists() {
            return Err(FileServiceError::DirectoryNotFound {
                path: normalized_path,
            });
        }

        if !full_path.is_dir() {
            return Err(FileServiceError::InvalidPath {
                path: normalized_path,
            });
        }

        let mut entries = Vec::new();

        if options.recursive {
            // Recursive listing
            for entry in WalkDir::new(&full_path)
                .min_depth(1)
                .max_depth(if options.recursive { 100 } else { 1 })
            {
                let entry = entry.map_err(|e| FileServiceError::IoError {
                    message: format!("Failed to read directory entry: {}", e),
                })?;

                let entry_path = entry.path();
                let relative_path = self.get_relative_path(entry_path)?;
                
                if let Ok(file_info) = self.create_file_info(&relative_path, entry_path).await {
                    entries.push(file_info);
                }
            }
        } else {
            // Non-recursive listing
            let mut dir_stream = fs::read_dir(&full_path).await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to read directory: {}", e),
                }
            })?;

            while let Some(entry) = dir_stream.next_entry().await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to read directory entry: {}", e),
                }
            })? {
                let entry_path = entry.path();
                let relative_path = self.get_relative_path(&entry_path)?;
                
                if let Ok(file_info) = self.create_file_info(&relative_path, &entry_path).await {
                    entries.push(file_info);
                }
            }
        }

        // Apply filters and sorting
        let filtered_entries = self.apply_file_filters(entries, &options);

        Ok(DirectoryListing::new(normalized_path, filtered_entries))
    }

    async fn delete_directory(
        &self,
        path: &str,
        recursive: bool,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        let full_path = self.get_full_path(&normalized_path);

        if !full_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::DirectoryNotFound {
                path: normalized_path,
            });
        }

        if !full_path.is_dir() {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_path,
            });
        }

        if recursive {
            fs::remove_dir_all(&full_path).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::IoError {
                    message: format!("Failed to delete directory recursively: {}", e),
                }
            })?;
        } else {
            fs::remove_dir(&full_path).await.map_err(|e| {
                self.record_operation(false, None);
                FileServiceError::IoError {
                    message: format!("Failed to delete directory: {}", e),
                }
            })?;
        }

        self.record_operation(true, None);

        Ok(FileOperationResult::success("delete_directory".to_string(), normalized_path))
    }

    async fn copy_directory(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult> {
        let normalized_source = self.validate_path(source)?;
        let normalized_dest = self.validate_path(destination)?;
        let source_path = self.get_full_path(&normalized_source);
        let dest_path = self.get_full_path(&normalized_dest);

        if !source_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::DirectoryNotFound {
                path: normalized_source,
            });
        }

        if !source_path.is_dir() {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_source,
            });
        }

        if !options.overwrite && dest_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::DirectoryAlreadyExists {
                path: normalized_dest,
            });
        }

        // Create destination directory
        fs::create_dir_all(&dest_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to create destination directory: {}", e),
            }
        })?;

        let mut bytes_transferred = 0u64;

        // Copy directory contents recursively
        if options.recursive {
            for entry in WalkDir::new(&source_path) {
                let entry = entry.map_err(|e| FileServiceError::IoError {
                    message: format!("Failed to read directory entry: {}", e),
                })?;

                let source_entry_path = entry.path();
                let relative_path = source_entry_path
                    .strip_prefix(&source_path)
                    .map_err(|_| FileServiceError::InternalError {
                        message: "Failed to compute relative path".to_string(),
                    })?;
                let dest_entry_path = dest_path.join(relative_path);

                if source_entry_path.is_dir() {
                    fs::create_dir_all(&dest_entry_path).await.map_err(|e| {
                        FileServiceError::IoError {
                            message: format!("Failed to create directory: {}", e),
                        }
                    })?;
                } else {
                    if let Some(parent) = dest_entry_path.parent() {
                        fs::create_dir_all(parent).await.map_err(|e| {
                            FileServiceError::IoError {
                                message: format!("Failed to create parent directory: {}", e),
                            }
                        })?;
                    }

                    let file_size = fs::copy(&source_entry_path, &dest_entry_path).await
                        .map_err(|e| FileServiceError::IoError {
                            message: format!("Failed to copy file: {}", e),
                        })?;

                    bytes_transferred += file_size;

                    // Preserve metadata if requested
                    if options.preserve_metadata {
                        let metadata = fs::metadata(&source_entry_path).await.map_err(|e| {
                            FileServiceError::IoError {
                                message: format!("Failed to read source metadata: {}", e),
                            }
                        })?;

                        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let accessed = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
                        
                        filetime::set_file_times(
                            &dest_entry_path,
                            filetime::FileTime::from_system_time(accessed),
                            filetime::FileTime::from_system_time(modified),
                        ).map_err(|e| {
                            FileServiceError::IoError {
                                message: format!("Failed to set file times: {}", e),
                            }
                        })?;

                        #[cfg(unix)]
                        if options.preserve_permissions {
                            use std::os::unix::fs::PermissionsExt;
                            let perms = metadata.permissions();
                            std::fs::set_permissions(&dest_entry_path, perms).map_err(|e| {
                                FileServiceError::IoError {
                                    message: format!("Failed to set permissions: {}", e),
                                }
                            })?;
                        }
                    }
                }
            }
        } else {
            // Copy only direct children
            let mut dir_stream = fs::read_dir(&source_path).await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to read source directory: {}", e),
                }
            })?;

            while let Some(entry) = dir_stream.next_entry().await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to read directory entry: {}", e),
                }
            })? {
                let source_entry_path = entry.path();
                let dest_entry_path = dest_path.join(entry.file_name());

                if source_entry_path.is_dir() {
                    fs::create_dir_all(&dest_entry_path).await.map_err(|e| {
                        FileServiceError::IoError {
                            message: format!("Failed to create directory: {}", e),
                        }
                    })?;
                } else {
                    let file_size = fs::copy(&source_entry_path, &dest_entry_path).await
                        .map_err(|e| FileServiceError::IoError {
                            message: format!("Failed to copy file: {}", e),
                        })?;

                    bytes_transferred += file_size;
                }
            }
        }

        self.record_operation(true, Some(bytes_transferred));

        Ok(FileOperationResult::success("copy_directory".to_string(), normalized_dest)
            .with_bytes_transferred(bytes_transferred))
    }

    async fn move_directory(
        &self,
        source: &str,
        destination: &str,
        options: CopyOptions,
    ) -> FileResult<FileOperationResult> {
        let normalized_source = self.validate_path(source)?;
        let normalized_dest = self.validate_path(destination)?;
        let source_path = self.get_full_path(&normalized_source);
        let dest_path = self.get_full_path(&normalized_dest);

        if !source_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::DirectoryNotFound {
                path: normalized_source,
            });
        }

        if !source_path.is_dir() {
            self.record_operation(false, None);
            return Err(FileServiceError::InvalidPath {
                path: normalized_source,
            });
        }

        if !options.overwrite && dest_path.exists() {
            self.record_operation(false, None);
            return Err(FileServiceError::DirectoryAlreadyExists {
                path: normalized_dest,
            });
        }

        // Create parent directories if needed
        if options.create_parents {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    self.record_operation(false, None);
                    FileServiceError::IoError {
                        message: format!("Failed to create parent directories: {}", e),
                    }
                })?;
            }
        }

        // Calculate bytes transferred before move
        let bytes_transferred = calculate_directory_size(&source_path).await.unwrap_or(0);

        fs::rename(&source_path, &dest_path).await.map_err(|e| {
            self.record_operation(false, None);
            FileServiceError::IoError {
                message: format!("Failed to move directory: {}", e),
            }
        })?;

        self.record_operation(true, Some(bytes_transferred));

        Ok(FileOperationResult::success("move_directory".to_string(), normalized_dest)
            .with_bytes_transferred(bytes_transferred))
    }

    // Multipart upload operations (simplified for local storage)
    async fn initiate_multipart_upload(
        &self,
        path: &str,
        options: UploadOptions,
    ) -> FileResult<MultipartUpload> {
        let normalized_path = self.validate_path(path)?;
        let upload_id = uuid::Uuid::new_v4().to_string();

        // Check if file exists and overwrite is not allowed
        let full_path = self.get_full_path(&normalized_path);
        if !options.overwrite && full_path.exists() {
            return Err(FileServiceError::FileAlreadyExists {
                path: normalized_path,
            });
        }

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

        // Create a temporary file for this part
        let temp_dir = std::env::temp_dir();
        let part_path = temp_dir.join(format!("{}_{}", upload_id, part_number));

        fs::write(&part_path, &content).await.map_err(|e| {
            FileServiceError::IoError {
                message: format!("Failed to write part: {}", e),
            }
        })?;

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

        let full_path = self.get_full_path(&upload.path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to create parent directories: {}", e),
                }
            })?;
        }

        // Combine all parts into final file
        let mut final_file = File::create(&full_path).await.map_err(|e| {
            FileServiceError::IoError {
                message: format!("Failed to create final file: {}", e),
            }
        })?;

        let mut total_bytes = 0u64;
        let temp_dir = std::env::temp_dir();

        for part in &upload.parts {
            let part_path = temp_dir.join(format!("{}_{}", upload_id, part.part_number));
            let part_content = fs::read(&part_path).await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to read part: {}", e),
                }
            })?;

            final_file.write_all(&part_content).await.map_err(|e| {
                FileServiceError::IoError {
                    message: format!("Failed to write to final file: {}", e),
                }
            })?;

            total_bytes += part_content.len() as u64;

            // Clean up part file
            let _ = fs::remove_file(&part_path).await;
        }

        final_file.flush().await.map_err(|e| {
            FileServiceError::IoError {
                message: format!("Failed to flush final file: {}", e),
            }
        })?;

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

        // Clean up part files
        let temp_dir = std::env::temp_dir();
        for part in &upload.parts {
            let part_path = temp_dir.join(format!("{}_{}", upload_id, part.part_number));
            let _ = fs::remove_file(&part_path).await;
        }

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
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        match operation.operation {
            BatchOperationType::Delete => {
                for source in &operation.sources {
                    match self.delete_file(source).await {
                        Ok(result) => {
                            results.push(result);
                            successful += 1;
                        }
                        Err(e) => {
                            results.push(FileOperationResult::failure(
                                "delete".to_string(),
                                source.clone(),
                                e.to_string(),
                            ));
                            failed += 1;
                        }
                    }
                }
            }
            BatchOperationType::Copy => {
                if let Some(destination) = &operation.destination {
                    for (i, source) in operation.sources.iter().enumerate() {
                        let dest = if operation.sources.len() == 1 {
                            destination.clone()
                        } else {
                            format!("{}/{}", destination, i)
                        };

                        match self.copy_file(source, &dest, CopyOptions::default()).await {
                            Ok(result) => {
                                results.push(result);
                                successful += 1;
                            }
                            Err(e) => {
                                results.push(FileOperationResult::failure(
                                    "copy".to_string(),
                                    source.clone(),
                                    e.to_string(),
                                ));
                                failed += 1;
                            }
                        }
                    }
                }
            }
            BatchOperationType::Move => {
                if let Some(destination) = &operation.destination {
                    for (i, source) in operation.sources.iter().enumerate() {
                        let dest = if operation.sources.len() == 1 {
                            destination.clone()
                        } else {
                            format!("{}/{}", destination, i)
                        };

                        match self.move_file(source, &dest, CopyOptions::default()).await {
                            Ok(result) => {
                                results.push(result);
                                successful += 1;
                            }
                            Err(e) => {
                                results.push(FileOperationResult::failure(
                                    "move".to_string(),
                                    source.clone(),
                                    e.to_string(),
                                ));
                                failed += 1;
                            }
                        }
                    }
                }
            }
            BatchOperationType::Archive | BatchOperationType::Extract => {
                // Not implemented for local storage
                return Err(FileServiceError::InternalError {
                    message: "Archive operations not supported by local storage".to_string(),
                });
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(BatchOperationResult {
            total: operation.sources.len(),
            successful,
            failed,
            results,
            timestamp: Utc::now(),
            duration_ms,
        })
    }

    async fn get_storage_usage(&self, path: Option<&str>) -> FileResult<StorageUsage> {
        let target_path = if let Some(path) = path {
            let normalized = self.validate_path(path)?;
            self.get_full_path(&normalized)
        } else {
            self.root_path.clone()
        };

        if !target_path.exists() {
            return Err(FileServiceError::DirectoryNotFound {
                path: target_path.to_string_lossy().to_string(),
            });
        }

        let (used_bytes, file_count, directory_count) = if target_path.is_dir() {
            calculate_directory_usage(&target_path).await.unwrap_or((0, 0, 0))
        } else {
            let size = fs::metadata(&target_path).await
                .map(|m| m.len())
                .unwrap_or(0);
            (size, 1, 0)
        };

        Ok(StorageUsage {
            total_bytes: None, // Local storage doesn't have a fixed total
            used_bytes,
            available_bytes: None, // Would need to check filesystem free space
            file_count,
            directory_count,
        })
    }

    async fn search_files(&self, criteria: SearchCriteria) -> FileResult<Vec<FileInfo>> {
        let search_path = self.validate_path(&criteria.path)?;
        let full_path = self.get_full_path(&search_path);

        if !full_path.exists() {
            return Err(FileServiceError::DirectoryNotFound {
                path: search_path,
            });
        }

        let mut results = Vec::new();
        let walker = if criteria.recursive {
            WalkDir::new(&full_path).min_depth(1)
        } else {
            WalkDir::new(&full_path).min_depth(1).max_depth(1)
        };

        for entry in walker {
            let entry = entry.map_err(|e| FileServiceError::IoError {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            let entry_path = entry.path();
            let relative_path = self.get_relative_path(entry_path)?;
            
            if let Ok(file_info) = self.create_file_info(&relative_path, entry_path).await {
                // Apply filters
                let mut matches = true;

                // Name pattern filter
                if let Some(pattern) = &criteria.name_pattern {
                    if !glob_match(pattern, &file_info.name) {
                        matches = false;
                    }
                }

                // Content type filter
                if let Some(pattern) = &criteria.content_type_pattern {
                    if !file_info.content_type.contains(pattern) {
                        matches = false;
                    }
                }

                // Size filters
                if let Some(min_size) = criteria.min_size {
                    if file_info.size < min_size {
                        matches = false;
                    }
                }

                if let Some(max_size) = criteria.max_size {
                    if file_info.size > max_size {
                        matches = false;
                    }
                }

                // Date filters
                if let Some(modified_after) = criteria.modified_after {
                    if file_info.modified_at < modified_after {
                        matches = false;
                    }
                }

                if let Some(modified_before) = criteria.modified_before {
                    if file_info.modified_at > modified_before {
                        matches = false;
                    }
                }

                if matches {
                    results.push(file_info);
                }

                // Check limit
                if let Some(limit) = criteria.limit {
                    if results.len() >= limit {
                        break;
                    }
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
        Err(FileServiceError::InternalError {
            message: "Presigned URLs not supported by local storage".to_string(),
        })
    }

    async fn set_metadata(
        &self,
        path: &str,
        _metadata: HashMap<String, String>,
    ) -> FileResult<FileOperationResult> {
        let normalized_path = self.validate_path(path)?;
        
        // Local storage doesn't support extended metadata
        // This would typically be stored in extended attributes or a separate metadata file
        
        Ok(FileOperationResult::success("set_metadata".to_string(), normalized_path))
    }

    async fn get_metadata(&self, _path: &str) -> FileResult<HashMap<String, String>> {
        // Local storage doesn't support extended metadata
        Ok(HashMap::new())
    }
}

// Helper functions

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

/// Calculate directory size recursively
async fn calculate_directory_size(path: &Path) -> std::io::Result<u64> {
    let mut total_size = 0;
    
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let metadata = fs::metadata(entry.path()).await?;
            total_size += metadata.len();
        }
    }
    
    Ok(total_size)
}

/// Calculate directory usage statistics
async fn calculate_directory_usage(path: &Path) -> std::io::Result<(u64, u64, u64)> {
    let mut total_size = 0;
    let mut file_count = 0;
    let mut directory_count = 0;
    
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let metadata = fs::metadata(entry.path()).await?;
            total_size += metadata.len();
            file_count += 1;
        } else if entry.file_type().is_dir() {
            directory_count += 1;
        }
    }
    
    Ok((total_size, file_count, directory_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio_test;

    async fn create_test_provider() -> (LocalStorageProvider, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = FileServiceConfig::local(
            "test".to_string(),
            temp_dir.path().to_path_buf(),
        );
        let mut provider = LocalStorageProvider::new(config).unwrap();
        provider.initialize().await.unwrap();
        (provider, temp_dir)
    }

    #[tokio::test]
    async fn test_local_provider_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileServiceConfig::local(
            "test".to_string(),
            temp_dir.path().to_path_buf(),
        );
        let mut provider = LocalStorageProvider::new(config).unwrap();
        
        assert!(!provider.is_ready());
        provider.initialize().await.unwrap();
        assert!(provider.is_ready());
    }

    #[tokio::test]
    async fn test_file_upload_and_download() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        let content = Bytes::from("Hello, World!");
        let options = UploadOptions::default();
        
        // Test upload
        let result = provider.upload_bytes("/test.txt", content.clone(), options).await;
        assert!(result.is_ok());
        
        // Test download
        let (downloaded_content, file_info) = provider
            .download_bytes("/test.txt", DownloadOptions::default())
            .await
            .unwrap();
        
        assert_eq!(downloaded_content, content);
        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.size, content.len() as u64);
    }

    #[tokio::test]
    async fn test_file_exists() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        assert!(!provider.exists("/nonexistent.txt").await.unwrap());
        
        let content = Bytes::from("test");
        provider.upload_bytes("/test.txt", content, UploadOptions::default()).await.unwrap();
        
        assert!(provider.exists("/test.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_file_deletion() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        let content = Bytes::from("test");
        provider.upload_bytes("/test.txt", content, UploadOptions::default()).await.unwrap();
        
        assert!(provider.exists("/test.txt").await.unwrap());
        
        let result = provider.delete_file("/test.txt").await;
        assert!(result.is_ok());
        
        assert!(!provider.exists("/test.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_directory_operations() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        // Create directory
        let result = provider.create_directory("/testdir").await;
        assert!(result.is_ok());
        
        // List empty directory
        let listing = provider.list_directory("/testdir", ListOptions::default()).await.unwrap();
        assert_eq!(listing.entries.len(), 0);
        
        // Add files to directory
        let content = Bytes::from("test");
        provider.upload_bytes("/testdir/file1.txt", content.clone(), UploadOptions::default()).await.unwrap();
        provider.upload_bytes("/testdir/file2.txt", content, UploadOptions::default()).await.unwrap();
        
        // List directory with files
        let listing = provider.list_directory("/testdir", ListOptions::default()).await.unwrap();
        assert_eq!(listing.entries.len(), 2);
        
        // Delete directory
        let result = provider.delete_directory("/testdir", true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_copy_and_move() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        let content = Bytes::from("test content");
        provider.upload_bytes("/source.txt", content.clone(), UploadOptions::default()).await.unwrap();
        
        // Test copy
        let result = provider.copy_file("/source.txt", "/copy.txt", CopyOptions::default()).await;
        assert!(result.is_ok());
        
        assert!(provider.exists("/source.txt").await.unwrap());
        assert!(provider.exists("/copy.txt").await.unwrap());
        
        // Test move
        let result = provider.move_file("/copy.txt", "/moved.txt", CopyOptions::default()).await;
        assert!(result.is_ok());
        
        assert!(!provider.exists("/copy.txt").await.unwrap());
        assert!(provider.exists("/moved.txt").await.unwrap());
        
        // Verify content
        let (downloaded_content, _) = provider
            .download_bytes("/moved.txt", DownloadOptions::default())
            .await
            .unwrap();
        assert_eq!(downloaded_content, content);
    }

    #[tokio::test]
    async fn test_multipart_upload() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        // Initiate multipart upload
        let upload = provider
            .initiate_multipart_upload("/large_file.txt", UploadOptions::default())
            .await
            .unwrap();
        
        // Upload parts
        let part1 = provider
            .upload_part(&upload.upload_id, 1, Bytes::from("part1"))
            .await
            .unwrap();
        
        let part2 = provider
            .upload_part(&upload.upload_id, 2, Bytes::from("part2"))
            .await
            .unwrap();
        
        // Complete upload
        let result = provider
            .complete_multipart_upload(&upload.upload_id, vec![part1, part2])
            .await;
        assert!(result.is_ok());
        
        // Verify file exists and has correct content
        assert!(provider.exists("/large_file.txt").await.unwrap());
        let (content, _) = provider
            .download_bytes("/large_file.txt", DownloadOptions::default())
            .await
            .unwrap();
        assert_eq!(content, Bytes::from("part1part2"));
    }

    #[tokio::test]
    async fn test_storage_usage() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        // Upload some files
        provider.upload_bytes("/file1.txt", Bytes::from("content1"), UploadOptions::default()).await.unwrap();
        provider.upload_bytes("/file2.txt", Bytes::from("content2"), UploadOptions::default()).await.unwrap();
        
        let usage = provider.get_storage_usage(None).await.unwrap();
        assert_eq!(usage.file_count, 2);
        assert_eq!(usage.used_bytes, 16); // "content1" + "content2"
    }

    #[tokio::test]
    async fn test_file_search() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        // Create test files
        provider.upload_bytes("/test1.txt", Bytes::from("content"), UploadOptions::default()).await.unwrap();
        provider.upload_bytes("/test2.log", Bytes::from("content"), UploadOptions::default()).await.unwrap();
        provider.upload_bytes("/other.txt", Bytes::from("content"), UploadOptions::default()).await.unwrap();
        
        // Search for .txt files
        let criteria = SearchCriteria {
            path: "/".to_string(),
            recursive: false,
            name_pattern: Some("*.txt".to_string()),
            content_type_pattern: None,
            min_size: None,
            max_size: None,
            modified_after: None,
            modified_before: None,
            metadata_filters: HashMap::new(),
            limit: None,
        };
        
        let results = provider.search_files(criteria).await.unwrap();
        assert_eq!(results.len(), 2);
        
        let names: Vec<_> = results.iter().map(|f| &f.name).collect();
        assert!(names.contains(&&"test1.txt".to_string()));
        assert!(names.contains(&&"other.txt".to_string()));
    }

    #[tokio::test]
    async fn test_health_check() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        let health = provider.health_check().await;
        assert!(health.is_ok());
        
        let health_info = StorageProvider::health_check(&provider).await.unwrap();
        assert!(health_info.healthy);
        assert!(health_info.response_time_ms > 0);
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        // Create test files
        provider.upload_bytes("/file1.txt", Bytes::from("content1"), UploadOptions::default()).await.unwrap();
        provider.upload_bytes("/file2.txt", Bytes::from("content2"), UploadOptions::default()).await.unwrap();
        
        // Batch delete
        let batch_op = BatchOperation {
            operation: BatchOperationType::Delete,
            sources: vec!["/file1.txt".to_string(), "/file2.txt".to_string()],
            destination: None,
            options: HashMap::new(),
        };
        
        let result = provider.batch_operation(batch_op).await.unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 0);
        
        // Verify files are deleted
        assert!(!provider.exists("/file1.txt").await.unwrap());
        assert!(!provider.exists("/file2.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_path_validation() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        // Test valid paths
        assert_eq!(provider.validate_path("/test.txt").unwrap(), "/test.txt");
        assert_eq!(provider.validate_path("test.txt").unwrap(), "/test.txt");
        assert_eq!(provider.validate_path("/dir/test.txt").unwrap(), "/dir/test.txt");
        
        // Test path normalization
        assert_eq!(provider.validate_path("/dir/../test.txt").unwrap(), "/test.txt");
        assert_eq!(provider.validate_path("/dir/./test.txt").unwrap(), "/dir/test.txt");
        
        // Test empty path
        assert!(provider.validate_path("").is_err());
    }

    #[tokio::test]
    async fn test_capabilities() {
        let (provider, _temp_dir) = create_test_provider().await;
        
        let caps = provider.capabilities();
        assert!(caps.streaming_upload);
        assert!(caps.streaming_download);
        assert!(caps.multipart_upload);
        assert!(caps.directory_operations);
        assert!(caps.batch_operations);
        assert!(!caps.presigned_urls); // Local storage doesn't support presigned URLs
    }
}

// Additional dependencies needed for the implementation
// Add to Cargo.toml:
// tokio-util = { version = "0.7", features = ["codec"] }
// filetime = "0.2"