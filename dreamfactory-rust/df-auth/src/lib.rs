pub mod models;
pub mod services;
pub mod handlers;
pub mod middleware;
pub mod error;
pub mod config;

// Re-export core error types
pub use error::{AuthError, Result};

// Re-export config
pub use config::AuthConfig;

// Re-export models
pub use models::{
    // User types
    User, UserProfile, CreateUserRequest, UpdateUserRequest,
    // Role types
    Role, CreateRoleRequest, UpdateRoleRequest, UserRole, RolePermission,
    // Permission types
    Permission, CreatePermissionRequest, UpdatePermissionRequest, Action,
    // Session types
    Session, SessionResponse,
    // API Key types
    ApiKey, ApiKeyResponse, CreateApiKeyRequest, UpdateApiKeyRequest,
    // Auth types
    Claims, AuthContext, AuthResponse, LogoutResponse, LoginRequest, 
    RegisterRequest, RefreshTokenRequest, ChangePasswordRequest
};

// Re-export services
pub use services::{
    JwtService, PasswordService, UserService, RoleService, PermissionService,
    SessionService, ApiKeyService, AuthService,
    // Service state types for dependency injection
    AuthServiceState, UserServiceState, RoleServiceState, PermissionServiceState,
    ApiKeyServiceState, SessionServiceState
};

// Re-export handler functions
pub use handlers::{
    // Auth handlers
    login, logout, register, refresh_token, change_password, get_profile, health_check,
    // User handlers
    get_current_user_profile, update_current_user_profile, change_current_user_password,
    get_user_sessions, terminate_user_session, get_user_api_keys, create_user_api_key,
    update_user_api_key, delete_user_api_key,
    // Admin handlers
    admin_list_users, admin_create_user, admin_get_user, admin_update_user, admin_delete_user,
    admin_assign_role_to_user, admin_remove_role_from_user,
    admin_list_roles, admin_create_role, admin_get_role, admin_update_role, admin_delete_role,
    admin_list_permissions, admin_create_permission,
    admin_list_api_keys, admin_create_api_key, admin_get_api_key, admin_update_api_key, admin_delete_api_key,
    admin_list_sessions, admin_get_user_sessions, admin_terminate_session, admin_terminate_user_sessions,
    // Handler types
    ListQuery, ListResponse, AssignRoleRequest
};

// Re-export middleware
pub use middleware::{
    // Auth middleware
    auth_middleware, optional_auth_middleware,
    // RBAC middleware
    rbac_middleware, require_permission, require_admin, require_role,
    dynamic_rbac_middleware, require_ownership_or_admin,
    // API Key middleware
    api_key_middleware, optional_api_key_middleware, api_key_rate_limit_middleware,
    ApiKeyQuery
};