use crate::DatabaseError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Database table schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub plural: Option<String>,
    pub primary_key: Vec<String>,
    pub name_field: Option<String>,
    pub field: Vec<FieldSchema>,
    pub related: Option<Vec<RelatedSchema>>,
    pub access: Option<Vec<String>>,
}

/// Database field/column schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub label: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub field_type: String,
    pub db_type: Option<String>,
    pub length: Option<u32>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
    pub default_value: Option<Value>,
    pub required: Option<bool>,
    pub allow_null: Option<bool>,
    pub fixed_length: Option<bool>,
    pub supports_multibyte: Option<bool>,
    pub auto_increment: Option<bool>,
    pub is_primary_key: Option<bool>,
    pub is_unique: Option<bool>,
    pub is_index: Option<bool>,
    pub is_foreign_key: Option<bool>,
    pub ref_table: Option<String>,
    pub ref_fields: Option<String>,
    pub ref_on_update: Option<String>,
    pub ref_on_delete: Option<String>,
    pub validation: Option<Vec<FieldValidation>>,
    pub values: Option<Vec<String>>,
    pub picklist: Option<Vec<PicklistValue>>,
}

/// Related table schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedSchema {
    pub name: String,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub relation_type: String,
    pub field: String,
    pub ref_table: String,
    pub ref_field: String,
    pub junction_table: Option<String>,
    pub junction_field: Option<String>,
    pub junction_ref_field: Option<String>,
}

/// Field validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
    #[serde(rename = "type")]
    pub validation_type: String,
    pub value: Option<Value>,
    pub message: Option<String>,
}

/// Picklist values for enum fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicklistValue {
    pub label: String,
    pub value: Value,
}

/// Schema introspection service
pub struct SchemaIntrospector {
    provider: crate::DatabaseProvider,
}

impl SchemaIntrospector {
    pub fn new(provider: crate::DatabaseProvider) -> Self {
        Self { provider }
    }

    /// Get all table schemas from database
    pub async fn get_all_schemas(
        &self,
        pool: &sqlx::AnyPool,
    ) -> Result<Vec<TableSchema>, DatabaseError> {
        match self.provider {
            crate::DatabaseProvider::MySQL => self.get_mysql_schemas(pool).await,
            crate::DatabaseProvider::PostgreSQL => self.get_postgres_schemas(pool).await,
            crate::DatabaseProvider::SQLite => self.get_sqlite_schemas(pool).await,
        }
    }

    /// Get schema for a specific table
    pub async fn get_table_schema(
        &self,
        pool: &sqlx::AnyPool,
        table_name: &str,
    ) -> Result<TableSchema, DatabaseError> {
        match self.provider {
            crate::DatabaseProvider::MySQL => self.get_mysql_table_schema(pool, table_name).await,
            crate::DatabaseProvider::PostgreSQL => {
                self.get_postgres_table_schema(pool, table_name).await
            }
            crate::DatabaseProvider::SQLite => {
                self.get_sqlite_table_schema(pool, table_name).await
            }
        }
    }

    /// Get MySQL table schemas
    async fn get_mysql_schemas(
        &self,
        pool: &sqlx::AnyPool,
    ) -> Result<Vec<TableSchema>, DatabaseError> {
        let query = r#"
            SELECT 
                TABLE_NAME,
                TABLE_COMMENT,
                TABLE_SCHEMA
            FROM information_schema.TABLES 
            WHERE TABLE_SCHEMA = DATABASE() 
            AND TABLE_TYPE = 'BASE TABLE'
            ORDER BY TABLE_NAME
        "#;

        let rows = sqlx::query(query).fetch_all(pool).await?;
        let mut schemas = Vec::new();

        for row in rows {
            let table_name: String = row.try_get("TABLE_NAME")?;
            let schema = self.get_mysql_table_schema(pool, &table_name).await?;
            schemas.push(schema);
        }

        Ok(schemas)
    }

