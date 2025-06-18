//! Integration tests for df-core functionality.
//!
//! These tests verify that all components work together correctly,
//! including service registry, configuration, plugins, and error handling.

use df_core::prelude::*;
use df_core::{
    config::{Config, ConfigBuilder},
    service::{BaseService, ServiceInfo, ServiceState},
    registry::ServiceRegistry,
    plugin::{PluginManager, PluginInfo, PluginState, PluginEvent},
    error::{DfError, DfResult},
};
use serde_json;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

// Test service implementations for integration testing
struct DatabaseService {
    base: BaseService,
    connection_count: u32,
}

impl DatabaseService {
    fn new() -> Self {
        let info = ServiceInfo::new("database", "1.0.0", "Database service")
            .with_tag("database")
            .with_tag("infrastructure")
            .with_metadata("max_connections", "100");
        
        Self {
            base: BaseService::new(info),
            connection_count: 0,
        }
    }
}

#[async_trait::async_trait]
impl Service for DatabaseService {
    fn info(&self) -> &ServiceInfo {
        self.base.info()
    }

    fn state(&self) -> ServiceState {
        self.base.state()
    }

    async fn initialize(&mut self) -> DfResult<()> {
        self.base.set_state(ServiceState::Initializing);
        // Simulate database initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        self.base.set_state(ServiceState::Initialized);
        Ok(())
    }

    async fn start(&mut self) -> DfResult<()> {
        if self.state() != ServiceState::Initialized {
            return Err(DfError::service("database", "Must be initialized before starting"));
        }
        
        self.base.set_state(ServiceState::Starting);
        // Simulate database startup
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        self.connection_count = 10;
        self.base.set_state(ServiceState::Running);
        Ok(())
    }

    async fn stop(&mut self) -> DfResult<()> {
        if self.state() != ServiceState::Running {
            return Err(DfError::service("database", "Must be running to stop"));
        }
        
        self.base.set_state(ServiceState::Stopping);
        self.connection_count = 0;
        self.base.set_state(ServiceState::Stopped);
        Ok(())
    }

    async fn health_check(&self) -> DfResult<bool> {
        Ok(self.state() == ServiceState::Running && self.connection_count > 0)
    }

    async fn metrics(&self) -> DfResult<HashMap<String, String>> {
        let mut metrics = HashMap::new();
        metrics.insert("connections".to_string(), self.connection_count.to_string());
        metrics.insert("state".to_string(), format!("{:?}", self.state()));
        Ok(metrics)
    }
}

struct ApiService {
    base: BaseService,
    request_count: u64,
}

impl ApiService {
    fn new() -> Self {
        let info = ServiceInfo::new("api", "1.0.0", "API service")
            .with_dependency("database")
            .with_tag("api")
            .with_tag("web")
            .with_metadata("port", "8080");
        
        Self {
            base: BaseService::new(info),
            request_count: 0,
        }
    }
}

#[async_trait::async_trait]
impl Service for ApiService {
    fn info(&self) -> &ServiceInfo {
        self.base.info()
    }

    fn state(&self) -> ServiceState {
        self.base.state()
    }

    async fn initialize(&mut self) -> DfResult<()> {
        self.base.set_state(ServiceState::Initializing);
        // Simulate API initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        self.base.set_state(ServiceState::Initialized);
        Ok(())
    }

    async fn start(&mut self) -> DfResult<()> {
        if self.state() != ServiceState::Initialized {
            return Err(DfError::service("api", "Must be initialized before starting"));
        }
        
        self.base.set_state(ServiceState::Starting);
        // Simulate API startup
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        self.base.set_state(ServiceState::Running);
        Ok(())
    }

    async fn stop(&mut self) -> DfResult<()> {
        if self.state() != ServiceState::Running {
            return Err(DfError::service("api", "Must be running to stop"));
        }
        
        self.base.set_state(ServiceState::Stopping);
        self.base.set_state(ServiceState::Stopped);
        Ok(())
    }

    async fn health_check(&self) -> DfResult<bool> {
        Ok(self.state() == ServiceState::Running)
    }

