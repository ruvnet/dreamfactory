//! Basic usage example for df-core framework.
//!
//! This example demonstrates how to:
//! - Create and configure services
//! - Set up a service registry
//! - Load configuration from files and environment
//! - Manage service lifecycle
//! - Handle errors gracefully

use df_core::prelude::*;
use std::collections::HashMap;
use tokio;

// Example service implementation
struct DatabaseService {
    base: df_core::service::BaseService,
    connection_pool_size: u32,
}

impl DatabaseService {
    fn new(pool_size: u32) -> Self {
        let info = ServiceInfo::new("database", "1.0.0", "Database connection service")
            .with_tag("database")
            .with_tag("infrastructure")
            .with_metadata("pool_size", pool_size.to_string());

        Self {
            base: df_core::service::BaseService::new(info),
            connection_pool_size: pool_size,
        }
    }
}

#[async_trait]
impl Service for DatabaseService {
    fn info(&self) -> &ServiceInfo {
        self.base.info()
    }

    fn state(&self) -> df_core::service::ServiceState {
        self.base.state()
    }

    async fn initialize(&mut self) -> DfResult<()> {
        println!("Initializing database service with pool size: {}", self.connection_pool_size);
        self.base.initialize().await
    }

    async fn start(&mut self) -> DfResult<()> {
        println!("Starting database service...");
        self.base.start().await
    }

    async fn stop(&mut self) -> DfResult<()> {
        println!("Stopping database service...");
        self.base.stop().await
    }

    async fn health_check(&self) -> DfResult<bool> {
        // Simulate health check
        Ok(self.state() == df_core::service::ServiceState::Running)
    }

    async fn metrics(&self) -> DfResult<HashMap<String, String>> {
        let mut metrics = HashMap::new();
        metrics.insert("pool_size".to_string(), self.connection_pool_size.to_string());
        metrics.insert("active_connections".to_string(), "5".to_string());
        Ok(metrics)
    }
}

// Another example service that depends on the database
struct ApiService {
    base: df_core::service::BaseService,
    port: u16,
}

impl ApiService {
    fn new(port: u16) -> Self {
        let info = ServiceInfo::new("api", "1.0.0", "REST API service")
            .with_dependency("database")  // Depends on database service
            .with_tag("api")
            .with_tag("web")
            .with_metadata("port", port.to_string());

        Self {
            base: df_core::service::BaseService::new(info),
            port,
        }
    }
}

#[async_trait]
impl Service for ApiService {
    fn info(&self) -> &ServiceInfo {
        self.base.info()
    }

    fn state(&self) -> df_core::service::ServiceState {
        self.base.state()
    }

    async fn initialize(&mut self) -> DfResult<()> {
        println!("Initializing API service on port: {}", self.port);
        self.base.initialize().await
    }

    async fn start(&mut self) -> DfResult<()> {
        println!("Starting API service on port {}...", self.port);
        self.base.start().await
    }

    async fn stop(&mut self) -> DfResult<()> {
        println!("Stopping API service...");
        self.base.stop().await
    }

    async fn health_check(&self) -> DfResult<bool> {
        Ok(self.state() == df_core::service::ServiceState::Running)
    }

    async fn metrics(&self) -> DfResult<HashMap<String, String>> {
        let mut metrics = HashMap::new();
        metrics.insert("port".to_string(), self.port.to_string());
        metrics.insert("active_requests".to_string(), "12".to_string());
        Ok(metrics)
    }
}

