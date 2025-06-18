use crate::{
    DatabaseError, DatabaseProvider, DatabaseService, ConnectionConfig, PoolConfig,
    database::DatabaseServiceImpl,
};
use async_trait::async_trait;

/// SQLite-specific database provider
pub struct SqliteProvider {
    inner: DatabaseServiceImpl,
}

impl SqliteProvider {
    /// Execute raw SQL (for testing and setup)
    pub async fn execute_raw_sql(&self, sql: &str) -> Result<u64, DatabaseError> {
        self.inner.execute_raw_sql(sql).await
    }

    /// Get the underlying pool (for testing)
    #[cfg(test)]
    pub fn pool(&self) -> &sqlx::AnyPool {
        self.inner.pool()
    }

    /// Create a new SQLite provider instance
    pub async fn new(config: ConnectionConfig, pool_config: PoolConfig) -> Result<Self, DatabaseError> {
        // Validate that we're using SQLite
        if config.provider != DatabaseProvider::SQLite {
            return Err(DatabaseError::Configuration {
                message: "Provider must be SQLite".to_string(),
            });
        }

        let inner = DatabaseServiceImpl::new(config, pool_config).await?;
        inner.initialize_relationships().await;

        Ok(Self { inner })
    }

    /// Create SQLite provider with database file path
    pub async fn from_file_path(
        file_path: &str,
        pool_config: Option<PoolConfig>,
    ) -> Result<Self, DatabaseError> {
        let config = ConnectionConfig {
            provider: DatabaseProvider::SQLite,
            host: None,
            port: None,
            database: file_path.to_string(),
            username: None,
            password: None,
            options: std::collections::HashMap::new(),
        };
        
        let pool_config = pool_config.unwrap_or_default();
        Self::new(config, pool_config).await
    }

    /// Create in-memory SQLite database
    pub async fn in_memory(pool_config: Option<PoolConfig>) -> Result<Self, DatabaseError> {
        Self::from_file_path(":memory:", pool_config).await
    }