    async fn metrics(&self) -> DfResult<HashMap<String, String>> {
        let mut metrics = HashMap::new();
        metrics.insert("requests".to_string(), self.request_count.to_string());
        metrics.insert("state".to_string(), format!("{:?}", self.state()));
        Ok(metrics)
    }

    async fn configure(&mut self, config: HashMap<String, String>) -> DfResult<()> {
        if let Some(port) = config.get("port") {
            // Simulate port configuration
            if port == "invalid" {
                return Err(DfError::config("Invalid port configuration"));
            }
        }
        Ok(())
    }
}

// Mock plugin for integration testing
struct TestPlugin {
    info: PluginInfo,
    state: PluginState,
    events_received: Vec<PluginEvent>,
}

impl TestPlugin {
    fn new() -> Self {
        let info = PluginInfo::new("test-plugin", "1.0.0", "Test plugin for integration", "Test Author")
            .with_capability("test")
            .with_capability("integration")
            .with_metadata("test", "integration");

        Self {
            info,
            state: PluginState::Loaded,
            events_received: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl df_core::plugin::Plugin for TestPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn state(&self) -> PluginState {
        self.state
    }

    async fn initialize(&mut self) -> DfResult<()> {
        self.state = PluginState::Initializing;
        // Simulate plugin initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        self.state = PluginState::Initialized;
        Ok(())
    }

    async fn start(&mut self) -> DfResult<()> {
        if self.state != PluginState::Initialized {
            return Err(DfError::plugin("test-plugin", "Must be initialized before starting"));
        }
        
        self.state = PluginState::Starting;
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        self.state = PluginState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> DfResult<()> {
        if self.state != PluginState::Running {
            return Err(DfError::plugin("test-plugin", "Must be running to stop"));
        }
        
        self.state = PluginState::Stopping;
        self.state = PluginState::Stopped;
        Ok(())
    }

    async fn handle_event(&mut self, event: PluginEvent) -> DfResult<()> {
        self.events_received.push(event);
        Ok(())
    }
}

#[tokio::test]
async fn test_complete_service_lifecycle_integration() {
    // Create service registry
    let registry = ServiceRegistry::new();
    
    // Register services
    registry.register_service(DatabaseService::new()).await.unwrap();
    registry.register_service(ApiService::new()).await.unwrap();
    
    // Verify services are registered
    assert!(registry.has_service("database").await);
    assert!(registry.has_service("api").await);
    
    // Check startup order (database should come before api due to dependency)
    let startup_order = registry.get_startup_order().await;
    assert_eq!(startup_order, vec!["database", "api"]);
    
    // Initialize all services
    registry.initialize_all().await.unwrap();
    
    // Verify all services are initialized
    assert_eq!(registry.get_service_state("database").await.unwrap(), ServiceState::Initialized);
    assert_eq!(registry.get_service_state("api").await.unwrap(), ServiceState::Initialized);
    
    // Start all services
    registry.start_all().await.unwrap();
    
    // Verify all services are running
    assert_eq!(registry.get_service_state("database").await.unwrap(), ServiceState::Running);
    assert_eq!(registry.get_service_state("api").await.unwrap(), ServiceState::Running);
    
    // Health check all services
    let health_results = registry.health_check_all().await.unwrap();
    assert_eq!(health_results.len(), 2);
    assert_eq!(health_results.get("database"), Some(&true));
    assert_eq!(health_results.get("api"), Some(&true));
    
    // Stop all services
    registry.stop_all().await.unwrap();
    
    // Verify all services are stopped
    assert_eq!(registry.get_service_state("database").await.unwrap(), ServiceState::Stopped);
    assert_eq!(registry.get_service_state("api").await.unwrap(), ServiceState::Stopped);
}

#[tokio::test]
async fn test_configuration_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");
    
    // Create test configuration file
    let config_content = r#"
app:
  name: "integration-test"
  version: "1.0.0"
  environment: "test"
  debug: true
  timezone: "UTC"

server:
  host: "127.0.0.1"
  port: 8080
  workers: 4

database:
  url: "sqlite://test.db"
  max_connections: 10

cache:
  enabled: true
  driver: "memory"
  ttl: 3600

logging:
  level: "debug"
  format: "json"
  output: "stdout"

security:
  jwt_secret: "test-secret"
  jwt_expiry: 3600
  cors_origins:
    - "http://localhost:3000"

plugins:
  enabled: true
  directory: "./test_plugins"
  auto_load: true
  whitelist: []
  blacklist: []

services:
  database:
    pool_size: 20
    timeout: 30
  api:
    rate_limit: 100
    timeout: 60
"#;

    std::fs::write(&config_path, config_content).unwrap();
    
    // Load configuration
    let config = Config::from_file(&config_path).unwrap();
    
    // Verify configuration is loaded correctly
    assert_eq!(config.app.name, "integration-test");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.workers, Some(4));
    assert_eq!(config.database.max_connections, Some(10));
    assert!(config.cache.enabled);
    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.security.jwt_secret, "test-secret");
    assert!(config.plugins.enabled);
    
