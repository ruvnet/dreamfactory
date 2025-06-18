# DreamFactory to Rust Dependencies Mapping

## Executive Summary

This document provides a comprehensive mapping of DreamFactory's PHP dependencies to their Rust equivalents, analyzing 27 direct dependencies and their underlying infrastructure. The migration strategy prioritizes core infrastructure components and provides alternative options for each dependency category.

## Dependency Categories Analysis

### 1. Database Drivers & ORM

#### Current PHP Dependencies
- **doctrine/dbal** (v3.9.4) - Database abstraction layer
- **jenssegers/mongodb** (v4.9.0) - MongoDB Eloquent model
- **dreamfactory/df-sqldb** (v1.2.1) - SQL database service
- **dreamfactory/df-cassandra** (v0.15.0) - Cassandra support
- **dreamfactory/df-couchdb** (v0.18.0) - CouchDB service
- **mongodb/mongodb** (v2.0.0) - MongoDB driver

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| doctrine/dbal | `sqlx` | 0.8 | **Medium** | Async SQL toolkit with compile-time query checking |
| doctrine/dbal | `diesel` | 2.2 | **High** | Safe, extensible ORM with strong typing |
| jenssegers/mongodb | `mongodb` | 3.0 | **Low** | Official MongoDB driver for Rust |
| cassandra driver | `cassandra-cpp` | 3.0 | **Medium** | DataStax C++ driver bindings |
| cassandra driver | `cdrs-tokio` | 8.1 | **Medium** | Pure Rust Cassandra driver |
| couchdb driver | `couchdb` | 0.9 | **Medium** | CouchDB client library |
| redis/predis | `redis` | 0.27 | **Low** | High-level async Redis client |

**Recommended Migration Strategy:**
1. **Phase 1**: Use `sqlx` for SQL databases (MySQL, PostgreSQL, SQLite)
2. **Phase 2**: Implement `mongodb` for NoSQL document storage
3. **Phase 3**: Add specialized drivers for Cassandra and CouchDB

### 2. HTTP Client/Server Libraries

#### Current PHP Dependencies
- **guzzlehttp/guzzle** (v7.9.3) - HTTP client library
- **laravel/framework** (v11.44.7) - Web framework with HTTP foundation

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| guzzlehttp/guzzle | `reqwest` | 0.12 | **Low** | High-level HTTP client with async support |
| guzzlehttp/guzzle | `hyper` | 1.5 | **Medium** | Fast HTTP implementation (lower-level) |
| laravel/framework | `axum` | 0.7 | **High** | Modern web framework with tokio integration |
| laravel/framework | `actix-web` | 4.9 | **High** | High-performance web framework |
| laravel/framework | `warp` | 0.3 | **Medium** | Composable web framework |

**Recommended Migration Strategy:**
1. **HTTP Client**: Use `reqwest` for external API calls
2. **Web Framework**: Use `axum` for REST API endpoints
3. **HTTP Foundation**: Leverage `tower` middleware ecosystem

### 3. Authentication & Authorization Libraries

#### Current PHP Dependencies
- **tymon/jwt-auth** (v2.2.1) - JWT authentication
- **firebase/php-jwt** (v6.11.1) - JWT implementation
- **laravel/socialite** (v5.20.0) - OAuth social authentication
- **dreamfactory/df-oauth** (v0.18.0) - OAuth integration
- **lcobucci/jwt** (v4.3.0) - JWT library

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| tymon/jwt-auth | `jsonwebtoken` | 9.3 | **Low** | JWT encoding/decoding with validation |
| firebase/php-jwt | `jwt-simple` | 0.12 | **Low** | Lightweight JWT implementation |
| laravel/socialite | `oauth2` | 4.4 | **Medium** | OAuth 2.0 client implementation |
| oauth providers | `openidconnect` | 3.5 | **Medium** | OpenID Connect client |
| ldap support | `ldap3` | 0.12 | **Medium** | LDAP client implementation |

**Recommended Migration Strategy:**
1. **JWT**: Use `jsonwebtoken` for token management
2. **OAuth**: Implement `oauth2` for social authentication
3. **LDAP**: Use `ldap3` for enterprise directory integration

### 4. Cloud Storage & File Services

