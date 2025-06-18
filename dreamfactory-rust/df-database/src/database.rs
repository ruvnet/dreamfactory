use crate::{
    DatabaseError, DatabaseProvider, DatabaseResult, DatabaseService, QueryParams,
    BatchRequest, BatchResponse, BatchResult, TableSchema, Relationship,
    SchemaIntrospector, QueryProcessor, SqlExecutor, RelationshipResolver,
    ComputedFieldEvaluator, ConnectionConfig, PoolConfig, ProviderCapabilities,
    OperationStats,
};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{AnyPool, Pool, Any};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Main database service implementation
pub struct DatabaseServiceImpl {
    provider: DatabaseProvider,
    pool: AnyPool,
    schema_introspector: SchemaIntrospector,
    relationship_resolver: Arc<RwLock<Option<RelationshipResolver>>>,
    computed_field_evaluator: Arc<RwLock<ComputedFieldEvaluator>>,
    capabilities: ProviderCapabilities,
    stats: Arc<RwLock<OperationStats>>,
}

impl DatabaseServiceImpl {
    /// Execute raw SQL (for testing and migrations)
    pub async fn execute_raw_sql(&self, sql: &str) -> Result<u64, DatabaseError> {
        SqlExecutor::execute_modify(&self.pool, sql, &[]).await
    }

    /// Get the underlying pool (for testing)
    #[cfg(test)]
    pub fn pool(&self) -> &sqlx::AnyPool {
        &self.pool
    }

    /// Create a new database service instance
    pub async fn new(config: ConnectionConfig, pool_config: PoolConfig) -> Result<Self, DatabaseError> {
        let connection_string = config.build_connection_string();
        
        // Build pool options
        let pool_options = sqlx::any::AnyPoolOptions::new()
            .max_connections(pool_config.max_connections)
            .min_connections(pool_config.min_connections)
            .acquire_timeout(Duration::from_secs(pool_config.acquire_timeout))
            .idle_timeout(Some(Duration::from_secs(pool_config.idle_timeout)))
            .max_lifetime(Some(Duration::from_secs(pool_config.max_lifetime)));

        let pool = pool_options
            .connect(&connection_string)
            .await
            .map_err(|e| DatabaseError::Connection {
                message: format!("Failed to connect to database: {}", e),
            })?;

        let schema_introspector = SchemaIntrospector::new(config.provider.clone());
        let computed_field_evaluator = Arc::new(RwLock::new(ComputedFieldEvaluator::new()));
        
        let capabilities = match config.provider {
            DatabaseProvider::MySQL => ProviderCapabilities::for_mysql(),
            DatabaseProvider::PostgreSQL => ProviderCapabilities::for_postgresql(),
            DatabaseProvider::SQLite => ProviderCapabilities::for_sqlite(),
        };

        let stats = Arc::new(RwLock::new(OperationStats::default()));

        Ok(Self {
            provider: config.provider,
            pool,
            schema_introspector,
            relationship_resolver: Arc::new(RwLock::new(None)),
            computed_field_evaluator,
            capabilities,
            stats,
        })
    }

    /// Initialize relationship resolver (requires self-reference)
    pub async fn initialize_relationships(&self) {
        let resolver = RelationshipResolver::new(Box::new(self.clone()));
        let mut resolver_lock = self.relationship_resolver.write().await;
        *resolver_lock = Some(resolver);
    }

    /// Get provider capabilities
    pub fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    /// Get operation statistics
    pub async fn stats(&self) -> OperationStats {
        self.stats.read().await.clone()
    }

    /// Update operation statistics
    async fn update_stats<F>(&self, operation: F) where F: FnOnce(&mut OperationStats) {
        let mut stats = self.stats.write().await;
        operation(&mut *stats);
    }

    /// Execute query with timing and stats tracking
    async fn execute_with_stats<T, F, Fut>(&self, operation: F) -> Result<T, DatabaseError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, DatabaseError>>,
    {
        let start_time = Instant::now();
        let result = operation().await;
        let duration = start_time.elapsed();

        self.update_stats(|stats| {
            stats.total_queries += 1;
            if result.is_ok() {
                stats.successful_queries += 1;
            } else {
                stats.failed_queries += 1;
            }
            
            // Update average query time
            let total_time = stats.average_query_time * (stats.total_queries - 1) as f64;
            stats.average_query_time = (total_time + duration.as_secs_f64()) / stats.total_queries as f64;
        }).await;

        result
    }
}