    // Test service-specific configuration
    #[derive(serde::Deserialize)]
    struct DatabaseConfig {
        pool_size: u32,
        timeout: u32,
    }
    
    let db_config: DatabaseConfig = config.get_service_config("database").unwrap();
    assert_eq!(db_config.pool_size, 20);
    assert_eq!(db_config.timeout, 30);
    
    // Validate configuration
    config.validate().unwrap();
    
    // Test environment helpers
    assert!(!config.is_production());
    assert!(config.is_debug());
}

#[tokio::test]
async fn test_configuration_with_environment_override() {
    // Set environment variables
    std::env::set_var("DF_APP_NAME", "env-override-app");
    std::env::set_var("DF_SERVER_PORT", "9999");
    std::env::set_var("DF_DATABASE_MAX_CONNECTIONS", "50");
    
    let config = Config::from_env().unwrap();
    
    // Verify environment overrides work
    assert_eq!(config.app.name, "env-override-app");
    assert_eq!(config.server.port, 9999);
    assert_eq!(config.database.max_connections, Some(50));
    
    // Clean up environment variables
    std::env::remove_var("DF_APP_NAME");
    std::env::remove_var("DF_SERVER_PORT");
    std::env::remove_var("DF_DATABASE_MAX_CONNECTIONS");
}

#[tokio::test]
async fn test_error_handling_integration() {
    let registry = ServiceRegistry::new();
    
    // Test service registration errors
    let service1 = DatabaseService::new();
    let service2 = DatabaseService::new(); // Same name
    
    registry.register_service(service1).await.unwrap();
    let result = registry.register_service(service2).await;
    assert!(result.is_err());
    
    // Verify error is properly categorized
    match result.unwrap_err() {
        DfError::Registry { message } => {
            assert!(message.contains("already registered"));
        }
        _ => panic!("Expected Registry error"),
    }
    
    // Test service lifecycle errors
    let result = registry.start_service("database").await;
    assert!(result.is_err()); // Should fail because not initialized
    
    registry.initialize_service("database").await.unwrap();
    registry.start_service("database").await.unwrap();
    
    let result = registry.start_service("database").await;
    assert!(result.is_err()); // Should fail because already running
}

#[tokio::test]
async fn test_plugin_system_integration() {
    let mut plugin_manager = PluginManager::new();
    
    // Configure plugin manager
    plugin_manager.set_auto_load(false);
    plugin_manager.set_api_version("1.0.0");
    
    // Since we can't easily test dynamic loading in unit tests,
    // we'll test the plugin management logic
    
    // Test plugin states and events
    let mut plugin = TestPlugin::new();
    
    // Initialize plugin
    plugin.initialize().await.unwrap();
    assert_eq!(plugin.state(), PluginState::Initialized);
    
    // Start plugin
    plugin.start().await.unwrap();
    assert_eq!(plugin.state(), PluginState::Running);
    
    // Send events to plugin
    let startup_event = PluginEvent::SystemStartup;
    plugin.handle_event(startup_event).await.unwrap();
    
    let config_event = PluginEvent::ConfigurationChanged {
        keys: vec!["server.port".to_string()],
    };
    plugin.handle_event(config_event).await.unwrap();
    
    assert_eq!(plugin.events_received.len(), 2);
    
    // Stop plugin
    plugin.stop().await.unwrap();
    assert_eq!(plugin.state(), PluginState::Stopped);
}

