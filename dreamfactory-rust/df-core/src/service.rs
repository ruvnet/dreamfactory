//! Service abstractions and lifecycle management for the DreamFactory framework.
//!
//! This module provides the core Service trait and related types for building
//! modular, composable services with proper lifecycle management.

use crate::error::DfResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Service state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    /// Service is created but not yet initialized
    Created,
    /// Service is initializing
    Initializing,
    /// Service is initialized and ready to start
    Initialized,
    /// Service is starting up
    Starting,
    /// Service is running and ready to handle requests
    Running,
    /// Service is stopping
    Stopping,
    /// Service is stopped
    Stopped,
    /// Service encountered an error
    Error,
}

/// Service information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Unique service identifier
    pub id: Uuid,
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service description
    pub description: String,
    /// Service dependencies (service names)
    pub dependencies: Vec<String>,
    /// Service tags for categorization
    pub tags: Vec<String>,
    /// Service metadata
    pub metadata: HashMap<String, String>,
}

impl ServiceInfo {
    /// Create a new ServiceInfo
    pub fn new<S: Into<String>>(name: S, version: S, description: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a dependency
    pub fn with_dependency<S: Into<String>>(mut self, dependency: S) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies
    pub fn with_dependencies<I, S>(mut self, dependencies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.dependencies.extend(dependencies.into_iter().map(|s| s.into()));
        self
    }

    /// Add a tag
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(|s| s.into()));
        self
    }

    /// Add metadata
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for ServiceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{} ({})", self.name, self.version, self.id)
    }
}

/// Core service trait that all DreamFactory services must implement
#[async_trait]
pub trait Service: Send + Sync {
    /// Get service information
    fn info(&self) -> &ServiceInfo;

    /// Get current service state
    fn state(&self) -> ServiceState;

    /// Initialize the service
    async fn initialize(&mut self) -> DfResult<()>;

    /// Start the service
    async fn start(&mut self) -> DfResult<()>;

    /// Stop the service
    async fn stop(&mut self) -> DfResult<()>;

    /// Check if the service is healthy
    async fn health_check(&self) -> DfResult<bool> {
        Ok(self.state() == ServiceState::Running)
    }

    /// Get service metrics (optional)
    async fn metrics(&self) -> DfResult<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    /// Handle configuration updates (optional)
    async fn configure(&mut self, _config: HashMap<String, String>) -> DfResult<()> {
        Ok(())
    }
}

/// Base service implementation with common functionality
#[derive(Debug)]
pub struct BaseService {
    info: ServiceInfo,
    state: ServiceState,
}

impl BaseService {
    /// Create a new base service
    pub fn new(info: ServiceInfo) -> Self {
        Self {
            info,
            state: ServiceState::Created,
        }
    }

    /// Set the service state
    pub fn set_state(&mut self, state: ServiceState) {
        self.state = state;
    }

    /// Transition to error state with context
    pub fn set_error_state(&mut self) {
        self.state = ServiceState::Error;
    }
}