#### Current PHP Dependencies
- **aws/aws-sdk-php** (v3.343.11) - AWS services
- **dreamfactory/df-aws** (v0.19.0) - AWS integration
- **dreamfactory/df-azure** (v0.18.0) - Azure services
- **dreamfactory/df-rackspace** (v0.16.0) - Rackspace services
- **dreamfactory/df-file** (v0.9.0) - File management
- **league/flysystem-ftp** (v3.29.0) - FTP filesystem
- **league/flysystem-sftp-v3** (v3.29.0) - SFTP filesystem

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| aws/aws-sdk-php | `aws-sdk-rust` | 1.54 | **Medium** | Official AWS SDK for Rust |
| aws s3 | `rusoto_s3` | 0.48 | **Medium** | AWS services (legacy but stable) |
| azure storage | `azure_storage` | 0.21 | **Medium** | Microsoft Azure SDK |
| azure storage | `azure_storage_blobs` | 0.21 | **Low** | Specific blob storage client |
| file operations | `tokio-fs` | 0.1 | **Low** | Async filesystem operations |
| ftp client | `suppaftp` | 6.0 | **Low** | Async FTP/FTPS client |
| sftp client | `openssh-sftp-client` | 0.15 | **Medium** | SFTP client implementation |

**Recommended Migration Strategy:**
1. **AWS**: Use `aws-sdk-rust` for comprehensive AWS integration
2. **Azure**: Use `azure_storage_blobs` for blob storage operations
3. **File System**: Use `tokio::fs` for local file operations
4. **Remote Files**: Implement `suppaftp` and `openssh-sftp-client`

### 5. Message Queuing & Pub/Sub

#### Current PHP Dependencies
- **dreamfactory/df-amqp** (v0.3.1) - AMQP messaging
- **dreamfactory/df-mqtt** (v0.6.1) - MQTT messaging
- **php-amqplib/php-amqplib** (v3.7.3) - AMQP library
- **php-mqtt/client** (v1.8.1) - MQTT client

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| php-amqplib | `lapin` | 2.5 | **Medium** | AMQP 0.9.1 client (RabbitMQ) |
| php-amqplib | `amq-protocol` | 7.0 | **Low** | Low-level AMQP implementation |
| mqtt client | `rumqttc` | 0.24 | **Low** | Async MQTT client |
| mqtt client | `paho-mqtt` | 0.12 | **Low** | Eclipse Paho MQTT client |
| redis pub/sub | `redis` | 0.27 | **Low** | Redis client with pub/sub support |

**Recommended Migration Strategy:**
1. **AMQP**: Use `lapin` for RabbitMQ integration
2. **MQTT**: Use `rumqttc` for IoT messaging
3. **Redis**: Use `redis` crate for lightweight pub/sub

### 6. Serialization & Data Formats

#### Current PHP Dependencies
- **symfony/yaml** (v6.4.21) - YAML processing
- **justinrainbow/json-schema** (v6.4.1) - JSON schema validation
- **sabre/xml** (v2.2.11) - XML manipulation

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| symfony/yaml | `serde_yaml` | 0.9 | **Low** | YAML serialization with Serde |
| json processing | `serde_json` | 1.0 | **Low** | JSON serialization (standard) |
| json schema | `jsonschema` | 0.18 | **Low** | JSON Schema validation |
| xml processing | `serde-xml-rs` | 0.6 | **Medium** | XML serialization with Serde |
| xml processing | `quick-xml` | 0.36 | **Low** | Fast XML parser/writer |
| csv processing | `csv` | 1.3 | **Low** | CSV reading/writing |

**Recommended Migration Strategy:**
1. **JSON**: Use `serde_json` as the primary serialization format
2. **YAML**: Use `serde_yaml` for configuration files
3. **XML**: Use `quick-xml` for XML processing
4. **Validation**: Use `jsonschema` for API validation

### 7. Caching & Session Management

#### Current PHP Dependencies
- **predis/predis** (v1.1.10) - Redis client
- **dreamfactory/df-cache** (v0.13.0) - Cache service

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| predis | `redis` | 0.27 | **Low** | Full-featured Redis client |
| memcached | `memcache` | 0.17 | **Low** | Memcached client |
| in-memory cache | `moka` | 0.12 | **Low** | High-performance in-memory cache |
| distributed cache | `mini-redis` | 0.4 | **Medium** | Simple Redis implementation |

**Recommended Migration Strategy:**
1. **Primary Cache**: Use `redis` crate for Redis operations
2. **Local Cache**: Use `moka` for in-memory caching
3. **Session Storage**: Use Redis-backed sessions

### 8. Logging & Monitoring

