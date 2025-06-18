use crate::{
    DatabaseError, DatabaseProvider, DatabaseService, ConnectionConfig, PoolConfig,
    database::DatabaseServiceImpl,
};
use async_trait::async_trait;

/// MySQL-specific database provider
pub struct MySqlProvider {
    inner: DatabaseServiceImpl,
}

impl MySqlProvider {
    /// Create a new MySQL provider instance
    pub async fn new(config: ConnectionConfig, pool_config: PoolConfig) -> Result<Self, DatabaseError> {
        // Validate that we're using MySQL
        if config.provider != DatabaseProvider::MySQL {
            return Err(DatabaseError::Configuration {
                message: "Provider must be MySQL".to_string(),
            });
        }

        let inner = DatabaseServiceImpl::new(config, pool_config).await?;
        inner.initialize_relationships().await;

        Ok(Self { inner })
    }

    /// Create MySQL provider with connection string
    pub async fn from_connection_string(
        connection_string: &str,
        pool_config: Option<PoolConfig>,
    ) -> Result<Self, DatabaseError> {
        let config = Self::parse_mysql_connection_string(connection_string)?;
        let pool_config = pool_config.unwrap_or_default();
        Self::new(config, pool_config).await
    }

    /// Parse MySQL connection string
    fn parse_mysql_connection_string(connection_string: &str) -> Result<ConnectionConfig, DatabaseError> {
        if !connection_string.starts_with("mysql://") {
            return Err(DatabaseError::Configuration {
                message: "MySQL connection string must start with mysql://".to_string(),
            });
        }

        let url = url::Url::parse(connection_string).map_err(|e| DatabaseError::Configuration {
            message: format!("Invalid MySQL connection string: {}", e),
        })?;

        Ok(ConnectionConfig {
            provider: DatabaseProvider::MySQL,
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
    }

    /// Get MySQL-specific version information
    pub async fn get_version(&self) -> Result<String, DatabaseError> {
        let query = "SELECT VERSION() as version";
        let result = crate::query::SqlExecutor::execute_select(
            &self.inner.pool,
            query,
            &[],
        ).await?;

        result
            .first()
            .and_then(|r| r.get("version"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| DatabaseError::Query {
                message: "Failed to get MySQL version".to_string(),
            })
    }

    /// Get MySQL-specific table engine information
    pub async fn get_table_engine(&self, table: &str) -> Result<String, DatabaseError> {
        let query = r#"
            SELECT ENGINE 
            FROM information_schema.TABLES 
            WHERE TABLE_SCHEMA = DATABASE() 
            AND TABLE_NAME = ?
        "#;

        let result = crate::query::SqlExecutor::execute_select(
            &self.inner.pool,
            query,
            &[serde_json::Value::String(table.to_string())],
        ).await?;

        result
            .first()
            .and_then(|r| r.get("ENGINE"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| DatabaseError::Query {
                message: format!("Failed to get engine for table {}", table),
            })
    }

    /// Get MySQL-specific index information
    pub async fn get_indexes(&self, table: &str) -> Result<Vec<crate::types::IndexInfo>, DatabaseError> {
        let query = r#"
            SELECT 
                INDEX_NAME as name,
                COLUMN_NAME as column_name,
                NON_UNIQUE = 0 as is_unique,
                INDEX_NAME = 'PRIMARY' as is_primary
            FROM information_schema.STATISTICS 
            WHERE TABLE_SCHEMA = DATABASE() 
            AND TABLE_NAME = ?
            ORDER BY INDEX_NAME, SEQ_IN_INDEX
        "#;

        let result = crate::query::SqlExecutor::execute_select(
            &self.inner.pool,
            query,
            &[serde_json::Value::String(table.to_string())],
        ).await?;

        let mut indexes = std::collections::HashMap::new();
        
        for row in result {
            let index_name = row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let column_name = row.get("column_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let is_unique = row.get("is_unique").and_then(|v| v.as_bool()).unwrap_or(false);
            let is_primary = row.get("is_primary").and_then(|v| v.as_bool()).unwrap_or(false);

            let index_info = indexes.entry(index_name.clone()).or_insert_with(|| crate::types::IndexInfo {
                name: index_name,
                columns: Vec::new(),
                unique: is_unique,
                primary: is_primary,
            });

            index_info.columns.push(column_name);
        }

        Ok(indexes.into_values().collect())
    }

    /// Get MySQL-specific foreign key information
    pub async fn get_foreign_keys(&self, table: &str) -> Result<Vec<crate::types::ForeignKeyInfo>, DatabaseError> {
        let query = r#"
            SELECT 
                CONSTRAINT_NAME as name,
                TABLE_NAME as table_name,
                COLUMN_NAME as column_name,
                REFERENCED_TABLE_NAME as ref_table,
                REFERENCED_COLUMN_NAME as ref_column,
                UPDATE_RULE as on_update,
                DELETE_RULE as on_delete
            FROM information_schema.KEY_COLUMN_USAGE 
            WHERE TABLE_SCHEMA = DATABASE() 
            AND TABLE_NAME = ?
            AND REFERENCED_TABLE_NAME IS NOT NULL
            ORDER BY CONSTRAINT_NAME, ORDINAL_POSITION
        "#;

        let result = crate::query::SqlExecutor::execute_select(
            &self.inner.pool,
            query,
            &[serde_json::Value::String(table.to_string())],
        ).await?;

        let mut foreign_keys = std::collections::HashMap::new();
        
        for row in result {
            let constraint_name = row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let table_name = row.get("table_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let column_name = row.get("column_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ref_table = row.get("ref_table").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ref_column = row.get("ref_column").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let on_update = row.get("on_update").and_then(|v| v.as_str()).map(|s| s.to_string());
            let on_delete = row.get("on_delete").and_then(|v| v.as_str()).map(|s| s.to_string());

            let fk_info = foreign_keys.entry(constraint_name.clone()).or_insert_with(|| crate::types::ForeignKeyInfo {
                name: constraint_name,
                table: table_name,
                columns: Vec::new(),
                ref_table,
                ref_columns: Vec::new(),
                on_update,
                on_delete,
            });

            fk_info.columns.push(column_name);
            fk_info.ref_columns.push(ref_column);
        }

        Ok(foreign_keys.into_values().collect())
    }

    /// Execute MySQL-specific SHOW command
    pub async fn show_command(&self, command: &str) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = format!("SHOW {}", command);
        crate::query::SqlExecutor::execute_select(&self.inner.pool, &query, &[]).await
    }

    /// Get MySQL-specific system variables
    pub async fn get_system_variables(&self) -> Result<std::collections::HashMap<String, String>, DatabaseError> {
        let result = self.show_command("VARIABLES").await?;
        let mut variables = std::collections::HashMap::new();

        for row in result {
            if let (Some(name), Some(value)) = (
                row.get("Variable_name").and_then(|v| v.as_str()),
                row.get("Value").and_then(|v| v.as_str()),
            ) {
                variables.insert(name.to_string(), value.to_string());
            }
        }

        Ok(variables)
    }
}

// Delegate all DatabaseService methods to the inner implementation
#[async_trait]
impl DatabaseService for MySqlProvider {
    fn provider(&self) -> DatabaseProvider {
        self.inner.provider()
    }

    async fn test_connection(&self) -> Result<(), DatabaseError> {
        self.inner.test_connection().await
    }

    async fn get_schema(&self, table: Option<&str>) -> Result<Vec<crate::TableSchema>, DatabaseError> {
        self.inner.get_schema(table).await
    }

    async fn describe_table(&self, table: &str) -> Result<crate::TableSchema, DatabaseError> {
        self.inner.describe_table(table).await
    }

    async fn get_records(
        &self,
        table: &str,
        params: &crate::QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.inner.get_records(table, params).await
    }

    async fn get_record(&self, table: &str, id: &str) -> Result<serde_json::Value, DatabaseError> {
        self.inner.get_record(table, id).await
    }

    async fn create_records(
        &self,
        table: &str,
        records: Vec<serde_json::Value>,
        params: &crate::QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.inner.create_records(table, records, params).await
    }

    async fn update_records(
        &self,
        table: &str,
        records: Vec<serde_json::Value>,
        params: &crate::QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.inner.update_records(table, records, params).await
    }

    async fn update_record(
        &self,
        table: &str,
        id: &str,
        record: serde_json::Value,
    ) -> Result<serde_json::Value, DatabaseError> {
        self.inner.update_record(table, id, record).await
    }

    async fn delete_records(
        &self,
        table: &str,
        params: &crate::QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.inner.delete_records(table, params).await
    }

    async fn delete_record(&self, table: &str, id: &str) -> Result<serde_json::Value, DatabaseError> {
        self.inner.delete_record(table, id).await
    }

    async fn batch_operations(
        &self,
        table: &str,
        batch: crate::BatchRequest,
    ) -> Result<crate::BatchResponse, DatabaseError> {
        self.inner.batch_operations(table, batch).await
    }

    async fn get_relationships(&self, table: &str) -> Result<Vec<crate::Relationship>, DatabaseError> {
        self.inner.get_relationships(table).await
    }

    async fn execute_virtual_relationships(
        &self,
        table: &str,
        records: &mut Vec<serde_json::Value>,
        relationships: &[String],
    ) -> Result<(), DatabaseError> {
        self.inner.execute_virtual_relationships(table, records, relationships).await
    }

    async fn evaluate_computed_fields(
        &self,
        table: &str,
        records: &mut Vec<serde_json::Value>,
    ) -> Result<(), DatabaseError> {
        self.inner.evaluate_computed_fields(table, records).await
    }
}