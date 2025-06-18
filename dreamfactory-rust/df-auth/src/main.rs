use axum::{
    middleware,
    routing::{get, post, put, delete},
    Router,
    Extension,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

use df_auth::{
    AuthConfig, AuthService, AuthServiceState,
    auth_handlers::*, auth_middleware, require_permission, require_admin,
    require_ownership_or_admin,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AuthConfig::new();
    config.validate().map_err(|e| format!("Configuration error: {}", e))?;

    // Setup database
    let db = setup_database(&config.database_url).await?;
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(db.clone(), config.clone()));

    // Initialize database schema
    run_migrations(&db).await?;

    // Build application
    let app = build_app(auth_service.clone()).await;

    // Start server
    let addr = "0.0.0.0:8080";
    info!("Starting DreamFactory Auth Server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn setup_database(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

async fn run_migrations(db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");

    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            username TEXT,
            password_hash TEXT NOT NULL,
            first_name TEXT,
            last_name TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            is_verified BOOLEAN NOT NULL DEFAULT FALSE,
            last_login_date TEXT,
            created_date TEXT NOT NULL,
            last_modified_date TEXT NOT NULL,
            created_by_id TEXT,
            last_modified_by_id TEXT,
            login_attempts INTEGER NOT NULL DEFAULT 0,
            locked_until TEXT
        )
        "#
    )
    .execute(db)
    .await?;

    // Create roles table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS roles (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_date TEXT NOT NULL,
            last_modified_date TEXT NOT NULL,
            created_by_id TEXT,
            last_modified_by_id TEXT
        )
        "#
    )
    .execute(db)
    .await?;

    // Create permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS permissions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            resource TEXT NOT NULL,
            action TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_date TEXT NOT NULL,
            last_modified_date TEXT NOT NULL,
            created_by_id TEXT,
            last_modified_by_id TEXT,
            UNIQUE(resource, action)
        )
        "#
    )
    .execute(db)
    .await?;

    // Create user_roles table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_roles (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            created_date TEXT NOT NULL,
            created_by_id TEXT,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
            FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
            UNIQUE(user_id, role_id)
        )
        "#
    )
    .execute(db)
    .await?;

    // Create role_permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS role_permissions (
            id TEXT PRIMARY KEY,
            role_id TEXT NOT NULL,
            permission_id TEXT NOT NULL,
            created_date TEXT NOT NULL,
            created_by_id TEXT,
            FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
            FOREIGN KEY (permission_id) REFERENCES permissions (id) ON DELETE CASCADE,
            UNIQUE(role_id, permission_id)
        )
        "#
    )
    .execute(db)
    .await?;

    // Create sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            refresh_expires_at TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            ip_address TEXT,
            user_agent TEXT,
            created_date TEXT NOT NULL,
            last_activity TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
        )
        "#
    )
    .execute(db)
    .await?;

    // Create api_keys table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            key_hash TEXT NOT NULL,
            user_id TEXT,
            role_id TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            expires_at TEXT,
            last_used_at TEXT,
            usage_count INTEGER NOT NULL DEFAULT 0,
            rate_limit_per_minute INTEGER,
            allowed_ips TEXT,
            created_date TEXT NOT NULL,
            last_modified_date TEXT NOT NULL,
            created_by_id TEXT,
            last_modified_by_id TEXT,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
            FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE SET NULL
        )
        "#
    )
    .execute(db)
    .await?;

    // Create indexes for performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(db)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)")
        .execute(db)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token)")
        .execute(db)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id)")
        .execute(db)
        .await?;

    info!("Database migrations completed successfully");
    Ok(())
}

async fn build_app(auth_service: AuthServiceState) -> Router {
    Router::new()
        // Health check endpoint (no auth required)
        .route("/health", get(health_check))
        
        // Public authentication endpoints
        .route("/api/v2/user/session", post(login))
        .route("/api/v2/user/register", post(register))
        .route("/api/v2/user/session/refresh", post(refresh_token))
        
        // Protected user endpoints
        .route("/api/v2/user/session", delete(logout))
        .route("/api/v2/user/profile", get(get_profile))
        .route("/api/v2/user/password", put(change_password))
        .layer(middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ))
        
        // Admin endpoints (require admin role)
        .nest("/api/v2/admin", admin_routes())
        .layer(middleware::from_fn(require_admin))
        .layer(middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ))
        
        // Global middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        
        // Application state
        .with_state(auth_service)
}

fn admin_routes() -> Router<AuthServiceState> {
    Router::new()
        // User management
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        
        // Role management
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/:id", get(get_role).put(update_role).delete(delete_role))
        .route("/roles/:id/permissions", get(get_role_permissions).put(update_role_permissions))
        .route("/roles/:id/users", get(get_role_users))
        
        // Permission management
        .route("/permissions", get(list_permissions).post(create_permission))
        .route("/permissions/:id", get(get_permission).put(update_permission).delete(delete_permission))
        
        // API key management
        .route("/api-keys", get(list_api_keys).post(create_api_key))
        .route("/api-keys/:id", get(get_api_key).put(update_api_key).delete(delete_api_key))
        
        // Session management
        .route("/sessions", get(list_sessions))
        .route("/sessions/:id", delete(terminate_session))
        .route("/users/:id/sessions", get(get_user_sessions).delete(terminate_user_sessions))
}

// Placeholder admin handlers - these would be implemented in a separate admin_handlers module
async fn list_users() -> &'static str { "List users endpoint" }
async fn create_user() -> &'static str { "Create user endpoint" }
async fn get_user() -> &'static str { "Get user endpoint" }
async fn update_user() -> &'static str { "Update user endpoint" }
async fn delete_user() -> &'static str { "Delete user endpoint" }

async fn list_roles() -> &'static str { "List roles endpoint" }
async fn create_role() -> &'static str { "Create role endpoint" }
async fn get_role() -> &'static str { "Get role endpoint" }
async fn update_role() -> &'static str { "Update role endpoint" }
async fn delete_role() -> &'static str { "Delete role endpoint" }
async fn get_role_permissions() -> &'static str { "Get role permissions endpoint" }
async fn update_role_permissions() -> &'static str { "Update role permissions endpoint" }
async fn get_role_users() -> &'static str { "Get role users endpoint" }

async fn list_permissions() -> &'static str { "List permissions endpoint" }
async fn create_permission() -> &'static str { "Create permission endpoint" }
async fn get_permission() -> &'static str { "Get permission endpoint" }
async fn update_permission() -> &'static str { "Update permission endpoint" }
async fn delete_permission() -> &'static str { "Delete permission endpoint" }

async fn list_api_keys() -> &'static str { "List API keys endpoint" }
async fn create_api_key() -> &'static str { "Create API key endpoint" }
async fn get_api_key() -> &'static str { "Get API key endpoint" }
async fn update_api_key() -> &'static str { "Update API key endpoint" }
async fn delete_api_key() -> &'static str { "Delete API key endpoint" }

async fn list_sessions() -> &'static str { "List sessions endpoint" }
async fn terminate_session() -> &'static str { "Terminate session endpoint" }
async fn get_user_sessions() -> &'static str { "Get user sessions endpoint" }
async fn terminate_user_sessions() -> &'static str { "Terminate user sessions endpoint" }