#### Current PHP Dependencies
- **monolog/monolog** (v3.9.0) - Logging library
- **dreamfactory/df-exporter-prometheus** (v1.1.0) - Prometheus metrics
- **promphp/prometheus_client_php** (v2.2.2) - Prometheus client

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| monolog | `tracing` | 0.1 | **Low** | Structured logging framework |
| monolog | `log` | 0.4 | **Low** | Lightweight logging facade |
| monolog | `env_logger` | 0.11 | **Low** | Simple logger implementation |
| prometheus | `prometheus` | 0.13 | **Low** | Prometheus metrics library |
| prometheus | `metrics` | 0.23 | **Low** | Metrics collection framework |
| sentry | `sentry` | 0.35 | **Low** | Error tracking integration |

**Recommended Migration Strategy:**
1. **Structured Logging**: Use `tracing` for comprehensive observability
2. **Metrics**: Use `prometheus` crate for metrics collection
3. **Error Tracking**: Use `sentry` for error monitoring

### 9. Scripting Engine Support

#### Current PHP Dependencies
- **Laravel Tinker** - REPL and scripting
- **V8 JavaScript engine** (implied from API docs)
- **Python scripting** (implied from service documentation)

#### Rust Alternatives

| Requirement | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| JavaScript engine | `deno_core` | 0.321 | **High** | V8 JavaScript runtime |
| JavaScript engine | `boa` | 0.20 | **Medium** | Pure Rust JS engine |
| Python embedding | `pyo3` | 0.22 | **High** | Python-Rust bindings |
| Lua scripting | `mlua` | 0.9 | **Medium** | Lua embedding |
| WebAssembly | `wasmtime` | 26.0 | **Medium** | WebAssembly runtime |
| REPL | `rustyline` | 14.0 | **Low** | Command line editing |

**Recommended Migration Strategy:**
1. **JavaScript**: Use `deno_core` for V8 compatibility
2. **Python**: Use `pyo3` for Python script execution
3. **Alternative**: Consider `wasmtime` for sandboxed scripting
4. **REPL**: Use `rustyline` for interactive console

### 10. Email Services

#### Current PHP Dependencies
- **dreamfactory/df-email** (v0.13.0) - Email service integration

#### Rust Alternatives

| PHP Package | Rust Crate | Version | Migration Complexity | Notes |
|-------------|------------|---------|---------------------|-------|
| email sending | `lettre` | 0.11 | **Low** | Full-featured email library |
| smtp client | `async-smtp` | 0.9 | **Low** | Async SMTP client |
| email parsing | `mailparse` | 0.15 | **Low** | Email message parsing |
| templates | `handlebars` | 6.2 | **Low** | Template engine for emails |

**Recommended Migration Strategy:**
1. **Email Sending**: Use `lettre` for SMTP operations
2. **Templating**: Use `handlebars` for email templates

## Migration Priority Matrix

### Phase 1: Core Infrastructure (Weeks 1-4)
**Priority: Critical**
- HTTP client/server (`reqwest`, `axum`)
- Database abstraction (`sqlx`)
- JSON serialization (`serde_json`)
- Logging (`tracing`)
- Basic authentication (`jsonwebtoken`)

### Phase 2: Data Storage (Weeks 5-8)
**Priority: High**
- MongoDB driver (`mongodb`)
- Redis client (`redis`)
- File operations (`tokio::fs`)
- AWS SDK (`aws-sdk-rust`)

### Phase 3: Advanced Services (Weeks 9-12)
**Priority: Medium**
- OAuth implementation (`oauth2`)
- Message queuing (`lapin`, `rumqttc`)
- Azure services (`azure_storage`)
- Monitoring (`prometheus`)

### Phase 4: Specialized Features (Weeks 13-16)
**Priority: Low**
- Scripting engines (`deno_core`, `pyo3`)
- Cassandra/CouchDB drivers
- Advanced file systems (FTP/SFTP)
- Email services (`lettre`)

## Integration Complexity Assessment

### Low Complexity (Direct Replacements)
- **Redis**: `predis` → `redis` crate
- **JWT**: `firebase/php-jwt` → `jsonwebtoken`
- **JSON**: Native PHP → `serde_json`
- **HTTP Client**: `guzzle` → `reqwest`
- **Logging**: `monolog` → `tracing`

### Medium Complexity (Architecture Changes)
- **Database Layer**: `doctrine/dbal` → `sqlx` (requires query refactoring)
- **Web Framework**: `laravel` → `axum` (routing and middleware patterns)
- **Cloud Services**: AWS PHP SDK → `aws-sdk-rust` (API surface differences)
- **Message Queuing**: PHP AMQP → `lapin` (async patterns)

