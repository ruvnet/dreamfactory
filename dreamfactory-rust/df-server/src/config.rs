use anyhow::Result;
use figment::{Figment, providers::{Format, Yaml, Env}};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

/// Main server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub email: EmailConfig,
    pub security: SecurityConfig,
    pub cors: CorsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_request_size: usize,
    pub worker_threads: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub default_connection: String,
    pub connections: std::collections::HashMap<String, DatabaseConnection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConnection {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: Option<String>,
    pub max_connections: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub database: Option<u32>,
    pub password: Option<String>,
    pub prefix: String,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub encryption: String,
    pub from_address: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_ttl_seconds: u64,
    pub bcrypt_rounds: u32,
    pub enable_rate_limiting: bool,
    pub rate_limit_requests: u32,
    pub rate_limit_window_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub file_path: Option<String>,
    pub rotate_size_mb: Option<u64>,
    pub rotate_keep_files: Option<u32>,
}

impl ServerConfig {
    /// Load configuration from file with environment variable overrides
    pub fn load<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref();
        
        info!("Loading configuration from: {}", config_path.display());

        let figment = Figment::new()
            // Start with defaults
            .merge(Yaml::string(&Self::default_yaml()?))
            // Layer on config file if it exists
            .merge(Yaml::file(config_path))
            // Override with environment variables (prefixed with DF_)
            .merge(Env::prefixed("DF_").split("_"));

        let config: ServerConfig = figment.extract()?;
        
        // Validate configuration
        config.validate()?;
        
        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate server settings
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port must be greater than 0"));
        }

        if self.server.timeout_seconds == 0 {
            return Err(anyhow::anyhow!("Server timeout must be greater than 0"));
        }

        // Validate database connections
        if self.database.connections.is_empty() {
            warn!("No database connections configured");
        }

        // Validate security settings
        if self.security.jwt_secret.len() < 32 {
            return Err(anyhow::anyhow!("JWT secret must be at least 32 characters"));
        }

        if self.security.bcrypt_rounds < 4 || self.security.bcrypt_rounds > 31 {
            return Err(anyhow::anyhow!("Bcrypt rounds must be between 4 and 31"));
        }

        info!("Configuration validation passed");
        Ok(())
    }

    /// Get default configuration as YAML string
    fn default_yaml() -> Result<String> {
        let default_config = Self::default();
        Ok(serde_yaml::to_string(&default_config)?)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        let mut database_connections = std::collections::HashMap::new();
        database_connections.insert("default".to_string(), DatabaseConnection {
            driver: "sqlite".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "dreamfactory.db".to_string(),
            username: "df_admin".to_string(),
            password: "df_admin".to_string(),
            ssl_mode: None,
            max_connections: Some(10),
        });

        Self {
            server: ServerSettings {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
                max_request_size: 1024 * 1024 * 16, // 16MB
                worker_threads: None, // Use default
            },
            database: DatabaseConfig {
                default_connection: "default".to_string(),
                connections: database_connections,
            },
            cache: CacheConfig {
                driver: "memory".to_string(),
                host: "localhost".to_string(),
                port: 6379,
                database: Some(0),
                password: None,
                prefix: "df:".to_string(),
                ttl_seconds: 3600, // 1 hour
            },
            email: EmailConfig {
                driver: "smtp".to_string(),
                host: "localhost".to_string(),
                port: 587,
                username: None,
                password: None,
                encryption: "tls".to_string(),
                from_address: "noreply@dreamfactory.local".to_string(),
                from_name: "DreamFactory".to_string(),
            },
            security: SecurityConfig {
                jwt_secret: "your-super-secret-jwt-key-change-this-in-production".to_string(),
                jwt_ttl_seconds: 86400, // 24 hours
                bcrypt_rounds: 12,
                enable_rate_limiting: true,
                rate_limit_requests: 100,
                rate_limit_window_seconds: 60, // 1 minute
            },
            cors: CorsConfig {
                enabled: true,
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "PATCH".to_string(),
                    "OPTIONS".to_string(),
                ],
                allowed_headers: vec![
                    "Authorization".to_string(),
                    "Content-Type".to_string(),
                    "X-Requested-With".to_string(),
                ],
                max_age_seconds: 3600,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
                file_path: None,
                rotate_size_mb: Some(100),
                rotate_keep_files: Some(10),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert!(config.cors.enabled);
        assert!(!config.database.connections.is_empty());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_port() {
        let mut config = ServerConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_short_jwt_secret() {
        let mut config = ServerConfig::default();
        config.security.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_bcrypt_rounds() {
        let mut config = ServerConfig::default();
        config.security.bcrypt_rounds = 2; // Too low
        assert!(config.validate().is_err());

        config.security.bcrypt_rounds = 50; // Too high
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_config_load_from_file() {
        let config_yaml = r#"
server:
  host: "0.0.0.0"
  port: 3000
  timeout_seconds: 60
  max_request_size: 8388608

security:
  jwt_secret: "test-secret-key-that-is-long-enough-for-validation"
  jwt_ttl_seconds: 7200
  bcrypt_rounds: 10

cors:
  enabled: false
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, config_yaml).unwrap();

        let config = ServerConfig::load(temp_file.path()).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.timeout_seconds, 60);
        assert!(!config.cors.enabled);
    }

    #[test]
    fn test_default_yaml_generation() {
        let yaml = ServerConfig::default_yaml().unwrap();
        assert!(!yaml.is_empty());
        assert!(yaml.contains("server:"));
        assert!(yaml.contains("database:"));
        assert!(yaml.contains("cache:"));
    }

    #[test]
    fn test_config_serialization() {
        let config = ServerConfig::default();
        let serialized = serde_yaml::to_string(&config).unwrap();
        let deserialized: ServerConfig = serde_yaml::from_str(&serialized).unwrap();
        
        assert_eq!(config.server.host, deserialized.server.host);
        assert_eq!(config.server.port, deserialized.server.port);
        assert_eq!(config.cors.enabled, deserialized.cors.enabled);
    }
}