    /// Get MySQL table schema
    async fn get_mysql_table_schema(
        &self,
        pool: &sqlx::AnyPool,
        table_name: &str,
    ) -> Result<TableSchema, DatabaseError> {
        // Get table information
        let table_query = r#"
            SELECT 
                TABLE_NAME,
                TABLE_COMMENT
            FROM information_schema.TABLES 
            WHERE TABLE_SCHEMA = DATABASE() 
            AND TABLE_NAME = ?
        "#;

        let table_row = sqlx::query(table_query)
            .bind(table_name)
            .fetch_one(pool)
            .await?;

        let table_comment: Option<String> = table_row.try_get("TABLE_COMMENT").ok();

        // Get column information
        let columns_query = r#"
            SELECT 
                COLUMN_NAME,
                DATA_TYPE,
                COLUMN_TYPE,
                IS_NULLABLE,
                COLUMN_DEFAULT,
                EXTRA,
                COLUMN_COMMENT,
                CHARACTER_MAXIMUM_LENGTH,
                NUMERIC_PRECISION,
                NUMERIC_SCALE,
                COLUMN_KEY
            FROM information_schema.COLUMNS 
            WHERE TABLE_SCHEMA = DATABASE() 
            AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION
        "#;

        let column_rows = sqlx::query(columns_query)
            .bind(table_name)
            .fetch_all(pool)
            .await?;

        let mut fields = Vec::new();
        let mut primary_keys = Vec::new();

        for row in column_rows {
            let column_name: String = row.try_get("COLUMN_NAME")?;
            let data_type: String = row.try_get("DATA_TYPE")?;
            let column_type: String = row.try_get("COLUMN_TYPE")?;
            let is_nullable: String = row.try_get("IS_NULLABLE")?;
            let column_default: Option<String> = row.try_get("COLUMN_DEFAULT").ok();
            let extra: String = row.try_get("EXTRA")?;
            let column_comment: Option<String> = row.try_get("COLUMN_COMMENT").ok();
            let max_length: Option<u32> = row.try_get("CHARACTER_MAXIMUM_LENGTH").ok();
            let precision: Option<u32> = row.try_get("NUMERIC_PRECISION").ok();
            let scale: Option<u32> = row.try_get("NUMERIC_SCALE").ok();
            let column_key: String = row.try_get("COLUMN_KEY")?;

            let is_primary = column_key.contains("PRI");
            if is_primary {
                primary_keys.push(column_name.clone());
            }

            let field = FieldSchema {
                name: column_name,
                label: None,
                description: column_comment,
                field_type: self.map_mysql_type_to_df_type(&data_type),
                db_type: Some(column_type),
                length: max_length,
                precision,
                scale,
                default_value: column_default.map(|v| serde_json::Value::String(v)),
                required: Some(is_nullable == "NO"),
                allow_null: Some(is_nullable == "YES"),
                fixed_length: None,
                supports_multibyte: None,
                auto_increment: Some(extra.contains("auto_increment")),
                is_primary_key: Some(is_primary),
                is_unique: Some(column_key.contains("UNI")),
                is_index: Some(!column_key.is_empty()),
                is_foreign_key: Some(column_key.contains("MUL")),
                ref_table: None,
                ref_fields: None,
                ref_on_update: None,
                ref_on_delete: None,
                validation: None,
                values: None,
                picklist: None,
            };

            fields.push(field);
        }

        Ok(TableSchema {
            name: table_name.to_string(),
            label: None,
            description: table_comment,
            plural: None,
            primary_key: primary_keys,
            name_field: None,
            field: fields,
            related: None,
            access: None,
        })
    }

    /// Get PostgreSQL table schemas
    async fn get_postgres_schemas(
        &self,
        pool: &sqlx::AnyPool,
    ) -> Result<Vec<TableSchema>, DatabaseError> {
        let query = r#"
            SELECT tablename 
            FROM pg_tables 
            WHERE schemaname = 'public'
            ORDER BY tablename
        "#;

        let rows = sqlx::query(query).fetch_all(pool).await?;
        let mut schemas = Vec::new();

        for row in rows {
            let table_name: String = row.try_get("tablename")?;
            let schema = self.get_postgres_table_schema(pool, &table_name).await?;
            schemas.push(schema);
        }

        Ok(schemas)
    }

