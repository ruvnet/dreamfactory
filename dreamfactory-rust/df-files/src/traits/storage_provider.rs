use crate::models::{
    config::FileServiceConfig,
    error::{FileResult, FileServiceError},
};
use async_trait::async_trait;

/// Trait for storage provider factories
#[async_trait]
pub trait StorageProviderFactory: Send + Sync {
    /// Create a new storage provider instance
    async fn create(&self, config: FileServiceConfig) -> FileResult<Box<dyn StorageProvider>>;

    /// Get the provider type name
    fn provider_type(&self) -> &'static str;

    /// Get supported configuration options
    fn supported_options(&self) -> Vec<ConfigOption>;

    /// Validate configuration before creating provider
    fn validate_config(&self, config: &FileServiceConfig) -> FileResult<()>;
}

/// Base trait for all storage providers
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Initialize the storage provider
    async fn initialize(&mut self) -> FileResult<()>;

    /// Shutdown the storage provider and cleanup resources
    async fn shutdown(&mut self) -> FileResult<()>;

    /// Get the provider configuration
    fn config(&self) -> &FileServiceConfig;

    /// Check if the provider is initialized and ready
    fn is_ready(&self) -> bool;

    /// Get provider-specific capabilities
    fn capabilities(&self) -> ProviderCapabilities;

    /// Perform a health check
    async fn health_check(&self) -> FileResult<ProviderHealth>;

    /// Get provider statistics
    async fn get_statistics(&self) -> FileResult<ProviderStatistics>;

    /// Reload configuration (if supported)
    async fn reload_config(&mut self, config: FileServiceConfig) -> FileResult<()>;
}

/// Configuration option descriptor
#[derive(Debug, Clone)]
pub struct ConfigOption {
    /// Option name
    pub name: String,
    /// Option description
    pub description: String,
    /// Option type
    pub option_type: ConfigOptionType,
    /// Whether the option is required
    pub required: bool,
    /// Default value (if any)
    pub default_value: Option<String>,
    /// Validation rules
    pub validation: Option<ConfigValidation>,
}

/// Configuration option types
#[derive(Debug, Clone)]
pub enum ConfigOptionType {
    String,
    Integer,
    Float,
    Boolean,
    Path,
    Url,
    Email,
    Duration,
    Size,
    Enum(Vec<String>),
}

/// Configuration validation rules
#[derive(Debug, Clone)]
pub struct ConfigValidation {
    /// Minimum value (for numbers)
    pub min: Option<f64>,
    /// Maximum value (for numbers)
    pub max: Option<f64>,
    /// Regular expression pattern (for strings)
    pub pattern: Option<String>,
    /// Minimum length (for strings)
    pub min_length: Option<usize>,
    /// Maximum length (for strings)
    pub max_length: Option<usize>,
}

/// Provider capabilities
#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    /// Whether the provider supports streaming uploads
    pub streaming_upload: bool,
    /// Whether the provider supports streaming downloads
    pub streaming_download: bool,
    /// Whether the provider supports multipart uploads
    pub multipart_upload: bool,
    /// Whether the provider supports resumable uploads
    pub resumable_upload: bool,
    /// Whether the provider supports directory operations
    pub directory_operations: bool,
    /// Whether the provider supports metadata
    pub metadata_support: bool,
    /// Whether the provider supports permissions
    pub permissions_support: bool,
    /// Whether the provider supports server-side encryption
    pub encryption_support: bool,
    /// Whether the provider supports compression
    pub compression_support: bool,
    /// Whether the provider supports versioning
    pub versioning_support: bool,
    /// Whether the provider supports pre-signed URLs
    pub presigned_urls: bool,
    /// Whether the provider supports batch operations
    pub batch_operations: bool,
    /// Whether the provider supports file search
    pub search_support: bool,
    /// Whether the provider supports file locking
    pub locking_support: bool,
    /// Whether the provider supports atomic operations
    pub atomic_operations: bool,
    /// Whether the provider supports transactions
    pub transaction_support: bool,
    /// Maximum file size supported
    pub max_file_size: Option<u64>,
    /// Maximum path length supported
    pub max_path_length: Option<usize>,
    /// Maximum metadata size supported
    pub max_metadata_size: Option<usize>,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            streaming_upload: false,
            streaming_download: false,
            multipart_upload: false,
            resumable_upload: false,
            directory_operations: true,
            metadata_support: false,
            permissions_support: false,
            encryption_support: false,
            compression_support: false,
            versioning_support: false,
            presigned_urls: false,
            batch_operations: false,
            search_support: false,
            locking_support: false,
            atomic_operations: false,
            transaction_support: false,
            max_file_size: None,
            max_path_length: None,
            max_metadata_size: None,
        }
    }
}

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Whether the provider is healthy
    pub healthy: bool,
    /// Health check timestamp
    pub checked_at: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error message (if unhealthy)
    pub error_message: Option<String>,
    /// Additional health details
    pub details: std::collections::HashMap<String, String>,
}