    /// Get SQLite-specific version information
    pub async fn get_version(&self) -> Result<String, DatabaseError> {
        let query = "SELECT sqlite_version() as version";
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
                message: "Failed to get SQLite version".to_string(),
            })
    }

    /// Get SQLite-specific database file size
    pub async fn get_database_size(&self) -> Result<i64, DatabaseError> {
        let query = "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()";
        let result = crate::query::SqlExecutor::execute_select(&self.inner.pool, query, &[]).await?;

        result
            .first()
            .and_then(|r| r.get("size"))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| DatabaseError::Query {
                message: "Failed to get database size".to_string(),
            })
    }

    /// Get SQLite-specific table information using PRAGMA
    pub async fn pragma_table_info(&self, table: &str) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = format!("PRAGMA table_info({})", table);
        crate::query::SqlExecutor::execute_select(&self.inner.pool, &query, &[]).await
    }

    /// Get SQLite-specific index information using PRAGMA
    pub async fn pragma_index_list(&self, table: &str) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = format!("PRAGMA index_list({})", table);
        crate::query::SqlExecutor::execute_select(&self.inner.pool, &query, &[]).await
    }

    /// Get SQLite-specific index details using PRAGMA
    pub async fn pragma_index_info(&self, index: &str) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = format!("PRAGMA index_info({})", index);
        crate::query::SqlExecutor::execute_select(&self.inner.pool, &query, &[]).await
    }

    /// Get SQLite-specific foreign key information using PRAGMA
    pub async fn pragma_foreign_key_list(&self, table: &str) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = format!("PRAGMA foreign_key_list({})", table);
        crate::query::SqlExecutor::execute_select(&self.inner.pool, &query, &[]).await
    }

    /// Get SQLite-specific index information in standard format
    pub async fn get_indexes(&self, table: &str) -> Result<Vec<crate::types::IndexInfo>, DatabaseError> {
        let index_list = self.pragma_index_list(table).await?;
        let mut indexes = Vec::new();

        for index_row in index_list {
            let index_name = index_row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let is_unique = index_row.get("unique").and_then(|v| v.as_bool()).unwrap_or(false);
            
            // Get index details
            let index_info_result = self.pragma_index_info(&index_name).await?;
            let mut columns = Vec::new();

            for info_row in index_info_result {
                if let Some(column_name) = info_row.get("name").and_then(|v| v.as_str()) {
                    columns.push(column_name.to_string());
                }
            }

            indexes.push(crate::types::IndexInfo {
                name: index_name,
                columns,
                unique: is_unique,
                primary: false, // SQLite doesn't distinguish primary key indexes in pragma_index_list
            });
        }

        Ok(indexes)
    }

    /// Get SQLite-specific foreign key information in standard format
    pub async fn get_foreign_keys(&self, table: &str) -> Result<Vec<crate::types::ForeignKeyInfo>, DatabaseError> {
        let fk_list = self.pragma_foreign_key_list(table).await?;
        let mut foreign_keys = std::collections::HashMap::new();

        for fk_row in fk_list {
            let id = fk_row.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let seq = fk_row.get("seq").and_then(|v| v.as_i64()).unwrap_or(0);
            let ref_table = fk_row.get("table").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let column = fk_row.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ref_column = fk_row.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let on_update = fk_row.get("on_update").and_then(|v| v.as_str()).map(|s| s.to_string());
            let on_delete = fk_row.get("on_delete").and_then(|v| v.as_str()).map(|s| s.to_string());

            let constraint_name = format!("fk_{}_{}", table, id);
            
            let fk_info = foreign_keys.entry(constraint_name.clone()).or_insert_with(|| crate::types::ForeignKeyInfo {
                name: constraint_name,
                table: table.to_string(),
                columns: Vec::new(),
                ref_table,
                ref_columns: Vec::new(),
                on_update,
                on_delete,
            });

            // Ensure we have enough space in the vectors
            while fk_info.columns.len() <= seq as usize {
                fk_info.columns.push(String::new());
                fk_info.ref_columns.push(String::new());
            }

            fk_info.columns[seq as usize] = column;
            fk_info.ref_columns[seq as usize] = ref_column;
        }

        // Clean up empty entries
        for fk_info in foreign_keys.values_mut() {
            fk_info.columns.retain(|c| !c.is_empty());
            fk_info.ref_columns.retain(|c| !c.is_empty());
        }

        Ok(foreign_keys.into_values().collect())
    }

    /// Execute SQLite-specific PRAGMA command
    pub async fn pragma_command(&self, pragma: &str) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = format!("PRAGMA {}", pragma);
        crate::query::SqlExecutor::execute_select(&self.inner.pool, &query, &[]).await
    }

    /// Get SQLite-specific database settings
    pub async fn get_database_settings(&self) -> Result<std::collections::HashMap<String, serde_json::Value>, DatabaseError> {
        let pragmas = vec![
            "cache_size",
            "foreign_keys",
            "journal_mode",
            "synchronous",
            "temp_store",
            "locking_mode",
            "page_size",
            "auto_vacuum",
            "encoding",
            "max_page_count",
            "secure_delete",
            "wal_autocheckpoint",
        ];

        let mut settings = std::collections::HashMap::new();

        for pragma in pragmas {
            match self.pragma_command(pragma).await {
                Ok(result) => {
                    if let Some(row) = result.first() {
                        if let Some(value) = row.values().next() {
                            settings.insert(pragma.to_string(), value.clone());
                        }
                    }
                }
                Err(_) => {
                    // Some pragmas might not be available in all SQLite versions
                    continue;
                }
            }
        }

        Ok(settings)
    }

    /// Execute SQLite-specific VACUUM command
    pub async fn vacuum(&self) -> Result<(), DatabaseError> {
        crate::query::SqlExecutor::execute_modify(&self.inner.pool, "VACUUM", &[]).await?;
        Ok(())
    }

    /// Execute SQLite-specific ANALYZE command
    pub async fn analyze(&self, table: Option<&str>) -> Result<(), DatabaseError> {
        let query = match table {
            Some(table_name) => format!("ANALYZE {}", table_name),
            None => "ANALYZE".to_string(),
        };
        crate::query::SqlExecutor::execute_modify(&self.inner.pool, &query, &[]).await?;
        Ok(())
    }

    /// Get SQLite-specific table list with additional information
    pub async fn get_table_list(&self) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = r#"
            SELECT 
                name,
                type,
                sql
            FROM sqlite_master 
            WHERE type IN ('table', 'view')
            AND name NOT LIKE 'sqlite_%'
            ORDER BY name
        "#;

        crate::query::SqlExecutor::execute_select(&self.inner.pool, query, &[]).await
    }

    /// Check if foreign key constraints are enabled
    pub async fn foreign_keys_enabled(&self) -> Result<bool, DatabaseError> {
        let result = self.pragma_command("foreign_keys").await?;
        
        Ok(result
            .first()
            .and_then(|row| row.values().next())
            .and_then(|v| v.as_i64())
            .map(|i| i != 0)
            .unwrap_or(false))
    }

    /// Enable or disable foreign key constraints
    pub async fn set_foreign_keys(&self, enabled: bool) -> Result<(), DatabaseError> {
        let query = format!("PRAGMA foreign_keys = {}", if enabled { 1 } else { 0 });
        crate::query::SqlExecutor::execute_modify(&self.inner.pool, &query, &[]).await?;
        Ok(())
    }

    /// Get SQLite compile options
    pub async fn get_compile_options(&self) -> Result<Vec<String>, DatabaseError> {
        let mut options = Vec::new();
        let mut id = 0;

        loop {
            let query = format!("SELECT sqlite_compileoption_get({}) as option", id);
            match crate::query::SqlExecutor::execute_select(&self.inner.pool, &query, &[]).await {
                Ok(result) => {
                    if let Some(row) = result.first() {
                        if let Some(option) = row.get("option").and_then(|v| v.as_str()) {
                            options.push(option.to_string());
                            id += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        Ok(options)
    }
}

// Delegate all DatabaseService methods to the inner implementation
#[async_trait]
impl DatabaseService for SqliteProvider {
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