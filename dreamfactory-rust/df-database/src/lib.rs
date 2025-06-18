pub mod database;
pub mod error;
pub mod providers;
pub mod schema;
pub mod query;
pub mod types;
pub mod relationships;
pub mod computed_fields;

pub use database::*;
pub use error::*;
pub use providers::*;
pub use schema::*;
pub use query::*;
pub use types::*;
pub use relationships::*;
pub use computed_fields::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Database provider types supported by DreamFactory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseProvider {
    #[serde(rename = "mysql")]
    MySQL,
    #[serde(rename = "pgsql")]
    PostgreSQL,
    #[serde(rename = "sqlite")]
    SQLite,
}

impl std::fmt::Display for DatabaseProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseProvider::MySQL => write!(f, "mysql"),
            DatabaseProvider::PostgreSQL => write!(f, "pgsql"),
            DatabaseProvider::SQLite => write!(f, "sqlite"),
        }
    }
}

/// Configuration for database connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub provider: DatabaseProvider,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub connection_string: Option<String>,
    pub options: HashMap<String, Value>,
}

/// Database connection pool wrapper
#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub provider: DatabaseProvider,
    pub pool: sqlx::AnyPool,
}

/// Query parameters for database operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryParams {
    pub filter: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub order: Option<String>,
    pub group: Option<String>,
    pub fields: Option<Vec<String>>,
    pub related: Option<Vec<String>>,
    pub include_count: Option<bool>,
    pub include_schema: Option<bool>,
}

/// Batch operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub resources: Vec<Value>,
    pub rollback: Option<bool>,
    pub continue_on_error: Option<bool>,
}

/// Batch operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    pub resources: Vec<BatchResult>,
    pub transaction_id: Option<String>,
}

/// Individual batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub status_code: u16,
    pub content: Option<Value>,
    pub error: Option<String>,
}

/// Database operation result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseResult {
    pub resource: Vec<Value>,
    pub meta: Option<ResultMetadata>,
}

/// Result metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMetadata {
    pub count: Option<u64>,
    pub schema: Option<Vec<TableSchema>>,
    pub next: Option<String>,
}

/// Main database service trait that all providers must implement
#[async_trait]
pub trait DatabaseService: Send + Sync {
    /// Get database provider type
    fn provider(&self) -> DatabaseProvider;

    /// Test database connection
    async fn test_connection(&self) -> Result<(), DatabaseError>;

    /// Get table schema information
    async fn get_schema(&self, table: Option<&str>) -> Result<Vec<TableSchema>, DatabaseError>;

    /// Describe a specific table
    async fn describe_table(&self, table: &str) -> Result<TableSchema, DatabaseError>;

    /// Get all records from a table
    async fn get_records(
        &self,
        table: &str,
        params: &QueryParams,
    ) -> Result<DatabaseResult, DatabaseError>;

    /// Get a single record by ID
    async fn get_record(&self, table: &str, id: &str) -> Result<Value, DatabaseError>;

    /// Create new records
    async fn create_records(
        &self,
        table: &str,
        records: Vec<Value>,
        params: &QueryParams,
    ) -> Result<DatabaseResult, DatabaseError>;

    /// Update records
    async fn update_records(
        &self,
        table: &str,
        records: Vec<Value>,
        params: &QueryParams,
    ) -> Result<DatabaseResult, DatabaseError>;

    /// Update a single record by ID
    async fn update_record(
        &self,
        table: &str,
        id: &str,
        record: Value,
    ) -> Result<Value, DatabaseError>;

    /// Delete records
    async fn delete_records(
        &self,
        table: &str,
        params: &QueryParams,
    ) -> Result<DatabaseResult, DatabaseError>;

    /// Delete a single record by ID
    async fn delete_record(&self, table: &str, id: &str) -> Result<Value, DatabaseError>;

    /// Execute batch operations
    async fn batch_operations(
        &self,
        table: &str,
        batch: BatchRequest,
    ) -> Result<BatchResponse, DatabaseError>;

    /// Get table relationships
    async fn get_relationships(&self, table: &str) -> Result<Vec<Relationship>, DatabaseError>;

    /// Execute virtual relationships
    async fn execute_virtual_relationships(
        &self,
        table: &str,
        records: &mut Vec<Value>,
        relationships: &[String],
    ) -> Result<(), DatabaseError>;

    /// Evaluate computed fields
    async fn evaluate_computed_fields(
        &self,
        table: &str,
        records: &mut Vec<Value>,
    ) -> Result<(), DatabaseError>;
}