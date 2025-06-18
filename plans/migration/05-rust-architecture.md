# DreamFactory Rust Architecture Design

## Executive Summary

This document outlines a comprehensive Rust architecture for migrating DreamFactory from PHP/Laravel to a modern, high-performance, async-first Rust implementation. The architecture leverages Rust's type safety, performance, and concurrency features while maintaining API compatibility and extending functionality.

## Architecture Overview

### Core Principles

1. **Async-First Design**: All I/O operations use async/await patterns
2. **Type Safety**: Leverage Rust's type system for compile-time guarantees
3. **Modular Plugin Architecture**: Extensible service and middleware system
4. **Performance Optimization**: Zero-cost abstractions and efficient memory usage
5. **API Compatibility**: Maintain REST API compatibility with current DreamFactory
6. **Configuration-Driven**: Runtime service configuration without recompilation

## Technology Stack Selection

### Web Framework: Axum
**Rationale**: Axum provides the best balance of performance, ergonomics, and ecosystem compatibility.

```rust
// Core advantages:
// - Built on Hyper and Tower ecosystem
// - Excellent async performance
// - Type-safe routing and extraction
// - Middleware composition
// - WebSocket support
// - Streaming responses
```

**Alternatives Considered**:
- **Actix-Web**: High performance but more complex actor model
- **Warp**: Functional approach but less ecosystem integration
- **Rocket**: Excellent ergonomics but sync-first design

### Database Layer: SQLx + SeaORM
**Rationale**: Compile-time checked queries with ORM convenience.

```rust
// SQLx for raw queries and migrations
// SeaORM for entity management and relationships
// Support for MySQL, PostgreSQL, SQLite, MongoDB
```

### Configuration: Figment
**Rationale**: Flexible configuration merging from multiple sources.

### Serialization: Serde
**Rationale**: De facto standard with excellent performance and ecosystem.

## Module Structure

```
dreamfactory-rust/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── app.rs              # Application container
│   │   ├── config.rs           # Configuration management
│   │   ├── error.rs            # Error handling
│   │   ├── middleware.rs       # Core middleware
│   │   ├── router.rs           # Route management
│   │   └── traits.rs           # Core trait definitions
│   ├── services/
│   │   ├── mod.rs
│   │   ├── base.rs             # Base service traits
│   │   ├── database/
│   │   │   ├── mod.rs
│   │   │   ├── sql.rs          # SQL database services
│   │   │   ├── nosql.rs        # NoSQL database services
│   │   │   └── schema.rs       # Schema management
│   │   ├── file/
│   │   │   ├── mod.rs
│   │   │   ├── local.rs        # Local file system
│   │   │   ├── s3.rs          # AWS S3 service
│   │   │   └── azure.rs       # Azure storage
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   ├── jwt.rs         # JWT authentication
│   │   │   ├── oauth.rs       # OAuth providers
│   │   │   └── session.rs     # Session management
│   │   ├── email/
│   │   │   ├── mod.rs
│   │   │   ├── smtp.rs        # SMTP email service
│   │   │   └── providers.rs   # Email service providers
│   │   └── system/
│   │       ├── mod.rs
│   │       ├── admin.rs       # Admin endpoints
│   │       ├── cache.rs       # Cache management
│   │       └── events.rs      # Event system
│   ├── plugins/
│   │   ├── mod.rs
│   │   ├── registry.rs        # Plugin registry
│   │   ├── loader.rs          # Dynamic plugin loading
│   │   └── traits.rs          # Plugin trait definitions
│   ├── api/
│   │   ├── mod.rs
│   │   ├── handlers.rs        # Request handlers
│   │   ├── extractors.rs      # Custom extractors
│   │   ├── responses.rs       # Response builders
│   │   └── validation.rs      # Input validation
│   ├── models/
│   │   ├── mod.rs
│   │   ├── entities/          # Database entities
│   │   ├── dto/              # Data transfer objects
│   │   └── requests.rs       # Request models
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── crypto.rs         # Cryptographic utilities
│   │   ├── time.rs           # Time utilities
│   │   └── validation.rs     # Validation helpers
│   └── migrations/
│       └── mod.rs
├── plugins/                   # External plugin directory
├── config/                   # Configuration files
├── tests/
└── docs/
```

