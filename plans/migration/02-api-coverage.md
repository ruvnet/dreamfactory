# DreamFactory API Coverage & Functionality Mapping

## Overview
This document provides comprehensive coverage of all DreamFactory API endpoints, service types, and functionality to guide the Rust migration. Based on analysis of the DreamFactory core architecture, this covers all REST endpoints, request/response patterns, authentication requirements, and business logic.

## Core Architecture

### REST API Base Pattern
All DreamFactory APIs follow a consistent REST pattern:
```
/api/v{version}/{service}/{resource?}
```

**Base Route Structure:**
- **Root Endpoint**: `/api` (configurable via `df.api_route_prefix`)
- **Version Pattern**: `v[0-9.]+` (e.g., v2, v2.1)
- **Service Pattern**: `[_0-9a-zA-Z-.]+`
- **Resource Pattern**: `[0-9a-zA-ZÀ-ÿ-_@&\#\!=,:;\/\^\$\.\|\{\}\[\]\(\)\*\+\?\' ]+`

**HTTP Methods Supported:**
- `GET` - Retrieve resources
- `POST` - Create resources
- `PUT` - Update/replace resources
- `PATCH` - Partial update resources
- `DELETE` - Remove resources
- `OPTIONS` - CORS preflight

## Service Type Groups & Categories

### 1. System Services (`ServiceTypeGroups::SYSTEM`)

#### System Management Service
**Service Name**: `system`
**Base Path**: `/api/system`
**Description**: Core system administration and configuration

**Resources:**
- `admin/` - Administrative functions
- `app/` - Application management
- `cache/` - Cache management
- `constant/` - System constants
- `cors/` - CORS configuration
- `custom/` - Custom settings
- `email_template/` - Email templates
- `environment/` - Environment information
- `event/` - Event management
- `import/` - Data import functionality
- `lookup/` - Lookup values
- `package/` - Package management
- `password/` - Password management
- `profile/` - User profiles
- `role/` - Role management
- `service/` - Service configuration
- `service_type/` - Available service types
- `session/` - Session management

**Access Exceptions (Public Endpoints):**
- `admin/session` - All verbs (login/logout)
- `admin/password` - POST only (password reset)
- `environment` - GET only (system info)

### 2. Database Services (`ServiceTypeGroups::DATABASE`)

#### Base Database Pattern
**Service Types**: `sqldb`, `mysql`, `pgsql`, `sqlite`, `sql_server`, etc.
**Base Path**: `/api/{db_service}`

**Core Resources:**
- `_table/` - Table management and data operations
- `_schema/` - Database schema operations
- `_proc/` - Stored procedures
- `_func/` - Functions
- `{table_name}/` - Direct table access

**Database Table Operations:**
```
GET    /api/{db_service}/_table                 # List all tables
GET    /api/{db_service}/_table/{table}         # Get table schema
POST   /api/{db_service}/_table/{table}         # Create table
PUT    /api/{db_service}/_table/{table}         # Update table schema
DELETE /api/{db_service}/_table/{table}         # Drop table

GET    /api/{db_service}/{table}                # Query records
POST   /api/{db_service}/{table}                # Create records
PUT    /api/{db_service}/{table}                # Update/replace records
PATCH  /api/{db_service}/{table}                # Partial update records
DELETE /api/{db_service}/{table}                # Delete records

GET    /api/{db_service}/{table}/{id}           # Get specific record
PUT    /api/{db_service}/{table}/{id}           # Update specific record
PATCH  /api/{db_service}/{table}/{id}           # Patch specific record
DELETE /api/{db_service}/{table}/{id}           # Delete specific record
```

**Query Parameters (Database):**
- `fields` - Select specific fields
- `filter` - WHERE clause conditions
- `limit` - Maximum records to return
- `offset` - Starting record offset
- `order` - ORDER BY clause
- `group` - GROUP BY clause
- `having` - HAVING clause
- `related` - Include related records
- `include_count` - Include total count
- `include_schema` - Include table schema
- `ids` - Specific record IDs

### 3. File Services (`ServiceTypeGroups::FILE`)

#### File Service Types
- `local_file` - Local filesystem
- `ftp_file` - FTP protocol
- `sftp_file` - SFTP protocol
- `webdav_file` - WebDAV protocol
- `s3` - Amazon S3 (via df-aws)
- `azure_blob` - Azure Blob Storage (via df-azure)

**Base Path**: `/api/{file_service}`

**File Operations:**
```
GET    /api/{file_service}/                     # List root directory
GET    /api/{file_service}/{path}               # List directory or get file
POST   /api/{file_service}/{path}               # Create file/folder or upload
PUT    /api/{file_service}/{path}               # Update file content
PATCH  /api/{file_service}/{path}               # Update file properties
DELETE /api/{file_service}/{path}               # Delete file/folder
```

**File Query Parameters:**
- `include_files` - Include files in listing
- `include_folders` - Include folders in listing
- `full_tree` - Return full directory tree
- `zip` - Download as ZIP archive
- `extract` - Extract uploaded archive
- `url` - Return download URL
- `download` - Force download