impl Clone for DatabaseServiceImpl {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            pool: self.pool.clone(),
            schema_introspector: SchemaIntrospector::new(self.provider.clone()),
            relationship_resolver: self.relationship_resolver.clone(),
            computed_field_evaluator: self.computed_field_evaluator.clone(),
            capabilities: self.capabilities.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[async_trait]
impl DatabaseService for DatabaseServiceImpl {
    fn provider(&self) -> DatabaseProvider {
        self.provider.clone()
    }

    async fn test_connection(&self) -> Result<(), DatabaseError> {
        self.execute_with_stats(|| async {
            match self.provider {
                DatabaseProvider::MySQL => {
                    sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
                }
                DatabaseProvider::PostgreSQL => {
                    sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
                }
                DatabaseProvider::SQLite => {
                    sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
                }
            }
            Ok(())
        }).await
    }

    async fn get_schema(&self, table: Option<&str>) -> Result<Vec<TableSchema>, DatabaseError> {
        self.execute_with_stats(|| async {
            match table {
                Some(table_name) => {
                    let schema = self.schema_introspector.get_table_schema(&self.pool, table_name).await?;
                    Ok(vec![schema])
                }
                None => {
                    self.schema_introspector.get_all_schemas(&self.pool).await
                }
            }
        }).await
    }

    async fn describe_table(&self, table: &str) -> Result<TableSchema, DatabaseError> {
        self.execute_with_stats(|| async {
            self.schema_introspector.get_table_schema(&self.pool, table).await
        }).await
    }