## Core Architecture Components

### 1. Application Container

```rust
// src/core/app.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::Router;
use crate::services::ServiceRegistry;
use crate::plugins::PluginRegistry;

#[derive(Clone)]
pub struct DreamFactoryApp {
    pub config: Arc<AppConfig>,
    pub services: Arc<RwLock<ServiceRegistry>>,
    pub plugins: Arc<RwLock<PluginRegistry>>,
    pub db_pool: Arc<DatabasePool>,
    pub cache: Arc<dyn CacheProvider>,
}

impl DreamFactoryApp {
    pub async fn new(config: AppConfig) -> Result<Self, AppError> {
        let db_pool = Arc::new(create_database_pool(&config.database).await?);
        let cache = Arc::new(create_cache_provider(&config.cache).await?);
        
        let services = Arc::new(RwLock::new(ServiceRegistry::new()));
        let plugins = Arc::new(RwLock::new(PluginRegistry::new()));
        
        Ok(Self {
            config: Arc::new(config),
            services,
            plugins,
            db_pool,
            cache,
        })
    }
    
    pub async fn build_router(&self) -> Router {
        let mut router = Router::new();
        
        // System routes
        router = router.nest("/api/v2/system", self.build_system_routes().await);
        
        // Service routes
        let services = self.services.read().await;
        for (name, service) in services.iter() {
            let service_router = service.build_router().await;
            router = router.nest(&format!("/api/v2/{}", name), service_router);
        }
        
        // Plugin routes
        let plugins = self.plugins.read().await;
        for plugin in plugins.active_plugins() {
            if let Some(plugin_router) = plugin.build_router().await {
                router = router.merge(plugin_router);
            }
        }
        
        router
    }
}
```

### 2. Service Architecture

```rust
// src/services/base.rs
use std::collections::HashMap;
use axum::{Router, extract::State};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait Service: Send + Sync {
    fn name(&self) -> &str;
    fn service_type(&self) -> &str;
    fn is_active(&self) -> bool;
    
    async fn initialize(&mut self, config: ServiceConfig) -> Result<(), ServiceError>;
    async fn build_router(&self) -> Router;
    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, ServiceError>;
    async fn get_resources(&self) -> Vec<ResourceInfo>;
    async fn get_api_doc(&self) -> ApiDocumentation;
}

#[async_trait]
pub trait DatabaseService: Service {
    async fn execute_query(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, ServiceError>;
    async fn get_schema(&self) -> Result<Schema, ServiceError>;
    async fn create_table(&self, definition: TableDefinition) -> Result<(), ServiceError>;
    async fn insert_records(&self, table: &str, records: Vec<Record>) -> Result<Vec<InsertResult>, ServiceError>;
    async fn update_records(&self, table: &str, updates: Vec<UpdateRecord>) -> Result<Vec<UpdateResult>, ServiceError>;
    async fn delete_records(&self, table: &str, conditions: Vec<Condition>) -> Result<DeleteResult, ServiceError>;
}

#[async_trait]
pub trait FileService: Service {
    async fn list_files(&self, path: &str) -> Result<Vec<FileInfo>, ServiceError>;
    async fn get_file(&self, path: &str) -> Result<FileContent, ServiceError>;
    async fn upload_file(&self, path: &str, content: FileContent) -> Result<FileInfo, ServiceError>;
    async fn delete_file(&self, path: &str) -> Result<(), ServiceError>;
}

pub struct ServiceRegistry {
    services: HashMap<String, Box<dyn Service>>,
    service_configs: HashMap<String, ServiceConfig>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            service_configs: HashMap::new(),
        }
    }
    
    pub async fn register<S: Service + 'static>(&mut self, mut service: S, config: ServiceConfig) -> Result<(), ServiceError> {
        service.initialize(config.clone()).await?;
        let name = service.name().to_string();
        self.services.insert(name.clone(), Box::new(service));
        self.service_configs.insert(name, config);
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Service> {
        self.services.get(name).map(|s| s.as_ref())
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Box<dyn Service>)> {
        self.services.iter()
    }
}
```