### 4. User Services (`ServiceTypeGroups::USER`)

#### User Management Service
**Service Name**: `user`
**Base Path**: `/api/user`

**Resources:**
- `session/` - User authentication
- `password/` - Password operations
- `register/` - User registration
- `profile/` - User profile management

**User Operations:**
```
POST   /api/user/session                        # Login
DELETE /api/user/session                        # Logout
GET    /api/user/session                        # Get session info

POST   /api/user/password                       # Password reset request
PUT    /api/user/password                       # Change password

POST   /api/user/register                       # Register new user

GET    /api/user/profile                        # Get user profile
PUT    /api/user/profile                        # Update user profile
PATCH  /api/user/profile                        # Partial profile update
```

**Access Exceptions:**
- `session` - All verbs (public login/logout)
- `password` - POST only (public password reset)
- `register` - POST only (public registration)
- `profile` - GET, PUT, PATCH, DELETE (authenticated users)

### 5. Remote Services (`ServiceTypeGroups::REMOTE`)

#### Remote Web Service (RWS)
**Service Name**: `rws`
**Base Path**: `/api/{rws_service}`

**Purpose**: Proxy/wrapper for external HTTP APIs

**Operations:**
```
GET    /api/{rws_service}/{resource}            # GET to external API
POST   /api/{rws_service}/{resource}            # POST to external API
PUT    /api/{rws_service}/{resource}            # PUT to external API
PATCH  /api/{rws_service}/{resource}            # PATCH to external API
DELETE /api/{rws_service}/{resource}            # DELETE to external API
```

### 6. OAuth Services (`ServiceTypeGroups::OAUTH`)

#### OAuth Provider Services
**Service Types**: `oauth_facebook`, `oauth_google`, `oauth_github`, `oauth_twitter`, etc.
**Base Path**: `/api/{oauth_service}`

**OAuth Operations:**
```
GET    /api/{oauth_service}/authorize           # Begin OAuth flow
GET    /api/{oauth_service}/callback            # OAuth callback
POST   /api/{oauth_service}/token               # Token exchange
DELETE /api/{oauth_service}/token               # Revoke token
```

### 7. Email Services (`ServiceTypeGroups::EMAIL`)

#### Email Service Operations
**Base Path**: `/api/{email_service}`

**Email Operations:**
```
POST   /api/{email_service}/                    # Send email
GET    /api/{email_service}/template            # List templates
POST   /api/{email_service}/template            # Create template
```

### 8. Cache Services (`ServiceTypeGroups::CACHE`)

#### Cache Operations
**Base Path**: `/api/{cache_service}`

**Cache Operations:**
```
GET    /api/{cache_service}/{key}               # Get cached value
POST   /api/{cache_service}/{key}               # Set cache value
PUT    /api/{cache_service}/{key}               # Update cache value
DELETE /api/{cache_service}/{key}               # Delete cache value
DELETE /api/{cache_service}/                    # Clear all cache
```

### 9. Event Services (`ServiceTypeGroups::EVENT`)

#### Event/Messaging Operations
**Service Types**: `amqp`, `mqtt`, `pubsub`
**Base Path**: `/api/{event_service}`

**Event Operations:**
```
POST   /api/{event_service}/publish             # Publish message
GET    /api/{event_service}/subscribe           # Subscribe to events
```

### 10. Scripting Services (`ServiceTypeGroups::SCRIPT`)

#### Script Execution
**Base Path**: `/api/{script_service}`

**Script Operations:**
```
POST   /api/{script_service}/{script_name}      # Execute script
GET    /api/{script_service}/                   # List available scripts
```

## Authentication & Authorization

### Authentication Methods
1. **API Key** - Header: `X-DREAMFACTORY-API-KEY`
2. **Session Token (JWT)** - Header: `X-DREAMFACTORY-SESSION-TOKEN`
3. **Basic Auth** - Standard HTTP Basic Authentication
4. **OAuth** - Via OAuth service providers

### Permission Model
- **Verb Masks**: Bitwise permissions (GET=1, POST=2, PUT=4, PATCH=8, DELETE=16)
- **Resource-based**: Permissions apply to service/resource combinations
- **Role-based**: Users assigned to roles with specific permissions

### Access Control Patterns
```php
// Example permission check
Session::checkServicePermission($operation, $serviceName, $resource, $requestType);

// Verb mask examples
VerbsMask::NONE    = 0   // No access
VerbsMask::GET     = 1   // Read only
VerbsMask::POST    = 2   // Create only
VerbsMask::PUT     = 4   // Update only
VerbsMask::PATCH   = 8   // Partial update
VerbsMask::DELETE  = 16  // Delete only
VerbsMask::ALL     = 31  // Full access (1+2+4+8+16)
```

## Request/Response Patterns