#[async_trait]
impl Service for BaseService {
    fn info(&self) -> &ServiceInfo {
        &self.info
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    async fn initialize(&mut self) -> DfResult<()> {
        self.state = ServiceState::Initializing;
        // Basic initialization logic
        self.state = ServiceState::Initialized;
        Ok(())
    }

    async fn start(&mut self) -> DfResult<()> {
        if self.state != ServiceState::Initialized {
            return Err(crate::error::DfError::service(
                &self.info.name,
                "Service must be initialized before starting"
            ));
        }
        
        self.state = ServiceState::Starting;
        // Basic start logic
        self.state = ServiceState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> DfResult<()> {
        if self.state != ServiceState::Running {
            return Err(crate::error::DfError::service(
                &self.info.name,
                "Service must be running to stop"
            ));
        }
        
        self.state = ServiceState::Stopping;
        // Basic stop logic
        self.state = ServiceState::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_service_info_creation() {
        let info = ServiceInfo::new("test-service", "1.0.0", "A test service");
        
        assert_eq!(info.name, "test-service");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, "A test service");
        assert!(info.dependencies.is_empty());
        assert!(info.tags.is_empty());
        assert!(info.metadata.is_empty());
    }

    #[test]
    fn test_service_info_builder_pattern() {
        let info = ServiceInfo::new("test-service", "1.0.0", "A test service")
            .with_dependency("database-service")
            .with_dependencies(["auth-service", "cache-service"])
            .with_tag("core")
            .with_tags(["database", "api"])
            .with_metadata("author", "DreamFactory")
            .with_metadata("category", "infrastructure");

        assert_eq!(info.dependencies, vec!["database-service", "auth-service", "cache-service"]);
        assert_eq!(info.tags, vec!["core", "database", "api"]);
        assert_eq!(info.metadata.get("author"), Some(&"DreamFactory".to_string()));
        assert_eq!(info.metadata.get("category"), Some(&"infrastructure".to_string()));
    }

    #[test]
    fn test_service_info_display() {
        let info = ServiceInfo::new("test-service", "1.0.0", "A test service");
        let display = format!("{}", info);
        assert!(display.contains("test-service"));
        assert!(display.contains("1.0.0"));
        assert!(display.contains(&info.id.to_string()));
    }

    #[test]
    fn test_service_state_transitions() {
        let states = [
            ServiceState::Created,
            ServiceState::Initializing,
            ServiceState::Initialized,
            ServiceState::Starting,
            ServiceState::Running,
            ServiceState::Stopping,
            ServiceState::Stopped,
            ServiceState::Error,
        ];

        for state in &states {
            let json = serde_json::to_string(state).unwrap();
            let deserialized: ServiceState = serde_json::from_str(&json).unwrap();
            assert_eq!(*state, deserialized);
        }
    }

    #[tokio::test]
    async fn test_base_service_lifecycle() {
        let info = ServiceInfo::new("test-service", "1.0.0", "A test service");
        let mut service = BaseService::new(info);

        // Initial state
        assert_eq!(service.state(), ServiceState::Created);

        // Initialize
        service.initialize().await.unwrap();
        assert_eq!(service.state(), ServiceState::Initialized);

        // Start
        service.start().await.unwrap();
        assert_eq!(service.state(), ServiceState::Running);

        // Health check
        assert!(service.health_check().await.unwrap());

        // Stop
        service.stop().await.unwrap();
        assert_eq!(service.state(), ServiceState::Stopped);
    }

    #[tokio::test]
    async fn test_base_service_invalid_transitions() {
        let info = ServiceInfo::new("test-service", "1.0.0", "A test service");
        let mut service = BaseService::new(info);

        // Try to start without initializing
        let result = service.start().await;
        assert!(result.is_err());
        assert_eq!(service.state(), ServiceState::Created);

        // Initialize first
        service.initialize().await.unwrap();
        
        // Try to stop without starting
        let result = service.stop().await;
        assert!(result.is_err());
        assert_eq!(service.state(), ServiceState::Initialized);
    }

    #[tokio::test]
    async fn test_service_default_methods() {
        let info = ServiceInfo::new("test-service", "1.0.0", "A test service");
        let mut service = BaseService::new(info);
        
        service.initialize().await.unwrap();
        service.start().await.unwrap();

        // Test default metrics implementation
        let metrics = service.metrics().await.unwrap();
        assert!(metrics.is_empty());

        // Test default configure implementation
        let config = HashMap::new();
        service.configure(config).await.unwrap();
    }

    #[tokio::test]
    async fn test_service_error_handling() {
        let info = ServiceInfo::new("test-service", "1.0.0", "A test service");
        let mut service = BaseService::new(info);

        // Test error states
        service.set_error_state();
        assert_eq!(service.state(), ServiceState::Error);

        // Health check should return true by default even in error state
        // This tests the default implementation
        let health = service.health_check().await.unwrap();
        assert!(!health); // Should be false because state is not Running
    }

    // Mock service for testing trait implementations
    struct MockService {
        base: BaseService,
        should_fail_initialize: bool,
        should_fail_start: bool,
        should_fail_stop: bool,
    }

    impl MockService {
        fn new(info: ServiceInfo) -> Self {
            Self {
                base: BaseService::new(info),
                should_fail_initialize: false,
                should_fail_start: false,
                should_fail_stop: false,
            }
        }

        fn with_failing_initialize(mut self) -> Self {
            self.should_fail_initialize = true;
            self
        }

        fn with_failing_start(mut self) -> Self {
            self.should_fail_start = true;
            self
        }

        fn with_failing_stop(mut self) -> Self {
            self.should_fail_stop = true;
            self
        }
    }

    #[async_trait]
    impl Service for MockService {
        fn info(&self) -> &ServiceInfo {
            self.base.info()
        }

        fn state(&self) -> ServiceState {
            self.base.state()
        }

        async fn initialize(&mut self) -> DfResult<()> {
            if self.should_fail_initialize {
                self.base.set_error_state();
                return Err(crate::error::DfError::service(&self.info().name, "Initialize failed"));
            }
            self.base.initialize().await
        }

        async fn start(&mut self) -> DfResult<()> {
            if self.should_fail_start {
                self.base.set_error_state();
                return Err(crate::error::DfError::service(&self.info().name, "Start failed"));
            }
            self.base.start().await
        }

        async fn stop(&mut self) -> DfResult<()> {
            if self.should_fail_stop {
                self.base.set_error_state();
                return Err(crate::error::DfError::service(&self.info().name, "Stop failed"));
            }
            self.base.stop().await
        }

        async fn health_check(&self) -> DfResult<bool> {
            Ok(self.state() == ServiceState::Running)
        }

        async fn metrics(&self) -> DfResult<HashMap<String, String>> {
            let mut metrics = HashMap::new();
            metrics.insert("uptime".to_string(), "100".to_string());
            metrics.insert("requests".to_string(), "42".to_string());
            Ok(metrics)
        }

        async fn configure(&mut self, config: HashMap<String, String>) -> DfResult<()> {
            if config.contains_key("fail") {
                return Err(crate::error::DfError::service(&self.info().name, "Configuration failed"));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_service_success() {
        let info = ServiceInfo::new("mock-service", "1.0.0", "A mock service");
        let mut service = MockService::new(info);

        service.initialize().await.unwrap();
        service.start().await.unwrap();
        
        assert!(service.health_check().await.unwrap());
        
        let metrics = service.metrics().await.unwrap();
        assert_eq!(metrics.get("uptime"), Some(&"100".to_string()));
        assert_eq!(metrics.get("requests"), Some(&"42".to_string()));

        let config = HashMap::new();
        service.configure(config).await.unwrap();

        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_service_failures() {
        let info = ServiceInfo::new("mock-service", "1.0.0", "A mock service");
        
        // Test initialize failure
        let mut service = MockService::new(info.clone()).with_failing_initialize();
        assert!(service.initialize().await.is_err());
        assert_eq!(service.state(), ServiceState::Error);

        // Test start failure
        let mut service = MockService::new(info.clone()).with_failing_start();
        service.initialize().await.unwrap();
        assert!(service.start().await.is_err());
        assert_eq!(service.state(), ServiceState::Error);

        // Test stop failure
        let mut service = MockService::new(info.clone()).with_failing_stop();
        service.initialize().await.unwrap();
        service.start().await.unwrap();
        assert!(service.stop().await.is_err());
        assert_eq!(service.state(), ServiceState::Error);

        // Test configure failure
        let mut service = MockService::new(info);
        let mut config = HashMap::new();
        config.insert("fail".to_string(), "true".to_string());
        assert!(service.configure(config).await.is_err());
    }

    #[test]
    fn test_service_state_equality() {
        assert_eq!(ServiceState::Running, ServiceState::Running);
        assert_ne!(ServiceState::Running, ServiceState::Stopped);
    }

    #[test]
    fn test_service_info_clone() {
        let info = ServiceInfo::new("test", "1.0", "test")
            .with_dependency("dep")
            .with_tag("tag")
            .with_metadata("key", "value");
        
        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.dependencies, cloned.dependencies);
        assert_eq!(info.tags, cloned.tags);
        assert_eq!(info.metadata, cloned.metadata);
    }
}