### 3. Async Database Layer

```rust
// src/services/database/sql.rs
use sqlx::{Database, Pool, Row};
use sea_orm::{Database as SeaDatabase, DatabaseConnection, EntityTrait};
use async_trait::async_trait;

pub struct SqlDatabaseService<DB: Database> {
    name: String,
    pool: Pool<DB>,
    sea_db: Option<DatabaseConnection>,
    config: DatabaseConfig,
    schema_cache: Arc<RwLock<Option<Schema>>>,
}

impl<DB: Database> SqlDatabaseService<DB> {
    pub async fn new(name: String, config: DatabaseConfig) -> Result<Self, ServiceError> {
        let pool = create_pool(&config).await?;
        let sea_db = if config.use_orm {
            Some(SeaDatabase::connect(&config.connection_string).await?)
        } else {
            None
        };
        
        Ok(Self {
            name,
            pool,
            sea_db,
            config,
            schema_cache: Arc::new(RwLock::new(None)),
        })
    }
    
    async fn get_cached_schema(&self) -> Result<Schema, ServiceError> {
        let cache = self.schema_cache.read().await;
        if let Some(schema) = cache.as_ref() {
            return Ok(schema.clone());
        }
        drop(cache);
        
        let schema = self.load_schema().await?;
        *self.schema_cache.write().await = Some(schema.clone());
        Ok(schema)
    }
    
    async fn load_schema(&self) -> Result<Schema, ServiceError> {
        let query = self.build_schema_query();
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;
            
        let mut schema = Schema::new();
        for row in rows {
            let table_name: String = row.get("table_name");
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let is_nullable: bool = row.get("is_nullable");
            
            schema.add_column(table_name, column_name, data_type, is_nullable);
        }
        
        Ok(schema)
    }
}

#[async_trait]
impl<DB: Database + 'static> DatabaseService for SqlDatabaseService<DB> {
    async fn execute_query(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, ServiceError> {
        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        let rows = query_builder.fetch_all(&self.pool).await?;
        Ok(QueryResult::from_rows(rows))
    }
    
    async fn get_schema(&self) -> Result<Schema, ServiceError> {
        self.get_cached_schema().await
    }
    
    async fn insert_records(&self, table: &str, records: Vec<Record>) -> Result<Vec<InsertResult>, ServiceError> {
        let mut results = Vec::new();
        let mut tx = self.pool.begin().await?;
        
        for record in records {
            let (query, params) = self.build_insert_query(table, &record)?;
            let result = sqlx::query(&query)
                .bind_all(params)
                .execute(&mut *tx)
                .await?;
                
            results.push(InsertResult {
                id: result.last_insert_id().map(|id| id.into()),
                affected_rows: result.rows_affected(),
            });
        }
        
        tx.commit().await?;
        Ok(results)
    }
    
    // Additional methods...
}
```

### 4. Plugin Architecture

```rust
// src/plugins/traits.rs
use async_trait::async_trait;
use axum::Router;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    async fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError>;
    async fn activate(&mut self) -> Result<(), PluginError>;
    async fn deactivate(&mut self) -> Result<(), PluginError>;
    
    // Optional router for adding custom endpoints
    async fn build_router(&self) -> Option<Router> {
        None
    }
    
    // Hook system for extending core functionality
    async fn pre_request_hook(&self, request: &mut ServiceRequest) -> Result<(), PluginError> {
        Ok(())
    }
    
    async fn post_request_hook(&self, request: &ServiceRequest, response: &mut ServiceResponse) -> Result<(), PluginError> {
        Ok(())
    }
    
    async fn pre_service_hook(&self, service: &str, operation: &str, data: &mut serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }
    
    async fn post_service_hook(&self, service: &str, operation: &str, data: &serde_json::Value, result: &mut serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }
}

// Plugin registry with dynamic loading
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_configs: HashMap<String, PluginConfig>,
    active_plugins: HashSet<String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_configs: HashMap::new(),
            active_plugins: HashSet::new(),
        }
    }
    
    pub async fn load_plugin(&mut self, path: &str) -> Result<(), PluginError> {
        // Dynamic plugin loading using libloading
        let plugin = unsafe { self.load_plugin_from_path(path)? };
        let name = plugin.name().to_string();
        
        self.plugins.insert(name.clone(), plugin);
        Ok(())
    }
    
    pub async fn activate_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.activate().await?;
            self.active_plugins.insert(name.to_string());
        }
        Ok(())
    }
    
    pub fn active_plugins(&self) -> impl Iterator<Item = &dyn Plugin> {
        self.active_plugins.iter()
            .filter_map(|name| self.plugins.get(name))
            .map(|plugin| plugin.as_ref())
    }
}
```