#[tokio::test]
async fn test_full_system_integration() {
    // This test verifies that all components work together
    
    // 1. Configuration
    let config = Config::builder()
        .with_validation(|config| {
            if config.server.port == 0 {
                Err(DfError::config("Port cannot be 0"))
            } else {
                Ok(())
            }
        })
        .build()
        .unwrap();
    
    assert!(config.validate().is_ok());
    
    // 2. Service Registry
    let registry = ServiceRegistry::new();
    
    // Register services
    registry.register_service(DatabaseService::new()).await.unwrap();
    registry.register_service(ApiService::new()).await.unwrap();
    
    // 3. Plugin Manager
    let mut plugin_manager = PluginManager::new();
    plugin_manager.set_api_version("1.0.0");
    
    // 4. Full system startup simulation
    
    // Initialize services
    registry.initialize_all().await.unwrap();
    
    // Start services
    registry.start_all().await.unwrap();
    
    // Verify system is healthy
    let health_results = registry.health_check_all().await.unwrap();
    assert!(health_results.values().all(|&healthy| healthy));
    
    // Simulate configuration change
    let mut config_update = HashMap::new();
    config_update.insert("port".to_string(), "8080".to_string());
    
    // This would normally trigger service reconfiguration
    // For now, we just verify the configuration is valid
    
    // 5. System shutdown simulation
    
    // Stop all services
    registry.stop_all().await.unwrap();
    
    // Verify all services are stopped
    let final_states: Vec<ServiceState> = vec![
        registry.get_service_state("database").await.unwrap(),
        registry.get_service_state("api").await.unwrap(),
    ];
    
    assert!(final_states.iter().all(|&state| state == ServiceState::Stopped));
}

#[tokio::test]
async fn test_dependency_resolution_complex() {
    let registry = ServiceRegistry::new();
    
    // Create a more complex dependency graph
    let auth_info = ServiceInfo::new("auth", "1.0.0", "Authentication service")
        .with_dependency("database");
    let auth_service = BaseService::new(auth_info);
    
    let cache_info = ServiceInfo::new("cache", "1.0.0", "Cache service");
    let cache_service = BaseService::new(cache_info);
    
    let api_info = ServiceInfo::new("api-v2", "2.0.0", "API service v2")
        .with_dependencies(["auth", "cache", "database"]);
    let api_v2_service = BaseService::new(api_info);
    
    let web_info = ServiceInfo::new("web", "1.0.0", "Web server")
        .with_dependency("api-v2");
    let web_service = BaseService::new(web_info);
    
    // Register services in random order
    registry.register_service(web_service).await.unwrap();
    registry.register_service(api_v2_service).await.unwrap();
    registry.register_service(DatabaseService::new()).await.unwrap();
    registry.register_service(auth_service).await.unwrap();
    registry.register_service(cache_service).await.unwrap();
    
    // Verify dependency order is correct
    let startup_order = registry.get_startup_order().await;
    
    // Database and cache should come first (no dependencies)
    assert!(startup_order.iter().position(|s| s == "database").unwrap() < 
            startup_order.iter().position(|s| s == "auth").unwrap());
    
    // Auth should come before api-v2
    assert!(startup_order.iter().position(|s| s == "auth").unwrap() < 
            startup_order.iter().position(|s| s == "api-v2").unwrap());
    
    // API-v2 should come before web
    assert!(startup_order.iter().position(|s| s == "api-v2").unwrap() < 
            startup_order.iter().position(|s| s == "web").unwrap());
    
    // Initialize all services in correct order
    registry.initialize_all().await.unwrap();
    registry.start_all().await.unwrap();
    
    // Verify all services are running
    let health_results = registry.health_check_all().await.unwrap();
    assert_eq!(health_results.len(), 5);
    assert!(health_results.values().all(|&healthy| healthy));
}

