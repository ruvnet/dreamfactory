use crate::{DatabaseError, QueryParams};
use serde_json::Value;
use std::collections::HashMap;

/// Query builder for constructing database queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    select_fields: Vec<String>,
    where_conditions: Vec<String>,
    order_by: Vec<String>,
    group_by: Vec<String>,
    limit: Option<u64>,
    offset: Option<u64>,
    parameters: Vec<Value>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            select_fields: vec!["*".to_string()],
            where_conditions: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            limit: None,
            offset: None,
            parameters: Vec::new(),
        }
    }

    /// Set select fields
    pub fn select(mut self, fields: Vec<String>) -> Self {
        if !fields.is_empty() {
            self.select_fields = fields;
        }
        self
    }

    /// Add WHERE condition
    pub fn where_condition(mut self, condition: String, value: Option<Value>) -> Self {
        self.where_conditions.push(condition);
        if let Some(val) = value {
            self.parameters.push(val);
        }
        self
    }

    /// Set ORDER BY
    pub fn order_by(mut self, order: Vec<String>) -> Self {
        self.order_by = order;
        self
    }

    /// Set GROUP BY
    pub fn group_by(mut self, group: Vec<String>) -> Self {
        self.group_by = group;
        self
    }

    /// Set LIMIT
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set OFFSET
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build the SELECT query
    pub fn build_select(&self, provider: &crate::DatabaseProvider) -> (String, Vec<Value>) {
        let mut query = format!("SELECT {} FROM {}", 
            self.select_fields.join(", "), 
            self.quote_identifier(&self.table, provider)
        );

        if !self.where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_conditions.join(" AND ")));
        }

        if !self.group_by.is_empty() {
            query.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        if !self.order_by.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        if let Some(limit) = self.limit {
            match provider {
                crate::DatabaseProvider::MySQL | crate::DatabaseProvider::SQLite => {
                    query.push_str(&format!(" LIMIT {}", limit));
                }
                crate::DatabaseProvider::PostgreSQL => {
                    query.push_str(&format!(" LIMIT {}", limit));
                }
            }
        }

        if let Some(offset) = self.offset {
            match provider {
                crate::DatabaseProvider::MySQL | crate::DatabaseProvider::SQLite => {
                    query.push_str(&format!(" OFFSET {}", offset));
                }
                crate::DatabaseProvider::PostgreSQL => {
                    query.push_str(&format!(" OFFSET {}", offset));
                }
            }
        }

        (query, self.parameters.clone())
    }

    /// Build INSERT query
    pub fn build_insert(&self, provider: &crate::DatabaseProvider, values: &[Value]) -> (String, Vec<Value>) {
        if values.is_empty() {
            return ("".to_string(), Vec::new());
        }

        let first_record = &values[0];
        if let Some(obj) = first_record.as_object() {
            let columns: Vec<String> = obj.keys()
                .map(|k| self.quote_identifier(k, provider))
                .collect();
            
            let placeholders = match provider {
                crate::DatabaseProvider::MySQL => 
                    (0..columns.len()).map(|_| "?").collect::<Vec<_>>().join(", "),
                crate::DatabaseProvider::PostgreSQL => 
                    (1..=columns.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", "),
                crate::DatabaseProvider::SQLite => 
                    (0..columns.len()).map(|_| "?").collect::<Vec<_>>().join(", "),
            };

            let query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.quote_identifier(&self.table, provider),
                columns.join(", "),
                placeholders
            );

            let params: Vec<Value> = obj.values().cloned().collect();
            (query, params)
        } else {
            ("".to_string(), Vec::new())
        }
    }

    /// Build UPDATE query
    pub fn build_update(&self, provider: &crate::DatabaseProvider, values: &Value) -> (String, Vec<Value>) {
        if let Some(obj) = values.as_object() {
            let mut set_clauses = Vec::new();
            let mut params = Vec::new();

            for (key, value) in obj {
                let placeholder = match provider {
                    crate::DatabaseProvider::MySQL => "?",
                    crate::DatabaseProvider::PostgreSQL => &format!("${}", params.len() + 1),
                    crate::DatabaseProvider::SQLite => "?",
                };
                
                set_clauses.push(format!("{} = {}", self.quote_identifier(key, provider), placeholder));
                params.push(value.clone());
            }

            let mut query = format!(
                "UPDATE {} SET {}",
                self.quote_identifier(&self.table, provider),
                set_clauses.join(", ")
            );

            if !self.where_conditions.is_empty() {
                query.push_str(&format!(" WHERE {}", self.where_conditions.join(" AND ")));
                params.extend(self.parameters.clone());
            }

            (query, params)
        } else {
            ("".to_string(), Vec::new())
        }
    }

    /// Build DELETE query
    pub fn build_delete(&self, provider: &crate::DatabaseProvider) -> (String, Vec<Value>) {
        let mut query = format!("DELETE FROM {}", self.quote_identifier(&self.table, provider));

        if !self.where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_conditions.join(" AND ")));
        }

        (query, self.parameters.clone())
    }

    /// Quote identifier based on database provider
    fn quote_identifier(&self, identifier: &str, provider: &crate::DatabaseProvider) -> String {
        match provider {
            crate::DatabaseProvider::MySQL => format!("`{}`", identifier),
            crate::DatabaseProvider::PostgreSQL => format!("\"{}\"", identifier),
            crate::DatabaseProvider::SQLite => format!("[{}]", identifier),
        }
    }
}

/// Filter parser for processing DreamFactory filter syntax
pub struct FilterParser;

