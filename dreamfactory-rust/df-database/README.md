# DreamFactory Database Abstraction Layer

A complete Rust implementation of DreamFactory's database layer with multi-provider support and 100% API compatibility.

## Features

### 🗄️ Multi-Database Provider Support
- **MySQL** - Full support with MySQL-specific optimizations
- **PostgreSQL** - Complete PostgreSQL integration with advanced features
- **SQLite** - Lightweight database support with in-memory options

### 🔍 Schema Introspection
- Automatic table and field discovery
- Primary key and foreign key detection
- Index information retrieval
- Data type mapping to DreamFactory types
- Constraint and validation rule extraction

### 📝 CRUD Operations
- **Create** - Insert single or multiple records
- **Read** - Query with filtering, sorting, pagination
- **Update** - Modify existing records with conditions
- **Delete** - Remove records with safety checks

### 🔗 Advanced Relationships
- **belongs_to** - Many-to-one relationships
- **has_one** - One-to-one relationships  
- **has_many** - One-to-many relationships
- **many_to_many** - Many-to-many with junction tables
- **Virtual relationships** - Custom relationship definitions

### 🧮 Computed Fields
- **Concatenation** - Combine multiple fields
- **Mathematical** - Arithmetic operations
- **Conditional** - IF/THEN logic
- **Formatting** - String templating
- **Custom expressions** - User-defined calculations

### 📦 Batch Operations
- **Transaction support** - ACID compliance
- **Error handling** - Continue or rollback on failure
- **Bulk inserts** - Efficient multi-record operations
- **Mixed operations** - Combine different operation types

## Architecture

```
df-database/
├── src/
│   ├── lib.rs              # Main exports and traits
│   ├── error.rs            # Error types and handling
│   ├── database.rs         # Core database service implementation
│   ├── schema.rs           # Schema introspection
│   ├── query.rs            # Query building and execution
│   ├── relationships.rs    # Relationship handling
│   ├── computed_fields.rs  # Computed field evaluation
│   ├── types.rs            # Type definitions
│   └── providers/          # Database-specific implementations
│       ├── mod.rs          # Provider factory
│       ├── mysql.rs        # MySQL provider
│       ├── postgresql.rs   # PostgreSQL provider
│       └── sqlite.rs       # SQLite provider
├── tests/
│   └── integration_tests.rs # Comprehensive test suite
└── bin/
    └── db_test.rs          # Demo application
```

## Core Traits

### DatabaseService
Main trait that all providers implement:

```rust
#[async_trait]
pub trait DatabaseService: Send + Sync {
    // Connection management
    fn provider(&self) -> DatabaseProvider;
    async fn test_connection(&self) -> Result<(), DatabaseError>;
    
    // Schema operations
    async fn get_schema(&self, table: Option<&str>) -> Result<Vec<TableSchema>, DatabaseError>;
    async fn describe_table(&self, table: &str) -> Result<TableSchema, DatabaseError>;
    
    // CRUD operations
    async fn get_records(&self, table: &str, params: &QueryParams) -> Result<DatabaseResult, DatabaseError>;
    async fn get_record(&self, table: &str, id: &str) -> Result<Value, DatabaseError>;
    async fn create_records(&self, table: &str, records: Vec<Value>, params: &QueryParams) -> Result<DatabaseResult, DatabaseError>;
    async fn update_records(&self, table: &str, records: Vec<Value>, params: &QueryParams) -> Result<DatabaseResult, DatabaseError>;
    async fn update_record(&self, table: &str, id: &str, record: Value) -> Result<Value, DatabaseError>;
    async fn delete_records(&self, table: &str, params: &QueryParams) -> Result<DatabaseResult, DatabaseError>;
    async fn delete_record(&self, table: &str, id: &str) -> Result<Value, DatabaseError>;
    
    // Batch operations
    async fn batch_operations(&self, table: &str, batch: BatchRequest) -> Result<BatchResponse, DatabaseError>;
    
    // Advanced features
    async fn get_relationships(&self, table: &str) -> Result<Vec<Relationship>, DatabaseError>;
    async fn execute_virtual_relationships(&self, table: &str, records: &mut Vec<Value>, relationships: &[String]) -> Result<(), DatabaseError>;
    async fn evaluate_computed_fields(&self, table: &str, records: &mut Vec<Value>) -> Result<(), DatabaseError>;
}
```

## Usage Examples

### Basic Connection
```rust
use df_database::*;

// Create SQLite provider
let provider = SqliteProvider::in_memory(None).await?;

// Test connection
provider.test_connection().await?;

// Create service using builder
let service = DatabaseServiceBuilder::new()
    .provider(DatabaseProvider::PostgreSQL)
    .host("localhost")
    .port(5432)
    .database("mydb")
    .username("user")
    .password("pass")
    .build()
    .await?;
```

