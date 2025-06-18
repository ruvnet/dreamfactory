use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// File service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileServiceConfig {
    /// Service name
    pub name: String,
    /// Service description
    pub description: Option<String>,
    /// Storage provider type
    pub provider: StorageProvider,
    /// Provider-specific configuration
    pub provider_config: ProviderConfig,
    /// Root path for file operations
    pub root_path: String,
    /// Maximum file size in bytes
    pub max_file_size: Option<u64>,
    /// Maximum directory depth
    pub max_directory_depth: Option<u32>,
    /// Allowed file types (MIME types or extensions)
    pub allowed_file_types: Option<Vec<String>>,
    /// Blocked file types (MIME types or extensions)
    pub blocked_file_types: Option<Vec<String>>,
    /// Whether to enable file versioning
    pub enable_versioning: bool,
    /// Whether to enable automatic backups
    pub enable_backups: bool,
    /// Whether to enable server-side encryption
    pub enable_encryption: bool,
    /// Default permissions for new files (Unix-style octal)
    pub default_permissions: Option<u32>,
    /// Whether to preserve file permissions on upload
    pub preserve_permissions: bool,
    /// Whether to enable virus scanning
    pub enable_virus_scanning: bool,
    /// Chunk size for multipart uploads (bytes)
    pub chunk_size: u64,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    /// Whether to enable compression
    pub enable_compression: bool,
    /// Compression threshold (files larger than this will be compressed)
    pub compression_threshold: u64,
    /// Cache settings
    pub cache_config: CacheConfig,
    /// Retry settings
    pub retry_config: RetryConfig,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl Default for FileServiceConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            provider: StorageProvider::Local,
            provider_config: ProviderConfig::Local(LocalConfig::default()),
            root_path: "/".to_string(),
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_directory_depth: Some(20),
            allowed_file_types: None,
            blocked_file_types: Some(vec![
                "application/x-executable".to_string(),
                ".exe".to_string(),
                ".bat".to_string(),
                ".sh".to_string(),
            ]),
            enable_versioning: false,
            enable_backups: false,
            enable_encryption: false,
            default_permissions: Some(644),
            preserve_permissions: true,
            enable_virus_scanning: false,
            chunk_size: 5 * 1024 * 1024, // 5MB
            connection_timeout: 30,
            request_timeout: 300, // 5 minutes
            max_concurrent_operations: 10,
            enable_compression: false,
            compression_threshold: 1024 * 1024, // 1MB
            cache_config: CacheConfig::default(),
            retry_config: RetryConfig::default(),
            metadata: HashMap::new(),
        }
    }
}

/// Storage provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageProvider {
    Local,
    S3,
    AzureBlob,
    GoogleCloud,
    Ftp,
    Sftp,
    WebDav,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderConfig {
    Local(LocalConfig),
    S3(S3Config),
    AzureBlob(AzureBlobConfig),
    GoogleCloud(GoogleCloudConfig),
    Ftp(FtpConfig),
    Sftp(SftpConfig),
    WebDav(WebDavConfig),
}

/// Local file system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Root directory path
    pub root_directory: PathBuf,
    /// Whether to create root directory if it doesn't exist
    pub create_root_directory: bool,
    /// Whether to enable symbolic links
    pub enable_symlinks: bool,
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
    /// File system permissions mask
    pub umask: Option<u32>,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            root_directory: PathBuf::from("./files"),
            create_root_directory: true,
            enable_symlinks: false,
            follow_symlinks: false,
            umask: None,
        }
    }
}

/// AWS S3 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// AWS region
    pub region: String,
    /// AWS access key ID
    pub access_key_id: Option<String>,
    /// AWS secret access key
    pub secret_access_key: Option<String>,
    /// AWS session token (for temporary credentials)
    pub session_token: Option<String>,
    /// Custom endpoint URL (for S3-compatible services)
    pub endpoint_url: Option<String>,
    /// Whether to use path-style addressing
    pub path_style: bool,
    /// Server-side encryption
    pub server_side_encryption: Option<String>,
    /// KMS key ID for encryption
    pub kms_key_id: Option<String>,
    /// Storage class
    pub storage_class: Option<String>,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: "".to_string(),
            region: "us-east-1".to_string(),
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            endpoint_url: None,
            path_style: false,
            server_side_encryption: None,
            kms_key_id: None,
            storage_class: None,
        }
    }
}

