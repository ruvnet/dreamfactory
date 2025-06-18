use df_database::*;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();

    println!("🚀 DreamFactory Database Layer Test");
    println!("=====================================");

    // Test SQLite provider
    test_sqlite_provider().await?;
    
    // Test database service builder
    test_service_builder().await?;
    
    // Test schema introspection
    test_schema_introspection().await?;
    
    // Test computed fields
    test_computed_fields().await?;
    
    // Test batch operations
    test_batch_operations().await?;
    
    println!("\n✅ All tests completed successfully!");
    
    Ok(())
}

async fn test_sqlite_provider() -> Result<(), DatabaseError> {
    println!("\n📂 Testing SQLite Provider");
    println!("--------------------------");
    
    // Create in-memory SQLite database
    let provider = SqliteProvider::in_memory(None).await?;
    println!("✓ Created in-memory SQLite database");
    
    // Test connection
    provider.test_connection().await?;
    println!("✓ Connection test passed");
    
    // Get SQLite version
    let version = provider.get_version().await?;
    println!("✓ SQLite version: {}", version);
    
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
    
    provider.execute_raw_sql(create_table_sql).await?;
    println!("✓ Created users table");
    
    // Insert test data
    let new_users = vec![
        json!({
            "name": "Alice Johnson",
            "email": "alice@example.com",
            "age": 28
        }),
        json!({
            "name": "Bob Smith",
            "email": "bob@example.com",
            "age": 35
        }),
        json!({
            "name": "Carol Brown",
            "email": "carol@example.com",
            "age": 42
        })
    ];
    
    let create_result = provider
        .create_records("users", new_users, &QueryParams::default())
        .await?;
    
    println!("✓ Inserted {} users", create_result.resource.len());
    
    // Query all users
    let all_users = provider
        .get_records("users", &QueryParams::default())
        .await?;
    
    println!("✓ Retrieved {} users", all_users.resource.len());
    
    // Query with filter
    let filter_params = QueryParams {
        filter: Some("age > 30".to_string()),
        ..Default::default()
    };
    
    let filtered_users = provider
        .get_records("users", &filter_params)
        .await?;
    
    println!("✓ Filtered query returned {} users over 30", filtered_users.resource.len());
    
    // Query with limit and order
    let ordered_params = QueryParams {
        order: Some("age DESC".to_string()),
        limit: Some(2),
        ..Default::default()
    };
    
    let ordered_users = provider
        .get_records("users", &ordered_params)
        .await?;
    
    println!("✓ Ordered query returned {} users", ordered_users.resource.len());
    for user in &ordered_users.resource {
        println!("  - {} (age: {})", user["name"], user["age"]);
    }
    
    // Update a user
    let updated_user = provider
        .update_record("users", "1", json!({"name": "Alice Updated", "age": 29}))
        .await?;
    
    println!("✓ Updated user: {}", updated_user["name"]);
    
    // Get single user
    let single_user = provider.get_record("users", "2").await?;
    println!("✓ Retrieved single user: {}", single_user["name"]);
    
    Ok(())
}

async fn test_service_builder() -> Result<(), DatabaseError> {
    println!("\n🏗️  Testing Database Service Builder");
    println!("------------------------------------");
    
    let service = DatabaseServiceBuilder::new()
        .provider(DatabaseProvider::SQLite)
        .database(":memory:")
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(30)
        .build()
        .await?;
    
    println!("✓ Built database service with builder pattern");
    
    service.test_connection().await?;
    println!("✓ Service connection test passed");
    
    println!("✓ Provider: {}", service.provider());
    
    Ok(())
}

async fn test_schema_introspection() -> Result<(), DatabaseError> {
    println!("\n🔍 Testing Schema Introspection");
    println!("-------------------------------");
    
    let provider = SqliteProvider::in_memory(None).await?;
    
    // Create complex table
    let create_table_sql = r#"
        CREATE TABLE products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            price DECIMAL(10,2) NOT NULL,
            category_id INTEGER,
            in_stock BOOLEAN DEFAULT 1,
            tags JSON,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME,
            FOREIGN KEY (category_id) REFERENCES categories(id)
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await?;
    
    println!("✓ Created products table");
    
    // Get table schema
    let schema = provider.describe_table("products").await?;
    
    println!("✓ Table: {}", schema.name);
    println!("  Primary keys: {:?}", schema.primary_key);
    println!("  Fields:");
    
    for field in &schema.field {
        println!("    - {} ({}): {}{}{}",
            field.name,
            field.field_type,
            field.db_type.as_ref().unwrap_or(&"".to_string()),
            if field.required.unwrap_or(false) { " NOT NULL" } else { "" },
            if field.is_primary_key.unwrap_or(false) { " PRIMARY KEY" } else { "" }
        );
    }
    
    // Get all schemas
    let all_schemas = provider.get_schema(None).await?;
    println!("✓ Found {} tables in database", all_schemas.len());
    
    for table_schema in &all_schemas {
        println!("  - {}", table_schema.name);
    }
    
    Ok(())
}