#[tokio::test]
async fn test_error_propagation_and_recovery() {
    let registry = ServiceRegistry::new();
    
    // Create a service that will fail during startup
    struct FailingService {
        base: BaseService,
        should_fail: bool,
    }
    
    impl FailingService {
        fn new() -> Self {
            let info = ServiceInfo::new("failing", "1.0.0", "Service that fails");
            Self {
                base: BaseService::new(info),
                should_fail: true,
            }
        }
    }
    
    #[async_trait::async_trait]
    impl Service for FailingService {
        fn info(&self) -> &ServiceInfo {
            self.base.info()
        }

        fn state(&self) -> ServiceState {
            self.base.state()
        }

        async fn initialize(&mut self) -> DfResult<()> {
            self.base.initialize().await
        }

        async fn start(&mut self) -> DfResult<()> {
            if self.should_fail {
                self.base.set_error_state();
                return Err(DfError::service("failing", "Simulated failure"));
            }
            self.base.start().await
        }

        async fn stop(&mut self) -> DfResult<()> {
            self.base.stop().await
        }
    }
    
    registry.register_service(FailingService::new()).await.unwrap();
    registry.initialize_service("failing").await.unwrap();
    
    // Service should fail to start
    let result = registry.start_service("failing").await;
    assert!(result.is_err());
    
    // Verify error is properly categorized
    match result.unwrap_err() {
        DfError::Service { service, message } => {
            assert_eq!(service, "failing");
            assert!(message.contains("Simulated failure"));
        }
        _ => panic!("Expected Service error"),
    }
    
    // Service state should be error
    assert_eq!(registry.get_service_state("failing").await.unwrap(), ServiceState::Error);
}

#[tokio::test] 
async fn test_metrics_and_monitoring_integration() {
    let registry = ServiceRegistry::new();
    
    registry.register_service(DatabaseService::new()).await.unwrap();
    registry.register_service(ApiService::new()).await.unwrap();
    
    registry.initialize_all().await.unwrap();
    registry.start_all().await.unwrap();
    
    // Get service information
    let db_info = registry.get_service_info("database").await.unwrap();
    assert_eq!(db_info.name, "database");
    assert!(db_info.tags.contains(&"database".to_string()));
    assert!(db_info.tags.contains(&"infrastructure".to_string()));
    
    let api_info = registry.get_service_info("api").await.unwrap();
    assert_eq!(api_info.dependencies, vec!["database"]);
    assert!(api_info.tags.contains(&"api".to_string()));
    
    // Health monitoring
    let health_results = registry.health_check_all().await.unwrap();
    assert_eq!(health_results.len(), 2);
    
    // All services should be healthy
    for (service_name, is_healthy) in health_results {
        assert!(is_healthy, "Service {} should be healthy", service_name);
    }
    
    registry.stop_all().await.unwrap();
}

#[tokio::test]
async fn test_service_factory_integration() {
    let registry = ServiceRegistry::new();
    
    // Register service factory
    registry.register_closure_factory("database".to_string(), || DatabaseService::new()).await.unwrap();
    
    // Service should be available through factory
    assert!(registry.has_service("database").await);
    
    // Initialize service from factory (this would create the service)
    // Note: The current implementation has limitations with service retrieval
    // In a real implementation, you'd want to handle this better
}

// Benchmark test to ensure performance is acceptable
#[tokio::test]
async fn test_performance_integration() {
    let start = std::time::Instant::now();
    
    let registry = ServiceRegistry::new();
    
    // Register many services to test performance
    for i in 0..100 {
        let info = ServiceInfo::new(
            format!("service-{}", i),
            "1.0.0",
            format!("Test service {}", i)
        );
        let service = BaseService::new(info);
        registry.register_service(service).await.unwrap();
    }
    
    // Initialize and start all services
    registry.initialize_all().await.unwrap();
    registry.start_all().await.unwrap();
    
    let elapsed = start.elapsed();
    
    // Should complete within reasonable time (this is quite generous)
    assert!(elapsed < std::time::Duration::from_secs(5), 
           "Performance test took too long: {:?}", elapsed);
    
    // Clean up
    registry.stop_all().await.unwrap();
}