### Standard Query Parameters
- `fields` - Select specific fields to return
- `related` - Include related records/resources
- `filter` - Filter conditions (WHERE clause for DB)
- `limit` - Maximum records to return (default: 1000)
- `offset` - Starting record offset for pagination
- `order` - Sort order specification
- `group` - Group by fields
- `ids` - Specific resource IDs to retrieve
- `include_count` - Include total count in response
- `include_schema` - Include schema information
- `file` - For file operations
- `url` - Return URLs for file resources

### Response Formats
**Success Response (200):**
```json
{
  "resource": [
    {
      "id": 1,
      "name": "example",
      "created_date": "2023-01-01T00:00:00Z"
    }
  ],
  "meta": {
    "count": 1,
    "schema": [...] // if include_schema=true
  }
}
```

**Error Response (4xx/5xx):**
```json
{
  "error": {
    "code": 400,
    "message": "Bad Request",
    "details": "Specific error details"
  }
}
```

### Batch Operations
Most services support batch operations via arrays:
```json
{
  "resource": [
    {"name": "item1", "value": "data1"},
    {"name": "item2", "value": "data2"}
  ]
}
```

## Service Configuration Patterns

### Configuration Structure
Each service has a configuration model extending `BaseServiceConfigModel`:
- Database connection parameters
- Authentication credentials
- Service-specific options
- Caching settings

### Example Database Service Config
```json
{
  "host": "localhost",
  "port": 3306,
  "database": "mydb",
  "username": "user",
  "password": "pass",
  "driver": "mysql",
  "charset": "utf8mb4",
  "options": {
    "ssl_enabled": false
  }
}
```

## Event System

### Event Types
- **PreProcessApiEvent** - Before request processing
- **PostProcessApiEvent** - After request processing
- **ServiceEvent** - Service-specific events
- **ServiceAssignedEvent** - When service is assigned
- **ServiceDeletedEvent** - When service is deleted

### Event Naming Convention
```
{service_name}.{resource_path}.{verb}.{timing}
```

Examples:
- `system.admin.session.post.pre_process`
- `mydb.users.get.post_process`
- `files.documents.delete.pre_process`

## Status & Health Endpoints

### System Status
```
GET /status                                     # System health check
```

**Response:**
```json
{
  "success": true,
  "version": "7.1.0",
  "timestamp": "2023-01-01T00:00:00Z",
  "services": {
    "database": "connected",
    "cache": "available"
  }
}
```

## Migration Implementation Requirements

### Rust Service Architecture
1. **Service Registry** - Dynamic service registration and discovery
2. **Request Router** - Route parsing and dispatch to services
3. **Authentication Middleware** - Handle all auth methods
4. **Permission Middleware** - Check resource-based permissions
5. **Event System** - Pre/post processing events
6. **Configuration Manager** - Service configuration handling

### Database Service Requirements
- **Connection Pooling** - Async database connections
- **Schema Introspection** - Dynamic table/column discovery
- **Query Builder** - SQL generation with filtering/pagination
- **Transaction Support** - ACID transaction handling
- **Batch Operations** - Multiple record CRUD

### File Service Requirements
- **Async I/O** - Non-blocking file operations
- **Stream Support** - Large file upload/download
- **Multiple Backends** - Local, S3, Azure, FTP/SFTP
- **Archive Support** - ZIP creation/extraction
- **Path Security** - Prevent directory traversal

### Test Coverage Strategy
Each service type requires comprehensive test coverage:

1. **Unit Tests**
   - Service configuration validation
   - Permission checking logic
   - Request/response serialization
   - Error handling

2. **Integration Tests**
   - End-to-end API workflows
   - Database operations with real connections
   - File operations with various backends
   - Authentication flows

3. **Performance Tests**
   - Concurrent request handling
   - Large dataset operations
   - File upload/download performance
   - Memory usage under load

### Security Considerations
- **Input Validation** - All inputs sanitized and validated
- **SQL Injection Prevention** - Parameterized queries only
- **Path Traversal Prevention** - File path validation
- **Rate Limiting** - API call rate limiting
- **CORS Support** - Configurable CORS policies
- **Audit Logging** - All API calls logged

## Implementation Priority

### Phase 1: Core Infrastructure
1. Request routing and service registry
2. Authentication and authorization middleware
3. Basic system service (environment, status)
4. Configuration management

### Phase 2: Database Services
1. Connection management and pooling
2. Schema introspection
3. Basic CRUD operations
4. Query filtering and pagination

### Phase 3: File Services
1. Local file system support
2. Basic file CRUD operations
3. Directory listing and navigation
4. Stream-based upload/download

### Phase 4: Extended Services
1. Remote web service proxy
2. Email service integration
3. Cache service implementation
4. OAuth provider integration

### Phase 5: Advanced Features
1. Event system and scripting
2. Advanced query capabilities
3. Batch operations optimization
4. Performance monitoring and analytics

This comprehensive API coverage document provides the foundation for implementing a complete DreamFactory-compatible REST API in Rust, maintaining full compatibility while leveraging Rust's performance and safety benefits.