#[tokio::main]
async fn main() -> DfResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting DreamFactory Core Example");

    // 1. Load Configuration
    println!("\n📁 Loading configuration...");
    let config = Config::builder()
        .with_env("DF_")  // Load from environment variables with DF_ prefix
        .build()?;
    
    println!("Configuration loaded: {} v{}", config.app.name, config.app.version);
    println!("Environment: {}", config.app.environment);
    println!("Database URL: {}", config.database.url);

    // 2. Create Service Registry
    println!("\n🔧 Setting up service registry...");
    let registry = ServiceRegistry::new();

    // 3. Register Services
    println!("\n📦 Registering services...");
    
    // Create services with configuration from loaded config
    let db_service = DatabaseService::new(config.database.max_connections.unwrap_or(10));
    let api_service = ApiService::new(config.server.port);

    registry.register_service(db_service).await?;
    registry.register_service(api_service).await?;

    // Show registered services
    let service_names = registry.get_service_names().await;
    println!("Registered services: {:?}", service_names);

    // Show dependency order
    let startup_order = registry.get_startup_order().await;
    println!("Startup order: {:?}", startup_order);

    // 4. Initialize Services
    println!("\n🔄 Initializing services...");
    registry.initialize_all().await?;
    
    // Check states after initialization
    for service_name in &service_names {
        let state = registry.get_service_state(service_name).await?;
        println!("Service '{}' state: {:?}", service_name, state);
    }

    // 5. Start Services
    println!("\n▶️ Starting services...");
    registry.start_all().await?;

    // 6. Health Check
    println!("\n❤️ Running health checks...");
    let health_results = registry.health_check_all().await?;
    for (service_name, is_healthy) in &health_results {
        let status = if *is_healthy { "✅ Healthy" } else { "❌ Unhealthy" };
        println!("Service '{}': {}", service_name, status);
    }

    // 7. Get Service Information
    println!("\n📊 Service Information:");
    for service_name in &service_names {
        let info = registry.get_service_info(service_name).await?;
        println!("Service: {} v{}", info.name, info.version);
        println!("  Description: {}", info.description);
        println!("  Dependencies: {:?}", info.dependencies);
        println!("  Tags: {:?}", info.tags);
        println!("  Metadata: {:?}", info.metadata);
    }

    // 8. Simulate running for a bit
    println!("\n⏰ Running services for 2 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 9. Graceful Shutdown
    println!("\n🛑 Shutting down services...");
    registry.stop_all().await?;

    // Final health check
    let final_health = registry.health_check_all().await?;
    println!("\n📊 Final health status:");
    for (service_name, is_healthy) in final_health {
        let status = if is_healthy { "✅ Healthy" } else { "❌ Stopped" };
        println!("Service '{}': {}", service_name, status);
    }

    println!("\n✅ Example completed successfully!");
    Ok(())
}

// Example of error handling
async fn demonstrate_error_handling() -> DfResult<()> {
    println!("\n🚨 Demonstrating error handling...");

    let registry = ServiceRegistry::new();
    
    // Try to get a non-existent service
    match registry.get_service_info("non-existent").await {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => {
            println!("Expected error: {}", e);
            println!("Error category: {}", e.category());
            println!("Error is retryable: {}", e.is_retryable());
            println!("HTTP status code: {}", e.status_code());
        }
    }

    // Try to start a service that's not initialized
    let service = DatabaseService::new(10);
    registry.register_service(service).await?;
    
    match registry.start_service("database").await {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => println!("Expected error (service not initialized): {}", e),
    }

    Ok(())
}

// Example of configuration validation
fn demonstrate_configuration_validation() -> DfResult<()> {
    println!("\n✅ Demonstrating configuration validation...");

    // Create a configuration with validation rules
    let config = Config::builder()
        .with_validation(|config| {
            if config.server.port < 1024 {
                Err(DfError::config("Server port must be >= 1024 for non-root users"))
            } else {
                Ok(())
            }
        })
        .with_validation(|config| {
            if config.app.environment == "production" && config.security.jwt_secret == "change-me-in-production" {
                Err(DfError::config("JWT secret must be changed in production"))
            } else {
                Ok(())
            }
        })
        .build()?;

    println!("Configuration validation passed!");
    println!("App: {} ({})", config.app.name, config.app.environment);
    
    Ok(())
}