### 5. Authentication & Authorization

```rust
// src/services/auth/jwt.rs
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Algorithm, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // Subject (user ID)
    pub iat: i64,             // Issued at
    pub exp: i64,             // Expiration
    pub iss: String,          // Issuer
    pub aud: String,          // Audience
    pub user_id: i64,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub session_id: String,
    pub is_admin: bool,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    issuer: String,
    audience: String,
    ttl: Duration,
}

impl JwtService {
    pub fn new(secret: &str, issuer: String, audience: String, ttl_minutes: i64) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[&audience]);
        validation.set_issuer(&[&issuer]);
        
        Self {
            encoding_key,
            decoding_key,
            validation,
            issuer,
            audience,
            ttl: Duration::minutes(ttl_minutes),
        }
    }
    
    pub fn generate_token(&self, user: &User, session_id: String) -> Result<String, AuthError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user.id.to_string(),
            iat: now.timestamp(),
            exp: (now + self.ttl).timestamp(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            user_id: user.id,
            email: user.email.clone(),
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
            session_id,
            is_admin: user.is_admin,
        };
        
        encode(&jsonwebtoken::Header::default(), &claims, &self.encoding_key)
            .map_err(AuthError::JwtError)
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(AuthError::JwtError)
    }
    
    pub async fn refresh_token(&self, token: &str) -> Result<String, AuthError> {
        let claims = self.validate_token(token)?;
        
        // Verify the token is still valid for refresh (not expired beyond grace period)
        let grace_period = Duration::minutes(5);
        let now = Utc::now();
        if now.timestamp() > claims.exp + grace_period.num_seconds() {
            return Err(AuthError::TokenExpired);
        }
        
        // Generate new token with updated expiration
        let new_claims = Claims {
            exp: (now + self.ttl).timestamp(),
            iat: now.timestamp(),
            ..claims
        };
        
        encode(&jsonwebtoken::Header::default(), &new_claims, &self.encoding_key)
            .map_err(AuthError::JwtError)
    }
}

// Axum extractor for JWT authentication
#[derive(Debug)]
pub struct JwtAuth {
    pub claims: Claims,
}

#[async_trait]
impl<B> FromRequest<B> for JwtAuth
where
    B: Send,
{
    type Rejection = AuthError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let jwt_service = req.extensions()
            .get::<JwtService>()
            .ok_or(AuthError::MissingJwtService)?;
            
        let token = extract_token_from_headers(req.headers())?;
        let claims = jwt_service.validate_token(&token)?;
        
        Ok(JwtAuth { claims })
    }
}
```

### 6. Error Handling

```rust
// src/core/error.rs
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),
    
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
    
    #[error("Internal server error")]
    Internal(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::Auth(AuthError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, "Invalid authentication token")
            }
            AppError::Auth(AuthError::TokenExpired) => {
                (StatusCode::UNAUTHORIZED, "Authentication token expired")
            }
            AppError::Auth(_) => {
                (StatusCode::UNAUTHORIZED, "Authentication failed")
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg.as_str())
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized")
            }
            AppError::Forbidden => {
                (StatusCode::FORBIDDEN, "Forbidden")
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
            AppError::Validation(err) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")
            }
            _ => {
                tracing::error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": error_message,
            }
        }));

        (status, body).into_response()
    }
}

// Custom error types for different domains
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service not found: {0}")]
    NotFound(String),
    
    #[error("Service configuration error: {0}")]
    Configuration(String),
    
    #[error("Service initialization failed: {0}")]
    Initialization(String),
    
    #[error("Service operation failed: {0}")]
    Operation(String),
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("JWT error: {0}")]
    JwtError(jsonwebtoken::errors::Error),
    
    #[error("Missing JWT service")]
    MissingJwtService,
}
```

