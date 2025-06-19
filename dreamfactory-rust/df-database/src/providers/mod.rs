pub mod mysql;
pub mod postgresql;
pub mod sqlite;

pub use mysql::*;
pub use postgresql::*;
pub use sqlite::*;

use crate::{DatabaseError, DatabaseProvider, DatabaseService, ConnectionConfig, PoolConfig};

/// Database provider factory
pub struct DatabaseProviderFactory;

impl DatabaseProviderFactory {
    /// Create a database service based on provider type
    pub async fn create_service(
        provider: DatabaseProvider,
        config: ConnectionConfig,
        pool_config: Option<PoolConfig>,
    ) -> Result<Box<dyn DatabaseService>, DatabaseError> {
        let pool_config = pool_config.unwrap_or_default();
        
        match provider {
            DatabaseProvider::MySQL => {
                let service = MySqlProvider::new(config, pool_config).await?;
                Ok(Box::new(service))
            }
            DatabaseProvider::PostgreSQL => {
                let service = PostgreSqlProvider::new(config, pool_config).await?;
                Ok(Box::new(service))
            }
            DatabaseProvider::SQLite => {
                let service = SqliteProvider::new(config, pool_config).await?;
                Ok(Box::new(service))
            }
        }
    }

    /// Create a database service with default configuration
    pub async fn create_service_with_defaults(
        provider: DatabaseProvider,
        database_url: &str,
    ) -> Result<Box<dyn DatabaseService>, DatabaseError> {
        let config = Self::parse_database_url(provider.clone(), database_url)?;
        Self::create_service(provider, config, None).await
    }

    /// Parse database URL into configuration
    fn parse_database_url(
        provider: DatabaseProvider,
        url: &str,
    ) -> Result<ConnectionConfig, DatabaseError> {
        match provider {
            DatabaseProvider::MySQL => {
                if url.starts_with("mysql://") {
                    let url = url::Url::parse(url).map_err(|e| DatabaseError::Configuration {
                        message: format!("Invalid MySQL URL: {}", e),
                    })?;

                    Ok(ConnectionConfig {
                        provider,
                        host: url.host_str().map(|s| s.to_string()),
                        port: url.port(),
                        database: url.path().trim_start_matches('/').to_string(),
                        username: if url.username().is_empty() {
                            None
                        } else {
                            Some(url.username().to_string())
                        },
                        password: url.password().map(|s| s.to_string()),
                        options: std::collections::HashMap::new(),
                    })
                } else {
                    Err(DatabaseError::Configuration {
                        message: "MySQL URL must start with mysql://".to_string(),
                    })
                }
            }
            DatabaseProvider::PostgreSQL => {
                if url.starts_with("postgresql://") || url.starts_with("postgres://") {
                    let url = url::Url::parse(url).map_err(|e| DatabaseError::Configuration {
                        message: format!("Invalid PostgreSQL URL: {}", e),
                    })?;

                    Ok(ConnectionConfig {
                        provider,
                        host: url.host_str().map(|s| s.to_string()),
                        port: url.port(),
                        database: url.path().trim_start_matches('/').to_string(),
                        username: if url.username().is_empty() {
                            None
                        } else {
                            Some(url.username().to_string())
                        },
                        password: url.password().map(|s| s.to_string()),
                        options: std::collections::HashMap::new(),
                    })
                } else {
                    Err(DatabaseError::Configuration {
                        message: "PostgreSQL URL must start with postgresql:// or postgres://".to_string(),
                    })
                }
            }
            DatabaseProvider::SQLite => {
                if url.starts_with("sqlite:") {
                    Ok(ConnectionConfig {
                        provider,
                        host: None,
                        port: None,
                        database: url.strip_prefix("sqlite:").unwrap_or(url).to_string(),
                        username: None,
                        password: None,
                        options: std::collections::HashMap::new(),
                    })
                } else {
                    Err(DatabaseError::Configuration {
                        message: "SQLite URL must start with sqlite:".to_string(),
                    })
                }
            }
        }
    }
}

/// Database service builder for fluent API
pub struct DatabaseServiceBuilder {
    provider: Option<DatabaseProvider>,
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    pool_config: PoolConfig,
    options: std::collections::HashMap<String, serde_json::Value>,
}

impl DatabaseServiceBuilder {
    pub fn new() -> Self {
        Self {
            provider: None,
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            pool_config: PoolConfig::default(),
            options: std::collections::HashMap::new(),
        }
    }

    pub fn provider(mut self, provider: DatabaseProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_string());
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.pool_config.max_connections = max;
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.pool_config.min_connections = min;
        self
    }

    pub fn acquire_timeout(mut self, timeout: u64) -> Self {
        self.pool_config.acquire_timeout = timeout;
        self
    }

    pub fn option(mut self, key: &str, value: serde_json::Value) -> Self {
        self.options.insert(key.to_string(), value);
        self
    }

    pub async fn build(self) -> Result<Box<dyn DatabaseService>, DatabaseError> {
        let provider = self.provider.ok_or_else(|| DatabaseError::Configuration {
            message: "Database provider is required".to_string(),
        })?;

        let database = self.database.ok_or_else(|| DatabaseError::Configuration {
            message: "Database name is required".to_string(),
        })?;

        let config = ConnectionConfig {
            provider: provider.clone(),
            host: self.host,
            port: self.port,
            database,
            username: self.username,
            password: self.password,
            options: self.options,
        };

        DatabaseProviderFactory::create_service(provider, config, Some(self.pool_config)).await
    }
}

impl Default for DatabaseServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}