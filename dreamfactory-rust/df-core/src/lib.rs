//! # DreamFactory Core Framework
//!
//! This crate provides the core abstractions and infrastructure for the DreamFactory
//! platform, including service management, configuration, error handling, and plugin architecture.
//!
//! ## Features
//!
//! - **Service Management**: Trait-based service abstractions with lifecycle management
//! - **Configuration**: Multi-source configuration loading (YAML, JSON, ENV)
//! - **Error Handling**: Comprehensive error types with proper propagation
//! - **Dependency Injection**: Service registry with automatic dependency resolution
//! - **Plugin Architecture**: Dynamic plugin loading and management
//! - **Async Support**: Full async/await support throughout the framework

pub mod error;
pub mod service;
pub mod config;
pub mod registry;
pub mod plugin;
pub mod integration;


// Re-export commonly used types
pub use error::{DfError, DfResult};
pub use service::{Service, ServiceState, ServiceInfo};
pub use config::{Config, ConfigBuilder, ConfigError};
pub use registry::{ServiceRegistry, RegistryError};
pub use plugin::{Plugin, PluginManager, PluginError};
pub use integration::{IntegratedServiceRegistry, ApiServiceHandler, ServiceContext, ApiRoute};

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        DfError, DfResult,
        Service, ServiceState, ServiceInfo,
        Config, ConfigBuilder,
        ServiceRegistry,
        Plugin, PluginManager,
        IntegratedServiceRegistry, ApiServiceHandler, ServiceContext, ApiRoute,
    };
    pub use async_trait::async_trait;
    pub use uuid::Uuid;
    pub use serde::{Deserialize, Serialize};
}