## Async Patterns and Concurrency Model

### 1. Request Processing Pipeline

```rust
// src/api/handlers.rs
use axum::{
    extract::{Path, Query, State},
    http::{Method, HeaderMap},
    Json, response::Json as ResponseJson,
};
use serde_json::Value;
use std::collections::HashMap;

// Main service handler with async processing
pub async fn handle_service_request(
    State(app): State<DreamFactoryApp>,
    Path((service_name, resource)): Path<(String, Option<String>)>,
    method: Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    body: Option<Json<Value>>,
    auth: Option<JwtAuth>,
) -> Result<ResponseJson<Value>, AppError> {
    // Create service request
    let request = ServiceRequest {
        method: method.as_str().to_string(),
        service: service_name.clone(),
        resource: resource.unwrap_or_default(),
        parameters: params,
        headers: headers.clone(),
        body: body.map(|b| b.0),
        auth: auth.map(|a| a.claims),
    };
    
    // Pre-request plugin hooks
    let plugins = app.plugins.read().await;
    for plugin in plugins.active_plugins() {
        plugin.pre_request_hook(&mut request).await?;
    }
    drop(plugins);
    
    // Get service and process request
    let services = app.services.read().await;
    let service = services.get(&service_name)
        .ok_or_else(|| AppError::NotFound(format!("Service '{}' not found", service_name)))?;
    
    let mut response = service.handle_request(request.clone()).await?;
    drop(services);
    
    // Post-request plugin hooks
    let plugins = app.plugins.read().await;
    for plugin in plugins.active_plugins() {
        plugin.post_request_hook(&request, &mut response).await?;
    }
    
    Ok(ResponseJson(response.data))
}

// Concurrent batch operations
pub async fn handle_batch_request(
    State(app): State<DreamFactoryApp>,
    Json(batch_request): Json<BatchRequest>,
    auth: JwtAuth,
) -> Result<ResponseJson<BatchResponse>, AppError> {
    use futures::future::join_all;
    
    // Process requests concurrently
    let futures = batch_request.requests.into_iter().map(|req| {
        let app = app.clone();
        let auth = auth.claims.clone();
        
        async move {
            let service_request = ServiceRequest {
                method: req.method,
                service: req.service,
                resource: req.resource,
                parameters: req.parameters,
                headers: HeaderMap::new(),
                body: req.body,
                auth: Some(auth),
            };
            
            let services = app.services.read().await;
            let service = services.get(&service_request.service)?;
            service.handle_request(service_request).await
        }
    });
    
    let results = join_all(futures).await;
    
    let responses = results.into_iter()
        .map(|result| match result {
            Ok(response) => BatchResponseItem::Success(response),
            Err(error) => BatchResponseItem::Error(error.to_string()),
        })
        .collect();
    
    Ok(ResponseJson(BatchResponse { responses }))
}
```

### 2. Database Connection Pooling