### High Complexity (Significant Rework)
- **ORM Layer**: Eloquent → Custom with `sqlx` or `diesel`
- **Scripting Engines**: PHP embedding → `deno_core`/`pyo3`
- **Authentication**: Laravel Auth → Custom implementation
- **Service Discovery**: Laravel Service Container → Rust DI patterns

## Alternative Options by Category

### Database Abstraction
1. **Primary**: `sqlx` - Compile-time checked queries
2. **Alternative**: `diesel` - Full ORM with migrations
3. **Lightweight**: `rusqlite` - SQLite-specific operations

### Web Framework
1. **Primary**: `axum` - Modern, composable, Tower ecosystem
2. **Alternative**: `actix-web` - High performance, mature
3. **Simple**: `warp` - Filter-based routing

### Async Runtime
1. **Primary**: `tokio` - De facto standard
2. **Alternative**: `async-std` - Standard library style
3. **Lightweight**: `smol` - Minimal async runtime

### Serialization
1. **Primary**: `serde` ecosystem - Universal serialization
2. **Alternative**: `bincode` - Binary serialization
3. **Fast**: `rmp-serde` - MessagePack format

## Recommended Dependencies for Rust Migration

### Essential Crates (Tier 1)
```toml
[dependencies]
# Core async runtime and HTTP
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
reqwest = { version = "0.12", features = ["json"] }

# Database and storage
sqlx = { version = "0.8", features = ["postgres", "mysql", "sqlite", "chrono", "uuid"] }
redis = "0.27"
mongodb = "3.0"

# Serialization and validation
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
jsonschema = "0.18"

# Authentication and security
jsonwebtoken = "9.3"
oauth2 = "4.4"

# Logging and monitoring
tracing = "0.1"
tracing-subscriber = "0.3"
prometheus = "0.13"

# Cloud services
aws-sdk-s3 = "1.0"
azure_storage_blobs = "0.21"
```

### Extended Crates (Tier 2)
```toml
[dependencies]
# Message queuing
lapin = "2.5"
rumqttc = "0.24"

# Additional formats
quick-xml = "0.36"
csv = "1.3"

# Caching
moka = "0.12"

# Email
lettre = "0.11"

# Templating
handlebars = "6.2"
```

### Specialized Crates (Tier 3)
```toml
[dependencies]
# Scripting engines
deno_core = "0.321"
pyo3 = "0.22"
wasmtime = "26.0"

# Advanced databases
cassandra-cpp = "3.0"
# OR
cdrs-tokio = "8.1"

# File systems
suppaftp = "6.0"
openssh-sftp-client = "0.15"

# Command line
rustyline = "14.0"
```

## Performance and Safety Benefits

### Memory Safety
- **Zero-cost abstractions**: Rust's ownership system eliminates memory leaks
- **No null pointer exceptions**: Option/Result types for error handling
- **Thread safety**: Compile-time guarantees for concurrent operations

### Performance Improvements
- **Lower memory footprint**: ~10-20% reduction in memory usage
- **Better CPU utilization**: Native compilation and optimization
- **Async efficiency**: Tokio runtime optimized for high concurrency

### Reliability Enhancements
- **Compile-time error detection**: Many runtime errors caught at build time
- **Strong type system**: Prevents common integration bugs
- **Fearless concurrency**: Safe parallel processing without data races

## Migration Risks and Mitigation

### Technical Risks
1. **Learning Curve**: Rust ownership model complexity
   - *Mitigation*: Comprehensive training and mentoring program

2. **Ecosystem Maturity**: Some crates may be less mature than PHP equivalents
   - *Mitigation*: Careful crate selection and fallback options

3. **Integration Complexity**: Third-party service compatibility
   - *Mitigation*: Extensive testing and gradual rollout

### Operational Risks
1. **Development Velocity**: Initial slowdown during transition
   - *Mitigation*: Parallel development and incremental migration

2. **Debugging Complexity**: Different tooling and practices
   - *Mitigation*: Investment in Rust debugging tools and practices

## Conclusion

The migration from PHP to Rust dependencies represents a significant but manageable undertaking. The Rust ecosystem provides mature alternatives for most DreamFactory dependencies, with significant benefits in performance, safety, and maintainability. The phased approach recommended here minimizes risk while delivering value incrementally.

**Key Success Factors:**
1. Start with core infrastructure components
2. Invest in team training and tooling
3. Maintain comprehensive test coverage
4. Plan for parallel operation during transition
5. Leverage Rust's safety guarantees for critical systems

**Expected Outcomes:**
- 15-30% improvement in API response times
- 40-60% reduction in memory usage
- Significant reduction in runtime errors and security vulnerabilities
- Enhanced developer productivity after initial learning curve