    async fn get_records(
        &self,
        table: &str,
        params: &QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.execute_with_stats(|| async {
            // Build query
            let query_builder = QueryProcessor::process_params(table, params)?;
            let (query, query_params) = query_builder.build_select(&self.provider);

            // Execute query
            let mut records = SqlExecutor::execute_select(&self.pool, &query, &query_params).await?;

            // Update stats
            self.update_stats(|stats| {
                stats.total_records_read += records.len() as u64;
            }).await;

            // Execute relationships if requested
            if let Some(related) = &params.related {
                if let Some(resolver) = self.relationship_resolver.read().await.as_ref() {
                    resolver.execute_relationships(table, &mut records, related).await?;
                }
            }

            // Evaluate computed fields
            let evaluator = self.computed_field_evaluator.read().await;
            evaluator.evaluate_computed_fields(table, &mut records).await?;

            // Build result metadata
            let mut meta = None;
            if params.include_count.unwrap_or(false) {
                // Get total count for pagination
                let count_query = format!("SELECT COUNT(*) as count FROM {}", table);
                let count_result = SqlExecutor::execute_select(&self.pool, &count_query, &[]).await?;
                let count = count_result
                    .first()
                    .and_then(|r| r.get("count"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                meta = Some(crate::ResultMetadata {
                    count: Some(count),
                    schema: if params.include_schema.unwrap_or(false) {
                        Some(self.get_schema(Some(table)).await?)
                    } else {
                        None
                    },
                    next: None,
                });
            }

            Ok(crate::DatabaseResult {
                resource: records,
                meta,
            })
        }).await
    }

    async fn get_record(&self, table: &str, id: &str) -> Result<Value, DatabaseError> {
        self.execute_with_stats(|| async {
            let query_builder = crate::query::QueryBuilder::new(table)
                .where_condition("id = ?".to_string(), Some(Value::String(id.to_string())));
            
            let (query, params) = query_builder.build_select(&self.provider);
            let records = SqlExecutor::execute_select(&self.pool, &query, &params).await?;

            records.into_iter().next().ok_or_else(|| DatabaseError::RecordNotFound {
                id: id.to_string(),
            })
        }).await
    }

    async fn create_records(
        &self,
        table: &str,
        records: Vec<Value>,
        _params: &QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.execute_with_stats(|| async {
            let mut created_records = Vec::new();

            for record in records {
                let query_builder = crate::query::QueryBuilder::new(table);
                let (query, params) = query_builder.build_insert(&self.provider, &[record.clone()]);

                SqlExecutor::execute_modify(&self.pool, &query, &params).await?;

                // For databases that support RETURNING, we could get the created record
                // For now, we'll return the input record
                created_records.push(record);
            }

            // Update stats
            self.update_stats(|stats| {
                stats.total_records_written += created_records.len() as u64;
            }).await;

            Ok(crate::DatabaseResult {
                resource: created_records,
                meta: None,
            })
        }).await
    }

    async fn update_records(
        &self,
        table: &str,
        records: Vec<Value>,
        params: &QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.execute_with_stats(|| async {
            let mut updated_records = Vec::new();

            for record in records {
                let mut query_builder = crate::query::QueryBuilder::new(table);
                
                // Add WHERE conditions from params
                if let Some(filter) = &params.filter {
                    let (conditions, filter_params) = crate::query::FilterParser::parse_filter(filter)?;
                    for (condition, param) in conditions.into_iter().zip(filter_params.into_iter()) {
                        query_builder = query_builder.where_condition(condition, Some(param));
                    }
                }

                let (query, query_params) = query_builder.build_update(&self.provider, &record);
                SqlExecutor::execute_modify(&self.pool, &query, &query_params).await?;

                updated_records.push(record);
            }

            // Update stats
            self.update_stats(|stats| {
                stats.total_records_written += updated_records.len() as u64;
            }).await;

            Ok(crate::DatabaseResult {
                resource: updated_records,
                meta: None,
            })
        }).await
    }

    async fn update_record(
        &self,
        table: &str,
        id: &str,
        record: Value,
    ) -> Result<Value, DatabaseError> {
        self.execute_with_stats(|| async {
            let query_builder = crate::query::QueryBuilder::new(table)
                .where_condition("id = ?".to_string(), Some(Value::String(id.to_string())));

            let (query, mut params) = query_builder.build_update(&self.provider, &record);
            
            SqlExecutor::execute_modify(&self.pool, &query, &params).await?;

            // Update stats
            self.update_stats(|stats| {
                stats.total_records_written += 1;
            }).await;

            Ok(record)
        }).await
    }

    async fn delete_records(
        &self,
        table: &str,
        params: &QueryParams,
    ) -> Result<crate::DatabaseResult, DatabaseError> {
        self.execute_with_stats(|| async {
            // First, get the records that will be deleted
            let to_delete = self.get_records(table, params).await?;

            // Build delete query
            let mut query_builder = crate::query::QueryBuilder::new(table);
            
            if let Some(filter) = &params.filter {
                let (conditions, filter_params) = crate::query::FilterParser::parse_filter(filter)?;
                for (condition, param) in conditions.into_iter().zip(filter_params.into_iter()) {
                    query_builder = query_builder.where_condition(condition, Some(param));
                }
            }

            let (query, query_params) = query_builder.build_delete(&self.provider);
            let deleted_count = SqlExecutor::execute_modify(&self.pool, &query, &query_params).await?;

            Ok(crate::DatabaseResult {
                resource: to_delete.resource,
                meta: Some(crate::ResultMetadata {
                    count: Some(deleted_count),
                    schema: None,
                    next: None,
                }),
            })
        }).await
    }

    async fn delete_record(&self, table: &str, id: &str) -> Result<Value, DatabaseError> {
        self.execute_with_stats(|| async {
            // First get the record
            let record = self.get_record(table, id).await?;

            // Delete it
            let query_builder = crate::query::QueryBuilder::new(table)
                .where_condition("id = ?".to_string(), Some(Value::String(id.to_string())));

            let (query, params) = query_builder.build_delete(&self.provider);
            SqlExecutor::execute_modify(&self.pool, &query, &params).await?;

            Ok(record)
        }).await
    }

    async fn batch_operations(
        &self,
        table: &str,
        batch: BatchRequest,
    ) -> Result<BatchResponse, DatabaseError> {
        self.execute_with_stats(|| async {
            let mut results = Vec::new();
            let use_transaction = batch.rollback.unwrap_or(true);
            let continue_on_error = batch.continue_on_error.unwrap_or(false);

            if use_transaction && self.capabilities.supports_transactions {
                // Use transaction
                let mut tx = self.pool.begin().await.map_err(|e| DatabaseError::Transaction {
                    message: format!("Failed to begin transaction: {}", e),
                })?;

                for resource in batch.resources {
                    let result = self.process_batch_resource(table, resource, &mut tx).await;
                    
                    match result {
                        Ok(content) => {
                            results.push(BatchResult {
                                status_code: 200,
                                content: Some(content),
                                error: None,
                            });
                        }
                        Err(e) => {
                            results.push(BatchResult {
                                status_code: 400,
                                content: None,
                                error: Some(e.to_string()),
                            });

                            if !continue_on_error {
                                tx.rollback().await.map_err(|e| DatabaseError::Transaction {
                                    message: format!("Failed to rollback transaction: {}", e),
                                })?;
                                
                                return Ok(BatchResponse {
                                    resources: results,
                                    transaction_id: None,
                                });
                            }
                        }
                    }
                }

                tx.commit().await.map_err(|e| DatabaseError::Transaction {
                    message: format!("Failed to commit transaction: {}", e),
                })?;
            } else {
                // No transaction
                for resource in batch.resources {
                    let result = self.process_batch_resource_no_tx(table, resource).await;
                    
                    match result {
                        Ok(content) => {
                            results.push(BatchResult {
                                status_code: 200,
                                content: Some(content),
                                error: None,
                            });
                        }
                        Err(e) => {
                            results.push(BatchResult {
                                status_code: 400,
                                content: None,
                                error: Some(e.to_string()),
                            });

                            if !continue_on_error {
                                break;
                            }
                        }
                    }
                }
            }

            Ok(BatchResponse {
                resources: results,
                transaction_id: None,
            })
        }).await
    }

    async fn get_relationships(&self, table: &str) -> Result<Vec<Relationship>, DatabaseError> {
        // This would typically be implemented by reading from a configuration
        // or by introspecting foreign key relationships from the database schema
        // For now, return an empty vec
        Ok(Vec::new())
    }

    async fn execute_virtual_relationships(
        &self,
        table: &str,
        records: &mut Vec<Value>,
        relationships: &[String],
    ) -> Result<(), DatabaseError> {
        if let Some(resolver) = self.relationship_resolver.read().await.as_ref() {
            resolver.execute_relationships(table, records, relationships).await
        } else {
            Ok(())
        }
    }

    async fn evaluate_computed_fields(
        &self,
        table: &str,
        records: &mut Vec<Value>,
    ) -> Result<(), DatabaseError> {
        let evaluator = self.computed_field_evaluator.read().await;
        evaluator.evaluate_computed_fields(table, records).await
    }
}

impl DatabaseServiceImpl {
    /// Process batch resource with transaction
    async fn process_batch_resource(
        &self,
        table: &str,
        resource: Value,
        tx: &mut sqlx::Transaction<'_, Any>,
    ) -> Result<Value, DatabaseError> {
        // This is a simplified implementation
        // In a real implementation, you'd need to determine the operation type
        // and execute the appropriate query within the transaction
        
        // For now, assume it's an insert operation
        if let Some(obj) = resource.as_object() {
            let columns: Vec<String> = obj.keys().cloned().collect();
            let placeholders = match self.provider {
                DatabaseProvider::MySQL => 
                    (0..columns.len()).map(|_| "?").collect::<Vec<_>>().join(", "),
                DatabaseProvider::PostgreSQL => 
                    (1..=columns.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", "),
                DatabaseProvider::SQLite => 
                    (0..columns.len()).map(|_| "?").collect::<Vec<_>>().join(", "),
            };

            let query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                columns.join(", "),
                placeholders
            );

            let mut query_builder = sqlx::query(&query);
            for column in &columns {
                if let Some(value) = obj.get(column) {
                    query_builder = match value {
                        Value::String(s) => query_builder.bind(s),
                        Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                query_builder.bind(i)
                            } else if let Some(f) = n.as_f64() {
                                query_builder.bind(f)
                            } else {
                                query_builder.bind(n.to_string())
                            }
                        }
                        Value::Bool(b) => query_builder.bind(b),
                        Value::Null => query_builder.bind(Option::<String>::None),
                        _ => query_builder.bind(value.to_string()),
                    };
                }
            }

            query_builder.execute(&mut **tx).await?;
        }

        Ok(resource)
    }

    /// Process batch resource without transaction
    async fn process_batch_resource_no_tx(
        &self,
        table: &str,
        resource: Value,
    ) -> Result<Value, DatabaseError> {
        // Similar to above but using the pool directly
        let query_builder = crate::query::QueryBuilder::new(table);
        let (query, params) = query_builder.build_insert(&self.provider, &[resource.clone()]);
        SqlExecutor::execute_modify(&self.pool, &query, &params).await?;
        Ok(resource)
    }
}