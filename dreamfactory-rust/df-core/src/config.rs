//! Configuration management system for DreamFactory.
//!
//! This module provides flexible configuration loading from multiple sources:
//! - YAML files
//! - JSON files  
//! - Environment variables
//! - Command line arguments
//! - Default values

use crate::error::{DfError, DfResult};
use figment::{
    providers::{Env, Format, Json, Serialized, Yaml},
    Figment,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Invalid configuration format: {message}")]
    InvalidFormat { message: String },
    
    #[error("Missing required configuration key: {key}")]
    MissingKey { key: String },
    
    #[error("Configuration validation failed: {message}")]
    ValidationFailed { message: String },
    
    #[error("Environment variable error: {message}")]
    EnvError { message: String },
}

impl From<ConfigError> for DfError {
    fn from(err: ConfigError) -> Self {
        DfError::config(err.to_string())
    }
}

/// Configuration builder for creating configuration instances
pub struct ConfigBuilder {
    figment: Figment,
    validation_rules: Vec<Box<dyn Fn(&Config) -> DfResult<()> + Send + Sync>>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            figment: Figment::new(),
            validation_rules: Vec::new(),
        }
    }

    /// Add default values from a serializable struct
    pub fn with_defaults<T: Serialize>(mut self, defaults: T) -> Self {
        self.figment = self.figment.merge(Serialized::defaults(defaults));
        self
    }

    /// Load configuration from a YAML file
    pub fn with_yaml_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.figment = self.figment.merge(Yaml::file(path));
        self
    }

    /// Load configuration from a JSON file
    pub fn with_json_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.figment = self.figment.merge(Json::file(path));
        self
    }

    /// Load configuration from environment variables with a prefix
    pub fn with_env(mut self, prefix: &str) -> Self {
        self.figment = self.figment.merge(Env::prefixed(prefix));
        self
    }

    /// Load configuration from all environment variables
    pub fn with_all_env(mut self) -> Self {
        self.figment = self.figment.merge(Env::raw());
        self
    }

    /// Add a validation rule
    pub fn with_validation<F>(mut self, rule: F) -> Self
    where
        F: Fn(&Config) -> DfResult<()> + Send + Sync + 'static,
    {
        self.validation_rules.push(Box::new(rule));
        self
    }

    /// Build the configuration
    pub fn build(self) -> DfResult<Config> {
        let config: Config = self.figment.extract()
            .map_err(|e| DfError::config(format!("Failed to extract configuration: {}", e)))?;

        // Run validation rules
        for rule in &self.validation_rules {
            rule(&config)?;
        }

        Ok(config)
    }

    /// Build a custom configuration type
    pub fn build_custom<T: DeserializeOwned>(self) -> DfResult<T> {
        self.figment.extract()
            .map_err(|e| DfError::config(format!("Failed to extract configuration: {}", e)))
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main configuration structure for DreamFactory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application settings
    pub app: AppConfig,
    
    /// Server configuration
    pub server: ServerConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Cache configuration
    pub cache: CacheConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Security settings
    pub security: SecurityConfig,
    
    /// Plugin configuration
    pub plugins: PluginConfig,
    
    /// Custom service configurations
    pub services: HashMap<String, serde_json::Value>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub debug: bool,
    pub timezone: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: Option<usize>,
    pub timeout: Option<u64>,
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_file: String,
    pub key_file: String,
    pub ca_file: Option<String>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connection_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
    pub max_lifetime: Option<u64>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub driver: String,
    pub url: Option<String>,
    pub ttl: Option<u64>,
    pub max_entries: Option<usize>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub file: Option<String>,
    pub rotation: Option<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiry: Option<u64>,
    pub cors_origins: Vec<String>,
    pub rate_limit: Option<RateLimitConfig>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u64,
    pub burst_size: u64,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub directory: String,
    pub auto_load: bool,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig {
                name: "dreamfactory".to_string(),
                version: "1.0.0".to_string(),
                environment: "development".to_string(),
                debug: true,
                timezone: "UTC".to_string(),
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: None,
                max_connections: None,
                timeout: None,
                tls: None,
            },
            database: DatabaseConfig {
                url: "sqlite://memory".to_string(),
                max_connections: None,
                min_connections: None,
                connection_timeout: None,
                idle_timeout: None,
                max_lifetime: None,
            },
            cache: CacheConfig {
                enabled: false,
                driver: "memory".to_string(),
                url: None,
                ttl: None,
                max_entries: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
                file: None,
                rotation: None,
            },
            security: SecurityConfig {
                jwt_secret: "change-me-in-production".to_string(),
                jwt_expiry: Some(3600),
                cors_origins: vec!["*".to_string()],
                rate_limit: None,
            },
            plugins: PluginConfig {
                enabled: true,
                directory: "./plugins".to_string(),
                auto_load: true,
                whitelist: Vec::new(),
                blacklist: Vec::new(),
            },
            services: HashMap::new(),
        }
    }
}