    /// Get PostgreSQL table schema
    async fn get_postgres_table_schema(
        &self,
        pool: &sqlx::AnyPool,
        table_name: &str,
    ) -> Result<TableSchema, DatabaseError> {
        let columns_query = r#"
            SELECT 
                c.column_name,
                c.data_type,
                c.udt_name,
                c.is_nullable,
                c.column_default,
                c.character_maximum_length,
                c.numeric_precision,
                c.numeric_scale,
                CASE WHEN pk.column_name IS NOT NULL THEN 'YES' ELSE 'NO' END as is_primary
            FROM information_schema.columns c
            LEFT JOIN (
                SELECT kcu.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu 
                    ON tc.constraint_name = kcu.constraint_name
                WHERE tc.table_name = $1 
                AND tc.constraint_type = 'PRIMARY KEY'
            ) pk ON c.column_name = pk.column_name
            WHERE c.table_name = $1
            ORDER BY c.ordinal_position
        "#;

        let column_rows = sqlx::query(columns_query)
            .bind(table_name)
            .fetch_all(pool)
            .await?;

        let mut fields = Vec::new();
        let mut primary_keys = Vec::new();

        for row in column_rows {
            let column_name: String = row.try_get("column_name")?;
            let data_type: String = row.try_get("data_type")?;
            let udt_name: String = row.try_get("udt_name")?;
            let is_nullable: String = row.try_get("is_nullable")?;
            let column_default: Option<String> = row.try_get("column_default").ok();
            let max_length: Option<i32> = row.try_get("character_maximum_length").ok();
            let precision: Option<i32> = row.try_get("numeric_precision").ok();
            let scale: Option<i32> = row.try_get("numeric_scale").ok();
            let is_primary: String = row.try_get("is_primary")?;

            if is_primary == "YES" {
                primary_keys.push(column_name.clone());
            }

            let is_auto_increment = column_default
                .as_ref()
                .map(|d| d.starts_with("nextval("))
                .unwrap_or(false);

            let field = FieldSchema {
                name: column_name,
                label: None,
                description: None,
                field_type: self.map_postgres_type_to_df_type(&data_type, &udt_name),
                db_type: Some(udt_name),
                length: max_length.map(|l| l as u32),
                precision: precision.map(|p| p as u32),
                scale: scale.map(|s| s as u32),
                default_value: column_default.map(|v| serde_json::Value::String(v)),
                required: Some(is_nullable == "NO"),
                allow_null: Some(is_nullable == "YES"),
                fixed_length: None,
                supports_multibyte: None,
                auto_increment: Some(is_auto_increment),
                is_primary_key: Some(is_primary == "YES"),
                is_unique: None,
                is_index: None,
                is_foreign_key: None,
                ref_table: None,
                ref_fields: None,
                ref_on_update: None,
                ref_on_delete: None,
                validation: None,
                values: None,
                picklist: None,
            };

            fields.push(field);
        }

        Ok(TableSchema {
            name: table_name.to_string(),
            label: None,
            description: None,
            plural: None,
            primary_key: primary_keys,
            name_field: None,
            field: fields,
            related: None,
            access: None,
        })
    }

    /// Get SQLite table schemas
    async fn get_sqlite_schemas(
        &self,
        pool: &sqlx::AnyPool,
    ) -> Result<Vec<TableSchema>, DatabaseError> {
        let query = r#"
            SELECT name 
            FROM sqlite_master 
            WHERE type = 'table' 
            AND name NOT LIKE 'sqlite_%'
            ORDER BY name
        "#;

        let rows = sqlx::query(query).fetch_all(pool).await?;
        let mut schemas = Vec::new();

        for row in rows {
            let table_name: String = row.try_get("name")?;
            let schema = self.get_sqlite_table_schema(pool, &table_name).await?;
            schemas.push(schema);
        }

        Ok(schemas)
    }

