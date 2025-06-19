pub mod jwt_service;
pub mod password_service;
pub mod user_service;
pub mod role_service;
pub mod permission_service;
pub mod session_service;
pub mod api_key_service;
pub mod auth_service;

use std::sync::Arc;

// Service exports
pub use jwt_service::JwtService;
pub use password_service::PasswordService;
pub use user_service::UserService;
pub use role_service::RoleService;
pub use permission_service::PermissionService;
pub use session_service::SessionService;
pub use api_key_service::ApiKeyService;
pub use auth_service::AuthService;

// Shared service state type aliases for dependency injection
pub type AuthServiceState = Arc<AuthService>;
pub type UserServiceState = Arc<UserService>;
pub type RoleServiceState = Arc<RoleService>;
pub type PermissionServiceState = Arc<PermissionService>;
pub type ApiKeyServiceState = Arc<ApiKeyService>;
pub type SessionServiceState = Arc<SessionService>;