/// Azure Blob Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureBlobConfig {
    /// Storage account name
    pub account_name: String,
    /// Container name
    pub container_name: String,
    /// Account key
    pub account_key: Option<String>,
    /// SAS token
    pub sas_token: Option<String>,
    /// Custom endpoint URL
    pub endpoint_url: Option<String>,
    /// Access tier
    pub access_tier: Option<String>,
}

impl Default for AzureBlobConfig {
    fn default() -> Self {
        Self {
            account_name: "".to_string(),
            container_name: "".to_string(),
            account_key: None,
            sas_token: None,
            endpoint_url: None,
            access_tier: None,
        }
    }
}

/// Google Cloud Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCloudConfig {
    /// GCS bucket name
    pub bucket: String,
    /// Project ID
    pub project_id: Option<String>,
    /// Service account key (JSON)
    pub service_account_key: Option<String>,
    /// Custom endpoint URL
    pub endpoint_url: Option<String>,
    /// Storage class
    pub storage_class: Option<String>,
}

impl Default for GoogleCloudConfig {
    fn default() -> Self {
        Self {
            bucket: "".to_string(),
            project_id: None,
            service_account_key: None,
            endpoint_url: None,
            storage_class: None,
        }
    }
}

/// FTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConfig {
    /// FTP server hostname
    pub host: String,
    /// FTP server port
    pub port: u16,
    /// Username
    pub username: String,
    /// Password
    pub password: Option<String>,
    /// Whether to use passive mode
    pub passive_mode: bool,
    /// Whether to use TLS/SSL
    pub use_tls: bool,
}

impl Default for FtpConfig {
    fn default() -> Self {
        Self {
            host: "".to_string(),
            port: 21,
            username: "".to_string(),
            password: None,
            passive_mode: true,
            use_tls: false,
        }
    }
}

/// SFTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpConfig {
    /// SFTP server hostname
    pub host: String,
    /// SFTP server port
    pub port: u16,
    /// Username
    pub username: String,
    /// Password
    pub password: Option<String>,
    /// Private key path
    pub private_key_path: Option<PathBuf>,
    /// Private key passphrase
    pub private_key_passphrase: Option<String>,
}

impl Default for SftpConfig {
    fn default() -> Self {
        Self {
            host: "".to_string(),
            port: 22,
            username: "".to_string(),
            password: None,
            private_key_path: None,
            private_key_passphrase: None,
        }
    }
}

/// WebDAV configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavConfig {
    /// WebDAV server URL
    pub url: String,
    /// Username
    pub username: String,
    /// Password
    pub password: Option<String>,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl Default for WebDavConfig {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            username: "".to_string(),
            password: None,
            verify_ssl: true,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether to enable caching
    pub enabled: bool,
    /// Cache TTL in seconds
    pub ttl: u64,
    /// Maximum number of cached entries
    pub max_entries: usize,
    /// Cache file metadata
    pub cache_metadata: bool,
    /// Cache directory listings
    pub cache_listings: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: 300, // 5 minutes
            max_entries: 1000,
            cache_metadata: true,
            cache_listings: true,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    /// Retry delay multiplier (exponential backoff)
    pub delay_multiplier: f64,
    /// Whether to retry on network errors
    pub retry_on_network_error: bool,
    /// Whether to retry on server errors (5xx)
    pub retry_on_server_error: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            delay_multiplier: 2.0,
            retry_on_network_error: true,
            retry_on_server_error: true,
        }
    }
}

impl FileServiceConfig {
    /// Create a new local file service configuration
    pub fn local<P: Into<PathBuf>>(name: String, root_directory: P) -> Self {
        Self {
            name,
            provider: StorageProvider::Local,
            provider_config: ProviderConfig::Local(LocalConfig {
                root_directory: root_directory.into(),
                ..LocalConfig::default()
            }),
            ..Self::default()
        }
    }

    /// Create a new S3 file service configuration
    pub fn s3(name: String, bucket: String, region: String) -> Self {
        Self {
            name,
            provider: StorageProvider::S3,
            provider_config: ProviderConfig::S3(S3Config {
                bucket,
                region,
                ..S3Config::default()
            }),
            ..Self::default()
        }
    }

    /// Create a new Azure Blob file service configuration
    pub fn azure_blob(name: String, account_name: String, container_name: String) -> Self {
        Self {
            name,
            provider: StorageProvider::AzureBlob,
            provider_config: ProviderConfig::AzureBlob(AzureBlobConfig {
                account_name,
                container_name,
                ..AzureBlobConfig::default()
            }),
            ..Self::default()
        }
    }