    /// Get SQLite table schema
    async fn get_sqlite_table_schema(
        &self,
        pool: &sqlx::AnyPool,
        table_name: &str,
    ) -> Result<TableSchema, DatabaseError> {
        let pragma_query = format!("PRAGMA table_info({})", table_name);
        let column_rows = sqlx::query(&pragma_query).fetch_all(pool).await?;

        let mut fields = Vec::new();
        let mut primary_keys = Vec::new();

        for row in column_rows {
            let column_name: String = row.try_get("name")?;
            let data_type: String = row.try_get("type")?;
            let not_null: bool = row.try_get("notnull")?;
            let default_value: Option<String> = row.try_get("dflt_value").ok();
            let is_primary: bool = row.try_get("pk")?;

            if is_primary {
                primary_keys.push(column_name.clone());
            }

            let field = FieldSchema {
                name: column_name,
                label: None,
                description: None,
                field_type: self.map_sqlite_type_to_df_type(&data_type),
                db_type: Some(data_type),
                length: None,
                precision: None,
                scale: None,
                default_value: default_value.map(|v| serde_json::Value::String(v)),
                required: Some(not_null),
                allow_null: Some(!not_null),
                fixed_length: None,
                supports_multibyte: None,
                auto_increment: Some(false), // SQLite handles this differently
                is_primary_key: Some(is_primary),
                is_unique: None,
                is_index: None,
                is_foreign_key: None,
                ref_table: None,
                ref_fields: None,
                ref_on_update: None,
                ref_on_delete: None,
                validation: None,
                values: None,
                picklist: None,
            };

            fields.push(field);
        }

        Ok(TableSchema {
            name: table_name.to_string(),
            label: None,
            description: None,
            plural: None,
            primary_key: primary_keys,
            name_field: None,
            field: fields,
            related: None,
            access: None,
        })
    }

    /// Map MySQL data types to DreamFactory types
    fn map_mysql_type_to_df_type(&self, mysql_type: &str) -> String {
        match mysql_type.to_lowercase().as_str() {
            "int" | "integer" | "tinyint" | "smallint" | "mediumint" => "integer".to_string(),
            "bigint" => "big_integer".to_string(),
            "decimal" | "numeric" => "decimal".to_string(),
            "float" | "double" | "real" => "float".to_string(),
            "varchar" | "char" => "string".to_string(),
            "text" | "longtext" | "mediumtext" | "tinytext" => "text".to_string(),
            "date" => "date".to_string(),
            "time" => "time".to_string(),
            "datetime" | "timestamp" => "datetime".to_string(),
            "boolean" | "bool" => "boolean".to_string(),
            "json" => "json".to_string(),
            "blob" | "longblob" | "mediumblob" | "tinyblob" => "binary".to_string(),
            _ => "string".to_string(),
        }
    }

    /// Map PostgreSQL data types to DreamFactory types
    fn map_postgres_type_to_df_type(&self, pg_type: &str, udt_name: &str) -> String {
        match pg_type.to_lowercase().as_str() {
            "integer" | "int4" => "integer".to_string(),
            "bigint" | "int8" => "big_integer".to_string(),
            "smallint" | "int2" => "integer".to_string(),
            "numeric" | "decimal" => "decimal".to_string(),
            "real" | "float4" => "float".to_string(),
            "double precision" | "float8" => "float".to_string(),
            "character varying" | "varchar" | "character" | "char" => "string".to_string(),
            "text" => "text".to_string(),
            "date" => "date".to_string(),
            "time" | "time without time zone" => "time".to_string(),
            "timestamp" | "timestamp without time zone" | "timestamp with time zone" => {
                "datetime".to_string()
            }
            "boolean" | "bool" => "boolean".to_string(),
            "json" | "jsonb" => "json".to_string(),
            "bytea" => "binary".to_string(),
            "uuid" => "string".to_string(),
            _ => match udt_name {
                "uuid" => "string".to_string(),
                _ => "string".to_string(),
            },
        }
    }

    /// Map SQLite data types to DreamFactory types
    fn map_sqlite_type_to_df_type(&self, sqlite_type: &str) -> String {
        match sqlite_type.to_uppercase().as_str() {
            t if t.starts_with("INT") => "integer".to_string(),
            t if t.starts_with("REAL") || t.starts_with("FLOA") || t.starts_with("DOUB") => {
                "float".to_string()
            }
            t if t.starts_with("TEXT") || t.starts_with("CHAR") || t.starts_with("VARCHAR") => {
                "string".to_string()
            }
            t if t.starts_with("BLOB") => "binary".to_string(),
            "NUMERIC" => "decimal".to_string(),
            "BOOLEAN" => "boolean".to_string(),
            "DATE" => "date".to_string(),
            "TIME" => "time".to_string(),
            "DATETIME" | "TIMESTAMP" => "datetime".to_string(),
            _ => "string".to_string(),
        }
    }
}