/// Provider statistics
#[derive(Debug, Clone)]
pub struct ProviderStatistics {
    /// Total number of operations performed
    pub total_operations: u64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Total bytes uploaded
    pub bytes_uploaded: u64,
    /// Total bytes downloaded
    pub bytes_downloaded: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Current active connections
    pub active_connections: u32,
    /// Statistics collection start time
    pub stats_since: chrono::DateTime<chrono::Utc>,
    /// Last operation timestamp
    pub last_operation_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ProviderStatistics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
            avg_response_time_ms: 0.0,
            active_connections: 0,
            stats_since: chrono::Utc::now(),
            last_operation_at: None,
        }
    }
}

/// Trait for providers that support connection pooling
#[async_trait]
pub trait PooledStorageProvider: StorageProvider {
    /// Get the current pool size
    fn pool_size(&self) -> u32;

    /// Get the number of active connections
    fn active_connections(&self) -> u32;

    /// Get the number of idle connections
    fn idle_connections(&self) -> u32;

    /// Resize the connection pool
    async fn resize_pool(&mut self, new_size: u32) -> FileResult<()>;

    /// Test all connections in the pool
    async fn test_connections(&self) -> FileResult<Vec<ConnectionHealth>>;
}

/// Connection health information
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    /// Connection ID
    pub connection_id: String,
    /// Whether the connection is healthy
    pub healthy: bool,
    /// Connection age in seconds
    pub age_seconds: u64,
    /// Last used timestamp
    pub last_used_at: chrono::DateTime<chrono::Utc>,
    /// Error message (if unhealthy)
    pub error_message: Option<String>,
}

/// Trait for providers that support caching
#[async_trait]
pub trait CachedStorageProvider: StorageProvider {
    /// Clear all cached data
    async fn clear_cache(&self) -> FileResult<()>;

    /// Clear cached data for a specific path
    async fn clear_cache_for_path(&self, path: &str) -> FileResult<()>;

    /// Get cache statistics
    async fn cache_statistics(&self) -> FileResult<CacheStatistics>;

    /// Set cache TTL for a specific path
    async fn set_cache_ttl(&self, path: &str, ttl_seconds: u64) -> FileResult<()>;
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
    /// Current cache size in bytes
    pub size_bytes: u64,
    /// Number of cached entries
    pub entry_count: u64,
    /// Cache eviction count
    pub evictions: u64,
}

/// Trait for providers that support monitoring
pub trait MonitorableStorageProvider: StorageProvider {
    /// Register a metrics collector
    fn register_metrics_collector(&mut self, collector: Box<dyn MetricsCollector>);

    /// Get current metrics
    fn get_metrics(&self) -> std::collections::HashMap<String, MetricValue>;

    /// Enable/disable detailed tracing
    fn set_tracing_enabled(&mut self, enabled: bool);
}

/// Metrics collector trait
pub trait MetricsCollector: Send + Sync {
    /// Record a metric
    fn record_metric(&self, name: &str, value: MetricValue, tags: &[(&str, &str)]);

    /// Record operation timing
    fn record_timing(&self, operation: &str, duration_ms: u64, tags: &[(&str, &str)]);

    /// Record operation count
    fn record_count(&self, operation: &str, count: u64, tags: &[(&str, &str)]);
}

/// Metric value types
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(u64), // milliseconds
}

