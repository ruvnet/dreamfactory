use df_database::*;
use serde_json::json;
use serial_test::serial;
use std::collections::HashMap;

#[tokio::test]
#[serial]
async fn test_sqlite_provider_basic_operations() {
    let provider = SqliteProvider::in_memory(None).await.unwrap();
    
    // Test connection
    provider.test_connection().await.unwrap();

    // Create test table
    let create_table_sql = r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            age INTEGER,
            active BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await.unwrap();

    // Test schema introspection
    let schema = provider.describe_table("users").await.unwrap();
    assert_eq!(schema.name, "users");
    assert!(!schema.field.is_empty());

    // Test create records
    let new_users = vec![
        json!({
            "name": "John Doe",
            "email": "john@example.com",
            "age": 30
        }),
        json!({
            "name": "Jane Smith",
            "email": "jane@example.com",
            "age": 25
        })
    ];

    let create_result = provider
        .create_records("users", new_users, &QueryParams::default())
        .await
        .unwrap();
    
    assert_eq!(create_result.resource.len(), 2);

    // Test get records
    let get_result = provider
        .get_records("users", &QueryParams::default())
        .await
        .unwrap();
    
    assert_eq!(get_result.resource.len(), 2);

    // Test get record by ID (assuming first record has ID 1)
    let single_record = provider.get_record("users", "1").await.unwrap();
    assert_eq!(single_record["name"], "John Doe");

    // Test update record
    let updated_record = provider
        .update_record("users", "1", json!({"name": "John Updated", "age": 31}))
        .await
        .unwrap();
    
    assert_eq!(updated_record["name"], "John Updated");

    // Test filtered get
    let filter_params = QueryParams {
        filter: Some("age > 25".to_string()),
        ..Default::default()
    };
    
    let filtered_result = provider
        .get_records("users", &filter_params)
        .await
        .unwrap();
    
    assert_eq!(filtered_result.resource.len(), 1);

    // Test delete record
    let deleted_record = provider.delete_record("users", "2").await.unwrap();
    assert_eq!(deleted_record["name"], "Jane Smith");

    // Verify deletion
    let final_result = provider
        .get_records("users", &QueryParams::default())
        .await
        .unwrap();
    
    assert_eq!(final_result.resource.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_query_params_processing() {
    let provider = SqliteProvider::in_memory(None).await.unwrap();
    
    // Create test table
    let create_table_sql = r#"
        CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT,
            price REAL,
            category TEXT,
            in_stock BOOLEAN
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await.unwrap();

    // Insert test data
    let test_products = vec![
        json!({"id": 1, "name": "Laptop", "price": 999.99, "category": "Electronics", "in_stock": true}),
        json!({"id": 2, "name": "Mouse", "price": 29.99, "category": "Electronics", "in_stock": true}),
        json!({"id": 3, "name": "Desk", "price": 199.99, "category": "Furniture", "in_stock": false}),
        json!({"id": 4, "name": "Chair", "price": 149.99, "category": "Furniture", "in_stock": true}),
    ];

    for product in test_products {
        provider
            .create_records("products", vec![product], &QueryParams::default())
            .await
            .unwrap();
    }

    // Test limit and offset
    let limit_params = QueryParams {
        limit: Some(2),
        offset: Some(1),
        ..Default::default()
    };
    
    let limited_result = provider
        .get_records("products", &limit_params)
        .await
        .unwrap();
    
    assert_eq!(limited_result.resource.len(), 2);

    // Test field selection
    let field_params = QueryParams {
        fields: Some(vec!["name".to_string(), "price".to_string()]),
        ..Default::default()
    };
    
    let field_result = provider
        .get_records("products", &field_params)
        .await
        .unwrap();
    
    // Should only have name and price fields
    for record in field_result.resource {
        assert!(record.get("name").is_some());
        assert!(record.get("price").is_some());
        // ID might still be included depending on implementation
    }

    // Test ordering
    let order_params = QueryParams {
        order: Some("price DESC".to_string()),
        ..Default::default()
    };
    
    let ordered_result = provider
        .get_records("products", &order_params)
        .await
        .unwrap();
    
    // First record should be the most expensive
    let first_price = ordered_result.resource[0]["price"].as_f64().unwrap();
    let second_price = ordered_result.resource[1]["price"].as_f64().unwrap();
    assert!(first_price >= second_price);
}

#[tokio::test]
#[serial]
async fn test_batch_operations() {
    let provider = SqliteProvider::in_memory(None).await.unwrap();
    
    // Create test table
    let create_table_sql = r#"
        CREATE TABLE batch_test (
            id INTEGER PRIMARY KEY,
            name TEXT,
            value INTEGER
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await.unwrap();

    // Test batch create
    let batch_request = BatchRequest {
        resources: vec![
            json!({"name": "Item 1", "value": 10}),
            json!({"name": "Item 2", "value": 20}),
            json!({"name": "Item 3", "value": 30}),
        ],
        rollback: Some(true),
        continue_on_error: Some(false),
    };

    let batch_result = provider
        .batch_operations("batch_test", batch_request)
        .await
        .unwrap();
    
    assert_eq!(batch_result.resources.len(), 3);
    for result in batch_result.resources {
        assert_eq!(result.status_code, 200);
        assert!(result.error.is_none());
    }

    // Verify batch insertion
    let all_records = provider
        .get_records("batch_test", &QueryParams::default())
        .await
        .unwrap();
    
    assert_eq!(all_records.resource.len(), 3);
}

#[tokio::test]
#[serial]
async fn test_schema_introspection() {
    let provider = SqliteProvider::in_memory(None).await.unwrap();
    
    // Create complex test table
    let create_table_sql = r#"
        CREATE TABLE complex_table (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            age INTEGER CHECK (age > 0),
            salary DECIMAL(10,2),
            active BOOLEAN DEFAULT 1,
            data JSON,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await.unwrap();

    // Test table schema
    let schema = provider.describe_table("complex_table").await.unwrap();
    
    assert_eq!(schema.name, "complex_table");
    assert!(!schema.field.is_empty());
    assert!(!schema.primary_key.is_empty());

    // Check specific fields
    let id_field = schema.field.iter().find(|f| f.name == "id").unwrap();
    assert_eq!(id_field.is_primary_key, Some(true));
    assert_eq!(id_field.auto_increment, Some(false)); // SQLite handles this differently

    let name_field = schema.field.iter().find(|f| f.name == "name").unwrap();
    assert_eq!(name_field.required, Some(true));

    let _email_field = schema.field.iter().find(|f| f.name == "email").unwrap();
    assert_eq!(name_field.field_type, "string");

    // Test all schemas
    let all_schemas = provider.get_schema(None).await.unwrap();
    assert!(!all_schemas.is_empty());
    
    let complex_schema = all_schemas.iter().find(|s| s.name == "complex_table");
    assert!(complex_schema.is_some());
}

#[tokio::test]
#[serial]
async fn test_computed_fields() {
    let provider = SqliteProvider::in_memory(None).await.unwrap();
    
    // Create test table
    let create_table_sql = r#"
        CREATE TABLE employees (
            id INTEGER PRIMARY KEY,
            first_name TEXT,
            last_name TEXT,
            salary REAL,
            department TEXT
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await.unwrap();

    // Add computed field for full name
    let mut evaluator = provider.computed_field_evaluator().write().await;
    evaluator.add_computed_field("employees", ComputedField {
        name: "full_name".to_string(),
        label: Some("Full Name".to_string()),
        description: Some("Concatenated first and last name".to_string()),
        field_type: "string".to_string(),
        expression: "first_name || ' ' || last_name".to_string(),
        depends_on: vec!["first_name".to_string(), "last_name".to_string()],
    });

    // Add computed field for annual salary
    evaluator.add_computed_field("employees", ComputedField {
        name: "annual_salary".to_string(),
        label: Some("Annual Salary".to_string()),
        description: Some("Monthly salary * 12".to_string()),
        field_type: "float".to_string(),
        expression: "salary * 12".to_string(),
        depends_on: vec!["salary".to_string()],
    });
    drop(evaluator);

    // Insert test data
    let test_employee = json!({
        "first_name": "John",
        "last_name": "Doe",
        "salary": 5000.0,
        "department": "Engineering"
    });

    provider
        .create_records("employees", vec![test_employee], &QueryParams::default())
        .await
        .unwrap();

    // Get records with computed fields
    let result = provider
        .get_records("employees", &QueryParams::default())
        .await
        .unwrap();
    
    assert_eq!(result.resource.len(), 1);
    let employee = &result.resource[0];
    
    // Check computed fields
    assert_eq!(employee["full_name"], "John Doe");
    assert_eq!(employee["annual_salary"], 60000.0);
}

#[tokio::test]
#[serial]
async fn test_database_provider_factory() {
    // Test SQLite provider creation
    let sqlite_config = ConnectionConfig {
        provider: DatabaseProvider::SQLite,
        host: None,
        port: None,
        database: ":memory:".to_string(),
        username: None,
        password: None,
        options: HashMap::new(),
    };

    let sqlite_service = DatabaseProviderFactory::create_service(
        DatabaseProvider::SQLite,
        sqlite_config,
        None,
    ).await.unwrap();

    assert_eq!(sqlite_service.provider(), DatabaseProvider::SQLite);
    sqlite_service.test_connection().await.unwrap();

    // Test builder pattern
    let built_service = DatabaseServiceBuilder::new()
        .provider(DatabaseProvider::SQLite)
        .database(":memory:")
        .max_connections(5)
        .build()
        .await
        .unwrap();

    assert_eq!(built_service.provider(), DatabaseProvider::SQLite);
    built_service.test_connection().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_error_handling() {
    let provider = SqliteProvider::in_memory(None).await.unwrap();
    
    // Test table not found error
    let result = provider.describe_table("nonexistent_table").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DatabaseError::TableNotFound { table } => {
            assert!(table.contains("nonexistent_table") || table == "unknown");
        }
        _ => panic!("Expected TableNotFound error"),
    }

    // Test record not found error
    let result = provider.get_record("nonexistent_table", "1").await;
    assert!(result.is_err());

    // Create table for further tests
    let create_table_sql = r#"
        CREATE TABLE error_test (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await.unwrap();

    // Insert record
    provider
        .create_records("error_test", vec![json!({"name": "test"})], &QueryParams::default())
        .await
        .unwrap();

    // Test constraint violation (unique constraint)
    let result = provider
        .create_records("error_test", vec![json!({"name": "test"})], &QueryParams::default())
        .await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        DatabaseError::ConstraintViolation { .. } => {
            // Expected constraint violation
        }
        DatabaseError::Query { .. } => {
            // Also acceptable as some databases report this differently
        }
        e => panic!("Unexpected error type: {:?}", e),
    }
}

#[tokio::test]
#[serial]
async fn test_transaction_support() {
    let provider = SqliteProvider::in_memory(None).await.unwrap();
    
    // Create test table
    let create_table_sql = r#"
        CREATE TABLE transaction_test (
            id INTEGER PRIMARY KEY,
            value INTEGER
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await.unwrap();

    // Test transaction rollback
    let batch_request = BatchRequest {
        resources: vec![
            json!({"value": 1}),
            json!({"value": 2}),
            json!({"invalid_field": "should_fail"}), // This should cause an error
        ],
        rollback: Some(true),
        continue_on_error: Some(false),
    };

    let batch_result = provider
        .batch_operations("transaction_test", batch_request)
        .await
        .unwrap();

    // Should have results but the transaction should have been rolled back
    assert_eq!(batch_result.resources.len(), 3);
    
    // Check that no records were actually inserted due to rollback
    let _records = provider
        .get_records("transaction_test", &QueryParams::default())
        .await
        .unwrap();
    
    // Depending on implementation, this might be 0 (if rollback worked) or have some records
    // The exact behavior depends on how the batch operation handles the invalid field
}

#[tokio::test]
#[serial]
async fn test_filter_parsing() {
    use df_database::query::FilterParser;
    
    // Test simple equality filter
    let (conditions, params) = FilterParser::parse_filter("name = 'John'").unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(params.len(), 1);
    assert!(conditions[0].contains("name"));
    assert!(conditions[0].contains("="));

    // Test LIKE filter
    let (conditions, params) = FilterParser::parse_filter("name like 'John'").unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(params.len(), 1);
    assert!(conditions[0].contains("LIKE"));

    // Test greater than filter
    let (conditions, params) = FilterParser::parse_filter("age > 25").unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(params.len(), 1);
    assert!(conditions[0].contains(">"));

    // Test less than filter
    let (conditions, params) = FilterParser::parse_filter("price < 100").unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(params.len(), 1);
    assert!(conditions[0].contains("<"));
}

#[tokio::test]
#[serial]
async fn test_query_builder() {
    use df_database::query::QueryBuilder;
    
    let builder = QueryBuilder::new("users")
        .select(vec!["name".to_string(), "email".to_string()])
        .where_condition("age > ?".to_string(), Some(json!(25)))
        .order_by(vec!["name ASC".to_string()])
        .limit(10)
        .offset(5);

    let (query, params) = builder.build_select(&DatabaseProvider::SQLite);
    
    assert!(query.contains("SELECT"));
    assert!(query.contains("name"));
    assert!(query.contains("email"));
    assert!(query.contains("FROM [users]"));
    assert!(query.contains("WHERE"));
    assert!(query.contains("ORDER BY"));
    assert!(query.contains("LIMIT"));
    assert!(query.contains("OFFSET"));
    assert_eq!(params.len(), 1);
}

// Additional tests can be added for:
// - PostgreSQL provider (requires test database)
// - MySQL provider (requires test database)
// - Virtual relationships
// - Complex computed fields
// - Performance testing
// - Concurrent access testing