```rust
// src/services/database/pool.rs
use sqlx::{Pool, Postgres, MySql, Sqlite};
use std::time::Duration;

pub struct DatabasePool {
    pool_type: DatabaseType,
    postgres_pool: Option<Pool<Postgres>>,
    mysql_pool: Option<Pool<MySql>>,
    sqlite_pool: Option<Pool<Sqlite>>,
}

impl DatabasePool {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, sqlx::Error> {
        let mut pool = DatabasePool {
            pool_type: config.database_type.clone(),
            postgres_pool: None,
            mysql_pool: None,
            sqlite_pool: None,
        };
        
        match config.database_type {
            DatabaseType::PostgreSQL => {
                pool.postgres_pool = Some(
                    sqlx::postgres::PgPoolOptions::new()
                        .max_connections(config.max_connections)
                        .acquire_timeout(Duration::from_secs(config.timeout_seconds))
                        .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
                        .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
                        .connect(&config.connection_string)
                        .await?
                );
            }
            DatabaseType::MySQL => {
                pool.mysql_pool = Some(
                    sqlx::mysql::MySqlPoolOptions::new()
                        .max_connections(config.max_connections)
                        .acquire_timeout(Duration::from_secs(config.timeout_seconds))
                        .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
                        .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
                        .connect(&config.connection_string)
                        .await?
                );
            }
            DatabaseType::SQLite => {
                pool.sqlite_pool = Some(
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(config.max_connections)
                        .acquire_timeout(Duration::from_secs(config.timeout_seconds))
                        .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
                        .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
                        .connect(&config.connection_string)
                        .await?
                );
            }
        }
        
        Ok(pool)
    }
    
    pub async fn execute_query(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, sqlx::Error> {
        match self.pool_type {
            DatabaseType::PostgreSQL => {
                let pool = self.postgres_pool.as_ref().unwrap();
                let mut query_builder = sqlx::query(query);
                for param in params {
                    query_builder = query_builder.bind(param);
                }
                let rows = query_builder.fetch_all(pool).await?;
                Ok(QueryResult::from_pg_rows(rows))
            }
            DatabaseType::MySQL => {
                let pool = self.mysql_pool.as_ref().unwrap();
                let mut query_builder = sqlx::query(query);
                for param in params {
                    query_builder = query_builder.bind(param);
                }
                let rows = query_builder.fetch_all(pool).await?;
                Ok(QueryResult::from_mysql_rows(rows))
            }
            DatabaseType::SQLite => {
                let pool = self.sqlite_pool.as_ref().unwrap();
                let mut query_builder = sqlx::query(query);
                for param in params {
                    query_builder = query_builder.bind(param);
                }
                let rows = query_builder.fetch_all(pool).await?;
                Ok(QueryResult::from_sqlite_rows(rows))
            }
        }
    }
}
```

## Configuration Management

### 1. Hierarchical Configuration

```rust
// src/core/config.rs
use figment::{Figment, providers::{Format, Toml, Yaml, Json, Env}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub auth: AuthConfig,
    pub services: HashMap<String, ServiceConfig>,
    pub plugins: HashMap<String, PluginConfig>,
    pub logging: LoggingConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: usize,
    pub timeout_seconds: u64,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub database_type: DatabaseType,
    pub connection_string: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub enable_logging: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    pub service_type: String,
    pub is_active: bool,
    pub config: serde_json::Value,
    pub permissions: Vec<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Figment::new()
            // Default configuration
            .merge(Toml::file("config/default.toml"))
            // Environment-specific configuration
            .merge(Toml::file(format!("config/{}.toml", 
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()))))
            // Local overrides
            .merge(Toml::file("config/local.toml"))
            // Environment variables with DF_ prefix
            .merge(Env::prefixed("DF_").split("_"))
            // Command line arguments
            .merge(clap::command!().get_matches());
            
        config.extract().map_err(ConfigError::from)
    }
    
    pub fn get_service_config(&self, service_name: &str) -> Option<&ServiceConfig> {
        self.services.get(service_name)
    }
    
    pub fn get_plugin_config(&self, plugin_name: &str) -> Option<&PluginConfig> {
        self.plugins.get(plugin_name)
    }
}

// Hot configuration reloading
pub struct ConfigWatcher {
    config: Arc<RwLock<AppConfig>>,
    watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn new(config: Arc<RwLock<AppConfig>>) -> Result<Self, ConfigError> {
        use notify::{Watcher, RecursiveMode, Event, EventKind};
        
        let config_clone = config.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        if let Ok(new_config) = AppConfig::load() {
                            *config_clone.write().unwrap() = new_config;
                            tracing::info!("Configuration reloaded");
                        }
                    }
                }
            },
            notify::Config::default(),
        )?;
        
        watcher.watch(Path::new("config/"), RecursiveMode::Recursive)?;
        
        Ok(Self { config, watcher })
    }
}
```

## Performance Optimization Strategies

### 1. Caching Layer

