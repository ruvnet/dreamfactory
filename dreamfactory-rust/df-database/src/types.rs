use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database field types supported by DreamFactory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    #[serde(rename = "id")]
    Id,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "big_integer")]
    BigInteger,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "double")]
    Double,
    #[serde(rename = "decimal")]
    Decimal,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "date")]
    Date,
    #[serde(rename = "time")]
    Time,
    #[serde(rename = "datetime")]
    DateTime,
    #[serde(rename = "timestamp")]
    Timestamp,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "reference")]
    Reference,
    #[serde(rename = "user_id")]
    UserId,
    #[serde(rename = "user_id_on_create")]
    UserIdOnCreate,
    #[serde(rename = "user_id_on_update")]
    UserIdOnUpdate,
    #[serde(rename = "timestamp_on_create")]
    TimestampOnCreate,
    #[serde(rename = "timestamp_on_update")]
    TimestampOnUpdate,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FieldType::Id => "id",
            FieldType::String => "string",
            FieldType::Text => "text",
            FieldType::Integer => "integer",
            FieldType::BigInteger => "big_integer",
            FieldType::Float => "float",
            FieldType::Double => "double",
            FieldType::Decimal => "decimal",
            FieldType::Boolean => "boolean",
            FieldType::Binary => "binary",
            FieldType::Date => "date",
            FieldType::Time => "time",
            FieldType::DateTime => "datetime",
            FieldType::Timestamp => "timestamp",
            FieldType::Json => "json",
            FieldType::Reference => "reference",
            FieldType::UserId => "user_id",
            FieldType::UserIdOnCreate => "user_id_on_create",
            FieldType::UserIdOnUpdate => "user_id_on_update",
            FieldType::TimestampOnCreate => "timestamp_on_create",
            FieldType::TimestampOnUpdate => "timestamp_on_update",
        };
        write!(f, "{}", s)
    }
}

/// Database constraint types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    #[serde(rename = "primary_key")]
    PrimaryKey,
    #[serde(rename = "foreign_key")]
    ForeignKey,
    #[serde(rename = "unique")]
    Unique,
    #[serde(rename = "index")]
    Index,
    #[serde(rename = "check")]
    Check,
}

/// Database index information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub primary: bool,
}

/// Foreign key constraint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    pub on_update: Option<String>,
    pub on_delete: Option<String>,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub provider: crate::DatabaseProvider,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub options: HashMap<String, serde_json::Value>,
}

impl ConnectionConfig {
    /// Build connection string for the database provider
    pub fn build_connection_string(&self) -> String {
        match self.provider {
            crate::DatabaseProvider::MySQL => {
                let host = self.host.as_deref().unwrap_or("localhost");
                let port = self.port.unwrap_or(3306);
                let username = self.username.as_deref().unwrap_or("root");
                let password = self.password.as_deref().unwrap_or("");
                
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    username, password, host, port, self.database
                )
            }
            crate::DatabaseProvider::PostgreSQL => {
                let host = self.host.as_deref().unwrap_or("localhost");
                let port = self.port.unwrap_or(5432);
                let username = self.username.as_deref().unwrap_or("postgres");
                let password = self.password.as_deref().unwrap_or("");
                
                format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    username, password, host, port, self.database
                )
            }
            crate::DatabaseProvider::SQLite => {
                format!("sqlite:{}", self.database)
            }
        }
    }
}

/// Data validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub value: Option<serde_json::Value>,
    pub message: Option<String>,
}

/// Common validation rule types
pub struct ValidationRules;

impl ValidationRules {
    pub fn required() -> ValidationRule {
        ValidationRule {
            rule_type: "required".to_string(),
            value: None,
            message: Some("This field is required".to_string()),
        }
    }

    pub fn min_length(length: u32) -> ValidationRule {
        ValidationRule {
            rule_type: "min_length".to_string(),
            value: Some(serde_json::Value::Number(serde_json::Number::from(length))),
            message: Some(format!("Minimum length is {}", length)),
        }
    }

