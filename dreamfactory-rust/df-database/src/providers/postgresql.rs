use crate::{
    DatabaseError, DatabaseProvider, DatabaseService, ConnectionConfig, PoolConfig,
    database::DatabaseServiceImpl,
};
use async_trait::async_trait;

/// PostgreSQL-specific database provider
pub struct PostgreSqlProvider {
    inner: DatabaseServiceImpl,
}

impl PostgreSqlProvider {
    /// Create a new PostgreSQL provider instance
    pub async fn new(config: ConnectionConfig, pool_config: PoolConfig) -> Result<Self, DatabaseError> {
        // Validate that we're using PostgreSQL
        if config.provider != DatabaseProvider::PostgreSQL {
            return Err(DatabaseError::Configuration {
                message: "Provider must be PostgreSQL".to_string(),
            });
        }

        let inner = DatabaseServiceImpl::new(config, pool_config).await?;
        inner.initialize_relationships().await;

        Ok(Self { inner })
    }

    /// Create PostgreSQL provider with connection string
    pub async fn from_connection_string(
        connection_string: &str,
        pool_config: Option<PoolConfig>,
    ) -> Result<Self, DatabaseError> {
        let config = Self::parse_postgresql_connection_string(connection_string)?;
        let pool_config = pool_config.unwrap_or_default();
        Self::new(config, pool_config).await
    }