impl FilterParser {
    /// Parse filter string into SQL WHERE conditions
    pub fn parse_filter(filter: &str) -> Result<(Vec<String>, Vec<Value>), DatabaseError> {
        let mut conditions = Vec::new();
        let mut parameters = Vec::new();

        // Simple filter parsing - in production this would be more sophisticated
        if filter.contains('=') {
            let parts: Vec<&str> = filter.split('=').collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim();
                
                conditions.push(format!("{} = ?", field));
                parameters.push(Value::String(value.to_string()));
            }
        } else if filter.contains("like") {
            let parts: Vec<&str> = filter.split("like").collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim().trim_matches('\'').trim_matches('"');
                
                conditions.push(format!("{} LIKE ?", field));
                parameters.push(Value::String(format!("%{}%", value)));
            }
        } else if filter.contains('>') {
            let parts: Vec<&str> = filter.split('>').collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim();
                
                conditions.push(format!("{} > ?", field));
                if let Ok(num) = value.parse::<i64>() {
                    parameters.push(Value::Number(serde_json::Number::from(num)));
                } else {
                    parameters.push(Value::String(value.to_string()));
                }
            }
        } else if filter.contains('<') {
            let parts: Vec<&str> = filter.split('<').collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim();
                
                conditions.push(format!("{} < ?", field));
                if let Ok(num) = value.parse::<i64>() {
                    parameters.push(Value::Number(serde_json::Number::from(num)));
                } else {
                    parameters.push(Value::String(value.to_string()));
                }
            }
        }

        if conditions.is_empty() {
            // If no specific operators found, treat as a simple search
            conditions.push("1=1".to_string());
        }

        Ok((conditions, parameters))
    }

    /// Parse order by string
    pub fn parse_order_by(order: &str) -> Vec<String> {
        order.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Parse group by string
    pub fn parse_group_by(group: &str) -> Vec<String> {
        group.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Query parameters processor
pub struct QueryProcessor;

impl QueryProcessor {
    /// Process query parameters into a QueryBuilder
    pub fn process_params(
        table: &str,
        params: &QueryParams,
    ) -> Result<QueryBuilder, DatabaseError> {
        let mut builder = QueryBuilder::new(table);

        // Set select fields
        if let Some(fields) = &params.fields {
            builder = builder.select(fields.clone());
        }

        // Process filter
        if let Some(filter) = &params.filter {
            let (conditions, parameters) = FilterParser::parse_filter(filter)?;
            for (condition, param) in conditions.into_iter().zip(parameters.into_iter()) {
                builder = builder.where_condition(condition, Some(param));
            }
        }

        // Process order by
        if let Some(order) = &params.order {
            let order_fields = FilterParser::parse_order_by(order);
            builder = builder.order_by(order_fields);
        }

        // Process group by
        if let Some(group) = &params.group {
            let group_fields = FilterParser::parse_group_by(group);
            builder = builder.group_by(group_fields);
        }

        // Set limit and offset
        if let Some(limit) = params.limit {
            builder = builder.limit(limit);
        }
        if let Some(offset) = params.offset {
            builder = builder.offset(offset);
        }

        Ok(builder)
    }
}

/// SQL execution helper
pub struct SqlExecutor;

impl SqlExecutor {
    /// Execute a query and return results as JSON values
    pub async fn execute_select(
        pool: &sqlx::AnyPool,
        query: &str,
        params: &[Value],
    ) -> Result<Vec<Value>, DatabaseError> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = match param {
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
                _ => query_builder.bind(param.to_string()),
            };
        }

        let rows = query_builder.fetch_all(pool).await?;
        let mut results = Vec::new();

        for row in rows {
            let mut record = serde_json::Map::new();
            
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
                let value = Self::extract_value_from_row(&row, i)?;
                record.insert(column_name.to_string(), value);
            }
            
            results.push(Value::Object(record));
        }

        Ok(results)
    }

    /// Execute an insert/update/delete query
    pub async fn execute_modify(
        pool: &sqlx::AnyPool,
        query: &str,
        params: &[Value],
    ) -> Result<u64, DatabaseError> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = match param {
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
                _ => query_builder.bind(param.to_string()),
            };
        }

        let result = query_builder.execute(pool).await?;
        Ok(result.rows_affected())
    }

    /// Extract value from database row
    fn extract_value_from_row(row: &sqlx::AnyRow, index: usize) -> Result<Value, DatabaseError> {
        use sqlx::Row;
        
        let column = &row.columns()[index];
        let type_info = column.type_info();
        
        // Handle different data types based on the database type
        match type_info.name() {
            "TEXT" | "VARCHAR" | "CHAR" | "STRING" => {
                match row.try_get::<Option<String>, _>(index) {
                    Ok(Some(s)) => Ok(Value::String(s)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "INTEGER" | "INT" | "BIGINT" | "SMALLINT" => {
                match row.try_get::<Option<i64>, _>(index) {
                    Ok(Some(i)) => Ok(Value::Number(serde_json::Number::from(i))),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "REAL" | "FLOAT" | "DOUBLE" => {
                match row.try_get::<Option<f64>, _>(index) {
                    Ok(Some(f)) => Ok(Value::Number(
                        serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0))
                    )),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "BOOLEAN" | "BOOL" => {
                match row.try_get::<Option<bool>, _>(index) {
                    Ok(Some(b)) => Ok(Value::Bool(b)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            "JSON" | "JSONB" => {
                match row.try_get::<Option<String>, _>(index) {
                    Ok(Some(s)) => {
                        serde_json::from_str(&s).unwrap_or(Value::String(s))
                    }
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
            _ => {
                // Default to string for unknown types
                match row.try_get::<Option<String>, _>(index) {
                    Ok(Some(s)) => Ok(Value::String(s)),
                    Ok(None) => Ok(Value::Null),
                    Err(_) => Ok(Value::Null),
                }
            }
        }
    }
}