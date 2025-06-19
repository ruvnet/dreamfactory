pub mod auth_handlers;
pub mod user_handlers;
pub mod admin_handlers;

// Auth handler functions
pub use auth_handlers::{
    login, logout, register, refresh_token, change_password, get_profile, health_check
};

// User handler functions
pub use user_handlers::{
    get_current_user_profile, update_current_user_profile, change_current_user_password,
    get_user_sessions, terminate_user_session, get_user_api_keys, create_user_api_key,
    update_user_api_key, delete_user_api_key
};

// Admin handler functions and types
pub use admin_handlers::{
    // User management
    admin_list_users, admin_create_user, admin_get_user, admin_update_user, admin_delete_user,
    admin_assign_role_to_user, admin_remove_role_from_user,
    
    // Role management  
    admin_list_roles, admin_create_role, admin_get_role, admin_update_role, admin_delete_role,
    
    // Permission management
    admin_list_permissions, admin_create_permission,
    
    // API Key management
    admin_list_api_keys, admin_create_api_key, admin_get_api_key, admin_update_api_key, admin_delete_api_key,
    
    // Session management
    admin_list_sessions, admin_get_user_sessions, admin_terminate_session, admin_terminate_user_sessions,
    
    // Types
    ListQuery, ListResponse, AssignRoleRequest
};