impl Config {
    /// Create a new configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new().with_defaults(Config::default())
    }

    /// Load configuration from environment variables with DF_ prefix
    pub fn from_env() -> DfResult<Self> {
        Self::builder()
            .with_env("DF_")
            .build()
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> DfResult<Self> {
        let path_ref = path.as_ref();
        let builder = Self::builder();
        
        let builder = match path_ref.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => builder.with_yaml_file(path),
            Some("json") => builder.with_json_file(path),
            _ => return Err(DfError::config("Unsupported configuration file format")),
        };
        
        builder.build()
    }

    /// Load configuration from file and environment
    pub fn from_file_and_env<P: AsRef<Path>>(path: P) -> DfResult<Self> {
        let path_ref = path.as_ref();
        let builder = Self::builder();
        
        let builder = match path_ref.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => builder.with_yaml_file(path),
            Some("json") => builder.with_json_file(path),
            _ => return Err(DfError::config("Unsupported configuration file format")),
        };
        
        builder.with_env("DF_").build()
    }

    /// Validate configuration
    pub fn validate(&self) -> DfResult<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(DfError::config("Server port cannot be 0"));
        }

        // Validate database URL
        if self.database.url.is_empty() {
            return Err(DfError::config("Database URL cannot be empty"));
        }

        // Validate JWT secret in production
        if self.app.environment == "production" && self.security.jwt_secret == "change-me-in-production" {
            return Err(DfError::config("JWT secret must be changed in production"));
        }

        // Validate logging level
        match self.logging.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => return Err(DfError::config("Invalid logging level")),
        }

        Ok(())
    }

    /// Get service configuration by name
    pub fn get_service_config<T: DeserializeOwned>(&self, service_name: &str) -> DfResult<T> {
        self.services
            .get(service_name)
            .ok_or_else(|| DfError::config(format!("Service configuration not found: {}", service_name)))
            .and_then(|value| {
                serde_json::from_value(value.clone())
                    .map_err(|e| DfError::config(format!("Failed to deserialize service config: {}", e)))
            })
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        self.app.environment == "development"
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        self.app.environment == "production"
    }

    /// Check if debug mode is enabled
    pub fn is_debug(&self) -> bool {
        self.app.debug
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        assert_eq!(config.app.name, "dreamfactory");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.url, "sqlite://memory");
        assert!(!config.cache.enabled);
        assert!(config.plugins.enabled);
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = Config::builder().build().unwrap();
        
        assert_eq!(config.app.name, "dreamfactory");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_config_builder_with_custom_defaults() {
        #[derive(Serialize)]
        struct CustomDefaults {
            app: AppConfig,
        }

        let custom_defaults = CustomDefaults {
            app: AppConfig {
                name: "custom-app".to_string(),
                version: "2.0.0".to_string(),
                environment: "test".to_string(),
                debug: false,
                timezone: "EST".to_string(),
            },
        };

        let config = ConfigBuilder::new()
            .with_defaults(custom_defaults)
            .build_custom::<Config>()
            .unwrap();

        assert_eq!(config.app.name, "custom-app");
        assert_eq!(config.app.version, "2.0.0");
        assert_eq!(config.app.environment, "test");
    }

    #[test]
    fn test_config_from_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        
        let yaml_content = r#"
app:
  name: "test-app"
  version: "1.2.3"
  environment: "test"
  debug: false
  timezone: "UTC"

server:
  host: "0.0.0.0"
  port: 9000
  workers: 4