    /// Create a new Google Cloud Storage file service configuration
    pub fn google_cloud(name: String, bucket: String) -> Self {
        Self {
            name,
            provider: StorageProvider::GoogleCloud,
            provider_config: ProviderConfig::GoogleCloud(GoogleCloudConfig {
                bucket,
                ..GoogleCloudConfig::default()
            }),
            ..Self::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Service name cannot be empty".to_string());
        }

        if self.root_path.is_empty() {
            return Err("Root path cannot be empty".to_string());
        }

        match &self.provider_config {
            ProviderConfig::Local(config) => {
                if !config.root_directory.is_absolute() && !config.create_root_directory {
                    return Err("Local root directory must be absolute or auto-created".to_string());
                }
            }
            ProviderConfig::S3(config) => {
                if config.bucket.is_empty() {
                    return Err("S3 bucket name cannot be empty".to_string());
                }
                if config.region.is_empty() {
                    return Err("S3 region cannot be empty".to_string());
                }
            }
            ProviderConfig::AzureBlob(config) => {
                if config.account_name.is_empty() {
                    return Err("Azure account name cannot be empty".to_string());
                }
                if config.container_name.is_empty() {
                    return Err("Azure container name cannot be empty".to_string());
                }
            }
            ProviderConfig::GoogleCloud(config) => {
                if config.bucket.is_empty() {
                    return Err("Google Cloud bucket name cannot be empty".to_string());
                }
            }
            ProviderConfig::Ftp(config) => {
                if config.host.is_empty() {
                    return Err("FTP host cannot be empty".to_string());
                }
                if config.username.is_empty() {
                    return Err("FTP username cannot be empty".to_string());
                }
            }
            ProviderConfig::Sftp(config) => {
                if config.host.is_empty() {
                    return Err("SFTP host cannot be empty".to_string());
                }
                if config.username.is_empty() {
                    return Err("SFTP username cannot be empty".to_string());
                }
            }
            ProviderConfig::WebDav(config) => {
                if config.url.is_empty() {
                    return Err("WebDAV URL cannot be empty".to_string());
                }
                if config.username.is_empty() {
                    return Err("WebDAV username cannot be empty".to_string());
                }
            }
        }

        if let Some(max_size) = self.max_file_size {
            if max_size == 0 {
                return Err("Maximum file size must be greater than 0".to_string());
            }
        }

        if self.chunk_size == 0 {
            return Err("Chunk size must be greater than 0".to_string());
        }

        if self.connection_timeout == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }

        if self.request_timeout == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }

        if self.max_concurrent_operations == 0 {
            return Err("Max concurrent operations must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileServiceConfig::default();
        assert_eq!(config.name, "default");
        assert_eq!(config.provider, StorageProvider::Local);
        assert!(config.max_file_size.is_some());
        assert!(!config.enable_versioning);
    }

    #[test]
    fn test_local_config() {
        let config = FileServiceConfig::local("test".to_string(), "/tmp/files");
        assert_eq!(config.name, "test");
        assert_eq!(config.provider, StorageProvider::Local);
        
        if let ProviderConfig::Local(local_config) = config.provider_config {
            assert_eq!(local_config.root_directory, PathBuf::from("/tmp/files"));
        } else {
            panic!("Expected local config");
        }
    }

    #[test]
    fn test_s3_config() {
        let config = FileServiceConfig::s3(
            "s3-test".to_string(),
            "my-bucket".to_string(),
            "us-west-2".to_string(),
        );
        assert_eq!(config.name, "s3-test");
        assert_eq!(config.provider, StorageProvider::S3);
        
        if let ProviderConfig::S3(s3_config) = config.provider_config {
            assert_eq!(s3_config.bucket, "my-bucket");
            assert_eq!(s3_config.region, "us-west-2");
        } else {
            panic!("Expected S3 config");
        }
    }

    #[test]
    fn test_config_validation() {
        let mut config = FileServiceConfig::default();
        assert!(config.validate().is_ok());

        config.name = "".to_string();
        assert!(config.validate().is_err());

        config.name = "test".to_string();
        config.max_file_size = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cache_config() {
        let cache_config = CacheConfig::default();
        assert!(cache_config.enabled);
        assert_eq!(cache_config.ttl, 300);
        assert_eq!(cache_config.max_entries, 1000);
    }

    #[test]
    fn test_retry_config() {
        let retry_config = RetryConfig::default();
        assert_eq!(retry_config.max_attempts, 3);
        assert_eq!(retry_config.initial_delay_ms, 100);
        assert_eq!(retry_config.delay_multiplier, 2.0);
    }
}