use thiserror::Error;

/// Database operation errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {message}")]
    Connection { message: String },

    #[error("Query error: {message}")]
    Query { message: String },

    #[error("Schema error: {message}")]
    Schema { message: String },

    #[error("Table not found: {table}")]
    TableNotFound { table: String },

    #[error("Column not found: {column} in table {table}")]
    ColumnNotFound { table: String, column: String },

    #[error("Record not found with ID: {id}")]
    RecordNotFound { id: String },

    #[error("Constraint violation: {message}")]
    ConstraintViolation { message: String },

    #[error("Transaction error: {message}")]
    Transaction { message: String },

    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Relationship error: {message}")]
    Relationship { message: String },

    #[error("Computed field error: {message}")]
    ComputedField { message: String },

    #[error("Batch operation error: {message}")]
    BatchOperation { message: String },

    #[error("Unsupported operation: {operation} for provider {provider}")]
    UnsupportedOperation { operation: String, provider: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(db_err) => {
                if db_err.message().contains("not found") || db_err.message().contains("doesn't exist") {
                    DatabaseError::TableNotFound {
                        table: "unknown".to_string(),
                    }
                } else if db_err.message().contains("constraint") {
                    DatabaseError::ConstraintViolation {
                        message: db_err.message().to_string(),
                    }
                } else {
                    DatabaseError::Query {
                        message: db_err.message().to_string(),
                    }
                }
            }
            sqlx::Error::RowNotFound => DatabaseError::RecordNotFound {
                id: "unknown".to_string(),
            },
            sqlx::Error::PoolTimedOut => DatabaseError::Connection {
                message: "Connection pool timed out".to_string(),
            },
            _ => DatabaseError::Internal {
                message: err.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        DatabaseError::Serialization {
            message: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for DatabaseError {
    fn from(err: anyhow::Error) -> Self {
        DatabaseError::Internal {
            message: err.to_string(),
        }
    }
}

/// Database operation result type
pub type DatabaseResult<T> = Result<T, DatabaseError>;