database:
  url: "postgresql://user:pass@localhost/test"
  max_connections: 10

cache:
  enabled: true
  driver: "redis"
  url: "redis://localhost:6379"

logging:
  level: "debug"
  format: "json"
  output: "file"
  file: "/var/log/app.log"

security:
  jwt_secret: "super-secret-key"
  jwt_expiry: 7200
  cors_origins:
    - "https://example.com"
    - "https://api.example.com"

plugins:
  enabled: true
  directory: "/opt/plugins"
  auto_load: false
  whitelist:
    - "auth-plugin"
    - "db-plugin"

services:
  user_service:
    max_users: 1000
    timeout: 30
"#;

        fs::write(&config_path, yaml_content).unwrap();
        
        let config = Config::from_file(&config_path).unwrap();
        
        assert_eq!(config.app.name, "test-app");
        assert_eq!(config.app.version, "1.2.3");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.server.workers, Some(4));
        assert_eq!(config.database.url, "postgresql://user:pass@localhost/test");
        assert_eq!(config.database.max_connections, Some(10));
        assert!(config.cache.enabled);
        assert_eq!(config.cache.driver, "redis");
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.security.jwt_secret, "super-secret-key");
        assert_eq!(config.security.cors_origins, vec!["https://example.com", "https://api.example.com"]);
        assert_eq!(config.plugins.directory, "/opt/plugins");
        assert!(!config.plugins.auto_load);
        assert_eq!(config.plugins.whitelist, vec!["auth-plugin", "db-plugin"]);
    }

    #[test]
    fn test_config_from_json_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let json_content = r#"
{
  "app": {
    "name": "json-app",
    "version": "3.0.0",
    "environment": "staging",
    "debug": true,
    "timezone": "PST"
  },
  "server": {
    "host": "127.0.0.1",
    "port": 8888
  },
  "database": {
    "url": "mysql://user:pass@localhost/test"
  },
  "cache": {
    "enabled": false,
    "driver": "memory"
  },
  "logging": {
    "level": "warn",
    "format": "text",
    "output": "stdout"
  },
  "security": {
    "jwt_secret": "json-secret",
    "cors_origins": ["*"]
  },
  "plugins": {
    "enabled": false,
    "directory": "./plugins",
    "auto_load": true,
    "whitelist": [],
    "blacklist": []
  },
  "services": {}
}
"#;

        fs::write(&config_path, json_content).unwrap();
        
        let config = Config::from_file(&config_path).unwrap();
        
        assert_eq!(config.app.name, "json-app");
        assert_eq!(config.app.version, "3.0.0");
        assert_eq!(config.server.port, 8888);
        assert_eq!(config.database.url, "mysql://user:pass@localhost/test");
        assert!(!config.cache.enabled);
        assert_eq!(config.logging.level, "warn");
        assert!(!config.plugins.enabled);
    }

    #[test] 
    fn test_config_from_env() {
        // Set environment variables
        env::set_var("DF_APP_NAME", "env-app");
        env::set_var("DF_APP_VERSION", "4.0.0");
        env::set_var("DF_SERVER_PORT", "7777");
        env::set_var("DF_DATABASE_URL", "env://database");
        env::set_var("DF_CACHE_ENABLED", "true");
        env::set_var("DF_LOGGING_LEVEL", "error");

        let config = Config::from_env().unwrap();

        assert_eq!(config.app.name, "env-app");
        assert_eq!(config.app.version, "4.0.0");
        assert_eq!(config.server.port, 7777);
        assert_eq!(config.database.url, "env://database");
        assert!(config.cache.enabled);
        assert_eq!(config.logging.level, "error");

        // Clean up
        env::remove_var("DF_APP_NAME");
        env::remove_var("DF_APP_VERSION");
        env::remove_var("DF_SERVER_PORT");
        env::remove_var("DF_DATABASE_URL");
        env::remove_var("DF_CACHE_ENABLED");
        env::remove_var("DF_LOGGING_LEVEL");
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());
        config.server.port = 8080;

        // Empty database URL
        config.database.url = "".to_string();
        assert!(config.validate().is_err());
        config.database.url = "sqlite://memory".to_string();

        // Production with default JWT secret
        config.app.environment = "production".to_string();
        assert!(config.validate().is_err());
        config.security.jwt_secret = "production-secret".to_string();
        assert!(config.validate().is_ok());

        // Invalid logging level
        config.logging.level = "invalid".to_string();
        assert!(config.validate().is_err());
        config.logging.level = "info".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_builder_validation() {
        let result = Config::builder()
            .with_validation(|config| {
                if config.server.port < 1024 {
                    Err(DfError::config("Port must be >= 1024"))
                } else {
                    Ok(())
                }
            })
            .build();

        // Should fail with default port 8080 > 1024, so this should pass
        assert!(result.is_ok());

        // Test with invalid port
        #[derive(Serialize)]
        struct InvalidPortConfig {
            server: ServerConfig,
        }

        let invalid_config = InvalidPortConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 80,
                workers: None,
                max_connections: None,
                timeout: None,
                tls: None,
            },
        };

        let result = ConfigBuilder::new()
            .with_defaults(invalid_config)
            .with_validation(|config: &Config| {
                if config.server.port < 1024 {
                    Err(DfError::config("Port must be >= 1024"))
                } else {
                    Ok(())
                }
            })
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_config_service_configuration() {
        let mut config = Config::default();
        
        // Add service configuration
        let service_config = serde_json::json!({
            "max_connections": 100,
            "timeout": 30,
            "retries": 3
        });
        config.services.insert("test_service".to_string(), service_config);

        #[derive(Deserialize, PartialEq, Debug)]
        struct TestServiceConfig {
            max_connections: u32,
            timeout: u32,
            retries: u32,
        }

        let service_config: TestServiceConfig = config.get_service_config("test_service").unwrap();
        assert_eq!(service_config.max_connections, 100);
        assert_eq!(service_config.timeout, 30);
        assert_eq!(service_config.retries, 3);

        // Test missing service
        let result: Result<TestServiceConfig, _> = config.get_service_config("missing_service");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_environment_helpers() {
        let mut config = Config::default();
        
        config.app.environment = "development".to_string();
        assert!(config.is_development());
        assert!(!config.is_production());

        config.app.environment = "production".to_string();
        assert!(!config.is_development());
        assert!(config.is_production());

        config.app.debug = true;
        assert!(config.is_debug());
        
        config.app.debug = false;
        assert!(!config.is_debug());
    }

    #[test]
    fn test_config_file_and_env_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        
        let yaml_content = r#"
app:
  name: "file-app"
  version: "1.0.0"
server:
  port: 8080
"#;

        fs::write(&config_path, yaml_content).unwrap();

        // Set env var that should override file
        env::set_var("DF_APP_NAME", "env-override-app");
        env::set_var("DF_SERVER_PORT", "9999");

        let config = Config::from_file_and_env(&config_path).unwrap();

        // Environment should override file
        assert_eq!(config.app.name, "env-override-app");
        assert_eq!(config.server.port, 9999);
        assert_eq!(config.app.version, "1.0.0"); // From file

        // Clean up
        env::remove_var("DF_APP_NAME");
        env::remove_var("DF_SERVER_PORT");
    }

    #[test]
    fn test_unsupported_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.txt");
        fs::write(&config_path, "invalid content").unwrap();

        let result = Config::from_file(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_tls_config() {
        let tls_config = TlsConfig {
            cert_file: "/path/to/cert.pem".to_string(),
            key_file: "/path/to/key.pem".to_string(),
            ca_file: Some("/path/to/ca.pem".to_string()),
        };

        let json = serde_json::to_string(&tls_config).unwrap();
        let deserialized: TlsConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(tls_config.cert_file, deserialized.cert_file);
        assert_eq!(tls_config.key_file, deserialized.key_file);
        assert_eq!(tls_config.ca_file, deserialized.ca_file);
    }

    #[test]
    fn test_rate_limit_config() {
        let rate_limit = RateLimitConfig {
            requests_per_minute: 100,
            burst_size: 10,
        };

        let json = serde_json::to_string(&rate_limit).unwrap();
        let deserialized: RateLimitConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(rate_limit.requests_per_minute, deserialized.requests_per_minute);
        assert_eq!(rate_limit.burst_size, deserialized.burst_size);
    }
}