    pub fn max_length(length: u32) -> ValidationRule {
        ValidationRule {
            rule_type: "max_length".to_string(),
            value: Some(serde_json::Value::Number(serde_json::Number::from(length))),
            message: Some(format!("Maximum length is {}", length)),
        }
    }

    pub fn email() -> ValidationRule {
        ValidationRule {
            rule_type: "email".to_string(),
            value: None,
            message: Some("Must be a valid email address".to_string()),
        }
    }

    pub fn regex(pattern: &str) -> ValidationRule {
        ValidationRule {
            rule_type: "regex".to_string(),
            value: Some(serde_json::Value::String(pattern.to_string())),
            message: Some("Must match the required pattern".to_string()),
        }
    }

    pub fn numeric() -> ValidationRule {
        ValidationRule {
            rule_type: "numeric".to_string(),
            value: None,
            message: Some("Must be a number".to_string()),
        }
    }

    pub fn min_value(value: f64) -> ValidationRule {
        ValidationRule {
            rule_type: "min".to_string(),
            value: Some(serde_json::Value::Number(
                serde_json::Number::from_f64(value).unwrap()
            )),
            message: Some(format!("Minimum value is {}", value)),
        }
    }

    pub fn max_value(value: f64) -> ValidationRule {
        ValidationRule {
            rule_type: "max".to_string(),
            value: Some(serde_json::Value::Number(
                serde_json::Number::from_f64(value).unwrap()
            )),
            message: Some(format!("Maximum value is {}", value)),
        }
    }
}

/// Database operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub total_records_read: u64,
    pub total_records_written: u64,
    pub average_query_time: f64,
    pub connection_pool_size: u32,
    pub active_connections: u32,
}

impl Default for OperationStats {
    fn default() -> Self {
        Self {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            total_records_read: 0,
            total_records_written: 0,
            average_query_time: 0.0,
            connection_pool_size: 0,
            active_connections: 0,
        }
    }
}

/// Database provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supports_transactions: bool,
    pub supports_foreign_keys: bool,
    pub supports_json: bool,
    pub supports_full_text_search: bool,
    pub supports_spatial: bool,
    pub supports_arrays: bool,
    pub supports_upsert: bool,
    pub supports_returning: bool,
    pub supports_bulk_insert: bool,
    pub max_connections: Option<u32>,
    pub max_identifier_length: Option<u32>,
    pub max_index_length: Option<u32>,
}

impl ProviderCapabilities {
    pub fn for_mysql() -> Self {
        Self {
            supports_transactions: true,
            supports_foreign_keys: true,
            supports_json: true,
            supports_full_text_search: true,
            supports_spatial: true,
            supports_arrays: false,
            supports_upsert: true, // INSERT ... ON DUPLICATE KEY UPDATE
            supports_returning: false,
            supports_bulk_insert: true,
            max_connections: Some(1000),
            max_identifier_length: Some(64),
            max_index_length: Some(767),
        }
    }

    pub fn for_postgresql() -> Self {
        Self {
            supports_transactions: true,
            supports_foreign_keys: true,
            supports_json: true,
            supports_full_text_search: true,
            supports_spatial: true,
            supports_arrays: true,
            supports_upsert: true, // INSERT ... ON CONFLICT
            supports_returning: true,
            supports_bulk_insert: true,
            max_connections: Some(100),
            max_identifier_length: Some(63),
            max_index_length: None,
        }
    }

    pub fn for_sqlite() -> Self {
        Self {
            supports_transactions: true,
            supports_foreign_keys: true,
            supports_json: true,
            supports_full_text_search: true,
            supports_spatial: false,
            supports_arrays: false,
            supports_upsert: true, // INSERT ... ON CONFLICT
            supports_returning: true,
            supports_bulk_insert: false,
            max_connections: Some(1),
            max_identifier_length: None,
            max_index_length: None,
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64, // seconds
    pub idle_timeout: u64,    // seconds
    pub max_lifetime: u64,    // seconds
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}