async fn test_computed_fields() -> Result<(), DatabaseError> {
    println!("\n🧮 Testing Computed Fields");
    println!("--------------------------");
    
    let provider = SqliteProvider::in_memory(None).await?;
    
    // Create employees table
    let create_table_sql = r#"
        CREATE TABLE employees (
            id INTEGER PRIMARY KEY,
            first_name TEXT,
            last_name TEXT,
            salary REAL,
            department TEXT,
            start_date DATE
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await?;
    
    println!("✓ Created employees table");
    
    // Add computed fields
    {
        let mut evaluator = provider.computed_field_evaluator.write().await;
        
        // Full name computed field
        evaluator.add_computed_field("employees", ComputedField {
            name: "full_name".to_string(),
            label: Some("Full Name".to_string()),
            description: Some("First and last name combined".to_string()),
            field_type: "string".to_string(),
            expression: "first_name || ' ' || last_name".to_string(),
            depends_on: vec!["first_name".to_string(), "last_name".to_string()],
        });
        
        // Annual salary computed field
        evaluator.add_computed_field("employees", ComputedField {
            name: "annual_salary".to_string(),
            label: Some("Annual Salary".to_string()),
            description: Some("Monthly salary multiplied by 12".to_string()),
            field_type: "float".to_string(),
            expression: "salary * 12".to_string(),
            depends_on: vec!["salary".to_string()],
        });
        
        // High earner status
        evaluator.add_computed_field("employees", ComputedField {
            name: "high_earner".to_string(),
            label: Some("High Earner".to_string()),
            description: Some("Whether employee earns more than $100k annually".to_string()),
            field_type: "boolean".to_string(),
            expression: "IF(salary * 12 > 100000, 'true', 'false')".to_string(),
            depends_on: vec!["salary".to_string()],
        });
    }
    
    println!("✓ Added computed fields: full_name, annual_salary, high_earner");
    
    // Insert test employees
    let employees = vec![
        json!({
            "first_name": "John",
            "last_name": "Doe",
            "salary": 8500.0,
            "department": "Engineering"
        }),
        json!({
            "first_name": "Jane",
            "last_name": "Smith",
            "salary": 7200.0,
            "department": "Marketing"
        }),
        json!({
            "first_name": "Mike",
            "last_name": "Johnson",
            "salary": 9800.0,
            "department": "Engineering"
        })
    ];
    
    provider
        .create_records("employees", employees, &QueryParams::default())
        .await?;
    
    println!("✓ Inserted {} employees", 3);
    
    // Retrieve employees with computed fields
    let result = provider
        .get_records("employees", &QueryParams::default())
        .await?;
    
    println!("✓ Retrieved employees with computed fields:");
    
    for employee in &result.resource {
        println!("  - {}: ${}/month, ${}/year (High earner: {})",
            employee["full_name"],
            employee["salary"],
            employee["annual_salary"],
            employee["high_earner"]
        );
    }
    
    Ok(())
}

async fn test_batch_operations() -> Result<(), DatabaseError> {
    println!("\n📦 Testing Batch Operations");
    println!("---------------------------");
    
    let provider = SqliteProvider::in_memory(None).await?;
    
    // Create orders table
    let create_table_sql = r#"
        CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            customer_id INTEGER,
            product_name TEXT,
            quantity INTEGER,
            unit_price REAL,
            status TEXT DEFAULT 'pending'
        )
    "#;
    
    provider.execute_raw_sql(create_table_sql).await?;
    
    println!("✓ Created orders table");
    
    // Prepare batch request
    let batch_request = BatchRequest {
        resources: vec![
            json!({
                "customer_id": 1,
                "product_name": "Widget A",
                "quantity": 2,
                "unit_price": 19.99
            }),
            json!({
                "customer_id": 2,
                "product_name": "Widget B",
                "quantity": 1,
                "unit_price": 29.99
            }),
            json!({
                "customer_id": 1,
                "product_name": "Widget C",
                "quantity": 3,
                "unit_price": 9.99
            }),
            json!({
                "customer_id": 3,
                "product_name": "Widget D",
                "quantity": 1,
                "unit_price": 49.99
            })
        ],
        rollback: Some(false),
        continue_on_error: Some(true),
    };
    
    println!("✓ Prepared batch with {} operations", batch_request.resources.len());
    
    // Execute batch
    let batch_result = provider
        .batch_operations("orders", batch_request)
        .await?;
    
    println!("✓ Executed batch operations");
    println!("  Results:");
    
    for (i, result) in batch_result.resources.iter().enumerate() {
        println!("    Operation {}: Status {}, {}",
            i + 1,
            result.status_code,
            if result.error.is_some() {
                format!("Error: {}", result.error.as_ref().unwrap())
            } else {
                "Success".to_string()
            }
        );
    }
    
    // Verify batch results
    let all_orders = provider
        .get_records("orders", &QueryParams::default())
        .await?;
    
    println!("✓ Total orders in database: {}", all_orders.resource.len());
    
    // Calculate totals
    let mut total_value = 0.0;
    for order in &all_orders.resource {
        let quantity = order["quantity"].as_f64().unwrap_or(0.0);
        let unit_price = order["unit_price"].as_f64().unwrap_or(0.0);
        total_value += quantity * unit_price;
    }
    
    println!("✓ Total order value: ${:.2}", total_value);
    
    Ok(())
}

// Additional demonstration functions could include:
// - Virtual relationships demo
// - Complex query filtering
// - Performance benchmarking
// - Error handling scenarios
// - Provider-specific features