```rust
// src/core/cache.rs
use std::time::Duration;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait CacheProvider: Send + Sync {
    async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, CacheError>;
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    async fn clear(&self) -> Result<(), CacheError>;
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
}

// Redis implementation
pub struct RedisCache {
    client: redis::Client,
    connection: Arc<Mutex<redis::aio::Connection>>,
}

#[async_trait]
impl CacheProvider for RedisCache {
    async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, CacheError> {
        use redis::AsyncCommands;
        
        let mut conn = self.connection.lock().await;
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(json_str) => {
                let deserialized = serde_json::from_str(&json_str)?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }
    
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError> {
        use redis::AsyncCommands;
        
        let json_str = serde_json::to_string(value)?;
        let mut conn = self.connection.lock().await;
        
        if let Some(duration) = ttl {
            conn.setex(key, duration.as_secs() as usize, json_str).await?;
        } else {
            conn.set(key, json_str).await?;
        }
        
        Ok(())
    }
}

// In-memory cache with LRU eviction
pub struct MemoryCache {
    cache: Arc<Mutex<lru::LruCache<String, (serde_json::Value, Option<std::time::Instant>)>>>,
    max_size: usize,
}

impl MemoryCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(lru::LruCache::new(max_size))),
            max_size,
        }
    }
    
    async fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().await;
        let now = std::time::Instant::now();
        
        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter_map(|(key, (_, expiry))| {
                if let Some(exp_time) = expiry {
                    if now > *exp_time {
                        Some(key.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
            
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }
}

// Cache service with multiple backends
pub struct CacheService {
    primary: Box<dyn CacheProvider>,
    secondary: Option<Box<dyn CacheProvider>>,
}

impl CacheService {
    pub fn new(primary: Box<dyn CacheProvider>, secondary: Option<Box<dyn CacheProvider>>) -> Self {
        Self { primary, secondary }
    }
    
    pub async fn get_or_compute<T, F, Fut>(&self, key: &str, compute: F) -> Result<T, CacheError>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
    {
        // Try primary cache first
        if let Some(cached) = self.primary.get(key).await? {
            return Ok(cached);
        }
        
        // Try secondary cache
        if let Some(secondary) = &self.secondary {
            if let Some(cached) = secondary.get(key).await? {
                // Promote to primary cache
                self.primary.set(key, &cached, None).await?;
                return Ok(cached);
            }
        }
        
        // Compute value
        let value = compute().await?;
        
        // Store in both caches
        let store_primary = self.primary.set(key, &value, None);
        let store_secondary = if let Some(secondary) = &self.secondary {
            Some(secondary.set(key, &value, None))
        } else {
            None
        };
        
        // Execute storage operations concurrently
        if let Some(secondary_future) = store_secondary {
            futures::try_join!(store_primary, secondary_future)?;
        } else {
            store_primary.await?;
        }
        
        Ok(value)
    }
}
```

### 2. Connection Management

```rust
// src/core/connections.rs
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::time::Duration;

pub struct ConnectionManager {
    max_connections: usize,
    semaphore: Arc<Semaphore>,
    active_connections: Arc<AtomicUsize>,
    connection_timeout: Duration,
}

impl ConnectionManager {
    pub fn new(max_connections: usize, connection_timeout: Duration) -> Self {
        Self {
            max_connections,
            semaphore: Arc::new(Semaphore::new(max_connections)),
            active_connections: Arc::new(AtomicUsize::new(0)),
            connection_timeout,
        }
    }
    
    pub async fn acquire_connection(&self) -> Result<ConnectionGuard, ConnectionError> {
        let permit = tokio::time::timeout(
            self.connection_timeout,
            self.semaphore.acquire()
        ).await??;
        
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        
        Ok(ConnectionGuard {
            _permit: permit,
            active_connections: self.active_connections.clone(),
        })
    }
    
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }
    
    pub fn available_connections(&self) -> usize {
        self.semaphore.available_permits()
    }
}

pub struct ConnectionGuard {
    _permit: tokio::sync::SemaphorePermit<'static>,
    active_connections: Arc<AtomicUsize>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}
```

## Implementation Phases and Milestones