    /// Parse PostgreSQL connection string
    fn parse_postgresql_connection_string(connection_string: &str) -> Result<ConnectionConfig, DatabaseError> {
        if !connection_string.starts_with("postgresql://") && !connection_string.starts_with("postgres://") {
            return Err(DatabaseError::Configuration {
                message: "PostgreSQL connection string must start with postgresql:// or postgres://".to_string(),
            });
        }

        let url = url::Url::parse(connection_string).map_err(|e| DatabaseError::Configuration {
            message: format!("Invalid PostgreSQL connection string: {}", e),
        })?;

        Ok(ConnectionConfig {
            provider: DatabaseProvider::PostgreSQL,
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

    /// Get PostgreSQL-specific version information
    pub async fn get_version(&self) -> Result<String, DatabaseError> {
        let query = "SELECT version() as version";
        let result = crate::query::SqlExecutor::execute_select(
            self.inner.get_pool(),
            query,
            &[],
        ).await?;

        result
            .first()
            .and_then(|r| r.get("version"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| DatabaseError::Query {
                message: "Failed to get PostgreSQL version".to_string(),
            })
    }

    /// Get PostgreSQL-specific database size
    pub async fn get_database_size(&self, database: &str) -> Result<i64, DatabaseError> {
        let query = "SELECT pg_database_size($1) as size";
        let result = crate::query::SqlExecutor::execute_select(
            self.inner.get_pool(),
            query,
            &[serde_json::Value::String(database.to_string())],
        ).await?;

        result
            .first()
            .and_then(|r| r.get("size"))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| DatabaseError::Query {
                message: format!("Failed to get size for database {}", database),
            })
    }

    /// Get PostgreSQL-specific table size
    pub async fn get_table_size(&self, table: &str) -> Result<i64, DatabaseError> {
        let query = "SELECT pg_total_relation_size($1) as size";
        let result = crate::query::SqlExecutor::execute_select(
            self.inner.get_pool(),
            query,
            &[serde_json::Value::String(table.to_string())],
        ).await?;

        result
            .first()
            .and_then(|r| r.get("size"))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| DatabaseError::Query {
                message: format!("Failed to get size for table {}", table),
            })
    }

    /// Get PostgreSQL-specific index information
    pub async fn get_indexes(&self, table: &str) -> Result<Vec<crate::types::IndexInfo>, DatabaseError> {
        let query = r#"
            SELECT 
                i.relname as index_name,
                a.attname as column_name,
                ix.indisunique as is_unique,
                ix.indisprimary as is_primary
            FROM 
                pg_class t,
                pg_class i,
                pg_index ix,
                pg_attribute a
            WHERE 
                t.oid = ix.indrelid
                AND i.oid = ix.indexrelid
                AND a.attrelid = t.oid
                AND a.attnum = ANY(ix.indkey)
                AND t.relkind = 'r'
                AND t.relname = $1
            ORDER BY i.relname, a.attnum
        "#;

        let result = crate::query::SqlExecutor::execute_select(
            self.inner.get_pool(),
            query,
            &[serde_json::Value::String(table.to_string())],
        ).await?;

        let mut indexes = std::collections::HashMap::new();
        
        for row in result {
            let index_name = row.get("index_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
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

    /// Get PostgreSQL-specific foreign key information
    pub async fn get_foreign_keys(&self, table: &str) -> Result<Vec<crate::types::ForeignKeyInfo>, DatabaseError> {
        let query = r#"
            SELECT 
                tc.constraint_name as name,
                tc.table_name,
                kcu.column_name,
                ccu.table_name AS ref_table,
                ccu.column_name AS ref_column,
                rc.update_rule as on_update,
                rc.delete_rule as on_delete
            FROM 
                information_schema.table_constraints AS tc 
                JOIN information_schema.key_column_usage AS kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage AS ccu
                    ON ccu.constraint_name = tc.constraint_name
                    AND ccu.table_schema = tc.table_schema
                JOIN information_schema.referential_constraints AS rc
                    ON tc.constraint_name = rc.constraint_name
                    AND tc.table_schema = rc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY' 
            AND tc.table_name = $1
            ORDER BY tc.constraint_name, kcu.ordinal_position
        "#;

        let result = crate::query::SqlExecutor::execute_select(
            self.inner.get_pool(),
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

    /// Get PostgreSQL-specific schema information
    pub async fn get_schemas(&self) -> Result<Vec<String>, DatabaseError> {
        let query = r#"
            SELECT schema_name 
            FROM information_schema.schemata 
            WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
            ORDER BY schema_name
        "#;

        let result = crate::query::SqlExecutor::execute_select(self.inner.get_pool(), query, &[]).await?;
        
        Ok(result
            .into_iter()
            .filter_map(|row| row.get("schema_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect())
    }

    /// Get PostgreSQL-specific extension information
    pub async fn get_extensions(&self) -> Result<Vec<serde_json::Value>, DatabaseError> {
        let query = r#"
            SELECT 
                extname as name,
                extversion as version,
                nspname as schema
            FROM pg_extension e
            JOIN pg_namespace n ON e.extnamespace = n.oid
            ORDER BY extname
        "#;

        crate::query::SqlExecutor::execute_select(self.inner.get_pool(), query, &[]).await
    }

    /// Execute PostgreSQL-specific ANALYZE on a table
    pub async fn analyze_table(&self, table: &str) -> Result<(), DatabaseError> {
        let query = format!("ANALYZE {}", table);
        crate::query::SqlExecutor::execute_modify(self.inner.get_pool(), &query, &[]).await?;
        Ok(())
    }

    /// Execute PostgreSQL-specific VACUUM on a table
    pub async fn vacuum_table(&self, table: &str, full: bool) -> Result<(), DatabaseError> {
        let query = if full {
            format!("VACUUM FULL {}", table)
        } else {
            format!("VACUUM {}", table)
        };
        crate::query::SqlExecutor::execute_modify(self.inner.get_pool(), &query, &[]).await?;
        Ok(())
    }

    /// Get PostgreSQL-specific table statistics
    pub async fn get_table_stats(&self, table: &str) -> Result<serde_json::Value, DatabaseError> {
        let query = r#"
            SELECT 
                schemaname,
                tablename,
                attname,
                n_distinct,
                correlation,
                most_common_vals,
                most_common_freqs,
                histogram_bounds
            FROM pg_stats 
            WHERE tablename = $1
        "#;

        let result = crate::query::SqlExecutor::execute_select(
            self.inner.get_pool(),
            query,
            &[serde_json::Value::String(table.to_string())],
        ).await?;

        Ok(serde_json::Value::Array(result))
    }

    /// Get PostgreSQL-specific connection information
    pub async fn get_connection_info(&self) -> Result<serde_json::Value, DatabaseError> {
        let query = r#"
            SELECT 
                pid,
                usename,
                application_name,
                client_addr,
                client_port,
                backend_start,
                state,
                query
            FROM pg_stat_activity 
            WHERE pid = pg_backend_pid()
        "#;

        let result = crate::query::SqlExecutor::execute_select(self.inner.get_pool(), query, &[]).await?;
        
        result.into_iter().next().ok_or_else(|| DatabaseError::Query {
            message: "Failed to get connection information".to_string(),
        })
    }
}

// Delegate all DatabaseService methods to the inner implementation
#[async_trait]
impl DatabaseService for PostgreSqlProvider {
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