/// Provider registry for managing multiple storage providers
pub struct ProviderRegistry {
    factories: std::collections::HashMap<String, Box<dyn StorageProviderFactory>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            factories: std::collections::HashMap::new(),
        }
    }

    /// Register a storage provider factory
    pub fn register<F>(&mut self, factory: F)
    where
        F: StorageProviderFactory + 'static,
    {
        let provider_type = factory.provider_type().to_string();
        self.factories.insert(provider_type, Box::new(factory));
    }

    /// Create a storage provider from configuration
    pub async fn create_provider(
        &self,
        config: FileServiceConfig,
    ) -> FileResult<Box<dyn StorageProvider>> {
        let provider_type = match config.provider {
            crate::models::config::StorageProvider::Local => "local",
            crate::models::config::StorageProvider::S3 => "s3",
            crate::models::config::StorageProvider::AzureBlob => "azure_blob",
            crate::models::config::StorageProvider::GoogleCloud => "google_cloud",
            crate::models::config::StorageProvider::Ftp => "ftp",
            crate::models::config::StorageProvider::Sftp => "sftp",
            crate::models::config::StorageProvider::WebDav => "webdav",
        };

        let factory = self
            .factories
            .get(provider_type)
            .ok_or_else(|| FileServiceError::ConfigError {
                message: format!("Unsupported storage provider: {}", provider_type),
            })?;

        factory.validate_config(&config)?;
        factory.create(config).await
    }

    /// List available provider types
    pub fn available_providers(&self) -> Vec<&str> {
        self.factories.keys().map(|k| k.as_str()).collect()
    }

    /// Get configuration options for a provider type
    pub fn get_provider_options(&self, provider_type: &str) -> Option<Vec<ConfigOption>> {
        self.factories
            .get(provider_type)
            .map(|f| f.supported_options())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_capabilities() {
        let mut caps = ProviderCapabilities::default();
        assert!(!caps.streaming_upload);
        assert!(caps.directory_operations);

        caps.streaming_upload = true;
        caps.multipart_upload = true;
        assert!(caps.streaming_upload);
        assert!(caps.multipart_upload);
    }

    #[test]
    fn test_provider_health() {
        let health = ProviderHealth {
            healthy: true,
            checked_at: chrono::Utc::now(),
            response_time_ms: 150,
            error_message: None,
            details: std::collections::HashMap::new(),
        };

        assert!(health.healthy);
        assert_eq!(health.response_time_ms, 150);
        assert!(health.error_message.is_none());
    }

    #[test]
    fn test_provider_statistics() {
        let mut stats = ProviderStatistics::default();
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.successful_operations, 0);

        stats.total_operations = 100;
        stats.successful_operations = 95;
        stats.failed_operations = 5;

        assert_eq!(stats.total_operations, 100);
        assert_eq!(stats.successful_operations, 95);
        assert_eq!(stats.failed_operations, 5);
    }

    #[test]
    fn test_config_option() {
        let option = ConfigOption {
            name: "bucket_name".to_string(),
            description: "S3 bucket name".to_string(),
            option_type: ConfigOptionType::String,
            required: true,
            default_value: None,
            validation: Some(ConfigValidation {
                min: None,
                max: None,
                pattern: Some(r"^[a-z0-9][a-z0-9\-]*[a-z0-9]$".to_string()),
                min_length: Some(3),
                max_length: Some(63),
            }),
        };

        assert_eq!(option.name, "bucket_name");
        assert!(option.required);
        assert!(option.validation.is_some());
    }

    #[test]
    fn test_provider_registry() {
        let registry = ProviderRegistry::new();
        assert_eq!(registry.available_providers().len(), 0);

        // Would normally register factories here in real tests
        // registry.register(LocalStorageProviderFactory::new());
    }

    #[test]
    fn test_cache_statistics() {
        let stats = CacheStatistics {
            hits: 750,
            misses: 250,
            hit_ratio: 0.75,
            size_bytes: 1024 * 1024,
            entry_count: 100,
            evictions: 10,
        };

        assert_eq!(stats.hits, 750);
        assert_eq!(stats.misses, 250);
        assert_eq!(stats.hit_ratio, 0.75);
    }

    #[test]
    fn test_metric_value() {
        let counter = MetricValue::Counter(42);
        let gauge = MetricValue::Gauge(3.14);
        let timer = MetricValue::Timer(150);

        match counter {
            MetricValue::Counter(value) => assert_eq!(value, 42),
            _ => panic!("Expected counter"),
        }

        match gauge {
            MetricValue::Gauge(value) => assert_eq!(value, 3.14),
            _ => panic!("Expected gauge"),
        }

        match timer {
            MetricValue::Timer(value) => assert_eq!(value, 150),
            _ => panic!("Expected timer"),
        }
    }
}