### Phase 1: Foundation (Weeks 1-4)
- [ ] Core application structure and dependency injection
- [ ] Configuration management system
- [ ] Basic HTTP server with Axum
- [ ] Error handling and logging
- [ ] Basic authentication (JWT)
- [ ] Database connection pooling

**Deliverables:**
- Basic HTTP API server
- Configuration-driven service loading
- JWT authentication working
- Database connectivity established

### Phase 2: Core Services (Weeks 5-8)
- [ ] Base service trait implementation
- [ ] SQL database service (MySQL, PostgreSQL, SQLite)
- [ ] Service registry and management
- [ ] Basic REST operations (CRUD)
- [ ] API documentation generation
- [ ] Request validation and response formatting

**Deliverables:**
- Database services functional
- REST API endpoints working
- Service registration system
- Basic admin endpoints

### Phase 3: Advanced Services (Weeks 9-12)
- [ ] File service implementation (local, S3, Azure)
- [ ] NoSQL database services (MongoDB, DynamoDB)
- [ ] Email service implementation
- [ ] Cache service integration
- [ ] Plugin architecture foundation
- [ ] Event system implementation

**Deliverables:**
- File storage services
- Email functionality
- NoSQL database support
- Plugin system working
- Event-driven architecture

### Phase 4: Performance & Scalability (Weeks 13-16)
- [ ] Advanced caching strategies
- [ ] Connection pooling optimization
- [ ] Batch operation support
- [ ] Streaming responses
- [ ] Rate limiting implementation
- [ ] Performance monitoring and metrics

**Deliverables:**
- High-performance caching
- Batch operations
- Rate limiting
- Performance monitoring
- Load testing results

### Phase 5: Enterprise Features (Weeks 17-20)
- [ ] Advanced plugin system with dynamic loading
- [ ] Multi-tenancy support
- [ ] Advanced security features
- [ ] Audit logging
- [ ] Backup and recovery
- [ ] Deployment automation

**Deliverables:**
- Dynamic plugin loading
- Multi-tenant architecture
- Security audit compliance
- Automated deployment
- Production-ready system

### Phase 6: Migration & Testing (Weeks 21-24)
- [ ] Migration tools from PHP to Rust
- [ ] Comprehensive testing suite
- [ ] Performance benchmarking
- [ ] Documentation completion
- [ ] Production deployment
- [ ] Monitoring and alerting

**Deliverables:**
- Migration tooling
- Complete test coverage
- Performance benchmarks
- Production deployment
- Monitoring dashboard

## Key Performance Metrics

### Target Performance Goals
- **Latency**: < 10ms for simple database queries
- **Throughput**: > 10,000 requests per second
- **Memory Usage**: < 100MB base memory footprint
- **Startup Time**: < 5 seconds cold start
- **Connection Efficiency**: > 95% connection pool utilization

### Monitoring and Observability
- Prometheus metrics integration
- Structured logging with tracing
- Health check endpoints
- Performance profiling tools
- Custom dashboards for key metrics

## Risk Assessment and Mitigation

### Technical Risks
1. **Async Complexity**: Mitigated by comprehensive testing and clear async patterns
2. **Plugin System Stability**: Sandboxed execution and error isolation
3. **Database Compatibility**: Extensive testing across different database systems
4. **Performance Regressions**: Continuous benchmarking and performance monitoring

### Migration Risks
1. **API Compatibility**: Comprehensive compatibility testing suite
2. **Data Migration**: Robust migration tools with rollback capabilities
3. **Feature Parity**: Detailed feature mapping and implementation tracking
4. **Operational Complexity**: Comprehensive documentation and training

## Conclusion

This Rust architecture provides a solid foundation for migrating DreamFactory to a modern, high-performance platform. The design emphasizes:

- **Type Safety**: Leveraging Rust's type system for compile-time guarantees
- **Performance**: Async-first design with efficient resource usage
- **Extensibility**: Plugin architecture for custom functionality
- **Maintainability**: Clear separation of concerns and modular design
- **Scalability**: Built-in support for horizontal scaling and load balancing

The phased implementation approach allows for incremental migration while maintaining system stability and adding new capabilities. The architecture is designed to exceed current performance benchmarks while providing a foundation for future enhancements and scaling requirements.