### CRUD Operations
```rust
// Create records
let users = vec![
    json!({"name": "John Doe", "email": "john@example.com"}),
    json!({"name": "Jane Smith", "email": "jane@example.com"})
];
let result = service.create_records("users", users, &QueryParams::default()).await?;

// Query with filtering and pagination
let params = QueryParams {
    filter: Some("age > 25".to_string()),
    limit: Some(10),
    offset: Some(0),
    order: Some("name ASC".to_string()),
    ..Default::default()
};
let users = service.get_records("users", &params).await?;

// Update single record
let updated = service.update_record(
    "users", 
    "1", 
    json!({"name": "John Updated"})
).await?;

// Delete with conditions
let delete_params = QueryParams {
    filter: Some("active = false".to_string()),
    ..Default::default()
};
let deleted = service.delete_records("users", &delete_params).await?;
```

### Schema Introspection
```rust
// Get all table schemas
let all_schemas = service.get_schema(None).await?;

// Get specific table schema
let user_schema = service.describe_table("users").await?;
println!("Table: {}", user_schema.name);
println!("Primary keys: {:?}", user_schema.primary_key);

for field in &user_schema.field {
    println!("Field: {} ({})", field.name, field.field_type);
}
```

### Batch Operations
```rust
let batch = BatchRequest {
    resources: vec![
        json!({"name": "User 1"}),
        json!({"name": "User 2"}),
        json!({"name": "User 3"}),
    ],
    rollback: Some(true),
    continue_on_error: Some(false),
};

let batch_result = service.batch_operations("users", batch).await?;
for result in batch_result.resources {
    println!("Status: {}, Success: {}", result.status_code, result.error.is_none());
}
```

### Computed Fields
```rust
// Add computed field to evaluator
let mut evaluator = ComputedFieldEvaluator::new();
evaluator.add_computed_field("users", ComputedField {
    name: "full_name".to_string(),
    label: Some("Full Name".to_string()),
    field_type: "string".to_string(),
    expression: "first_name || ' ' || last_name".to_string(),
    depends_on: vec!["first_name".to_string(), "last_name".to_string()],
});

// Computed fields are automatically evaluated during record retrieval
let users = service.get_records("users", &QueryParams::default()).await?;
// Each user record now includes the computed 'full_name' field
```

## Query Parameters

The `QueryParams` struct supports DreamFactory's standard query parameters:

- **filter** - SQL-like filtering conditions
- **limit** - Maximum number of records to return
- **offset** - Number of records to skip
- **order** - Sorting specification
- **group** - Grouping specification
- **fields** - Specific fields to select
- **related** - Related data to include
- **include_count** - Include total count in response
- **include_schema** - Include schema information

## Error Handling

Comprehensive error types for different failure scenarios:

```rust
pub enum DatabaseError {
    Connection { message: String },
    Query { message: String },
    Schema { message: String },
    TableNotFound { table: String },
    ColumnNotFound { table: String, column: String },
    RecordNotFound { id: String },
    ConstraintViolation { message: String },
    Transaction { message: String },
    Validation { field: String, message: String },
    // ... and more
}
```

## Testing

Comprehensive test suite with:

- **Unit tests** - Individual component testing
- **Integration tests** - End-to-end functionality
- **Provider tests** - Database-specific features
- **Error handling tests** - Failure scenarios
- **Performance tests** - Load and stress testing

Run tests with:
```bash
cargo test
cargo test --features test-mysql    # MySQL tests (requires database)
cargo test --features test-postgres # PostgreSQL tests (requires database)
```

## Dependencies

Core dependencies:
- **sqlx** - Async SQL toolkit with compile-time checked queries
- **sea-orm** - Async ORM for advanced features
- **serde/serde_json** - Serialization and JSON handling
- **tokio** - Async runtime
- **anyhow/thiserror** - Error handling

## Performance

- **Connection pooling** - Configurable pool sizes and timeouts
- **Prepared statements** - SQL injection protection and performance
- **Batch operations** - Efficient bulk processing
- **Lazy loading** - Optional relationship and computed field evaluation
- **Query optimization** - Database-specific optimizations

## Compatibility

100% API compatibility with DreamFactory PHP implementation:
- Same REST endpoints
- Identical request/response formats
- Compatible query parameters
- Matching error codes and messages
- Full feature parity

## Provider-Specific Features

### MySQL
- Engine information (InnoDB, MyISAM, etc.)
- Index and foreign key introspection
- MySQL-specific data types
- SHOW commands support
- System variables access

### PostgreSQL
- Extension information
- Schema management
- Array and JSON support
- Full-text search capabilities
- Database and table size information

### SQLite
- PRAGMA commands
- In-memory database support
- File-based databases
- Compile options information
- Lightweight operation mode

## Future Enhancements

- **Additional providers** - MongoDB, Redis, etc.
- **Query caching** - Intelligent query result caching
- **Real-time sync** - Change streams and notifications
- **Advanced analytics** - Query performance monitoring
- **Migration tools** - Schema versioning and migrations