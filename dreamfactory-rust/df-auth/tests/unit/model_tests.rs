use df_auth::*;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_user_creation() {
    let email = "test@example.com".to_string();
    let password_hash = "hashed_password".to_string();
    let user = User::new(
        email.clone(),
        password_hash.clone(),
        Some("testuser".to_string()),
        Some("Test".to_string()),
        Some("User".to_string()),
        None,
    );

    assert_eq!(user.email, email);
    assert_eq!(user.password_hash, password_hash);
    assert_eq!(user.username, Some("testuser".to_string()));
    assert_eq!(user.first_name, Some("Test".to_string()));
    assert_eq!(user.last_name, Some("User".to_string()));
    assert!(user.is_active);
    assert!(!user.is_verified);
    assert_eq!(user.login_attempts, 0);
    assert!(!user.is_locked());
}

#[test]
fn test_user_locking() {
    let mut user = User::new(
        "test@example.com".to_string(),
        "hash".to_string(),
        None,
        None,
        None,
        None,
    );

    assert!(!user.is_locked());
    
    // Lock user for 1 hour
    user.locked_until = Some(Utc::now() + chrono::Duration::hours(1));
    assert!(user.is_locked());
    
    // Unlock user
    user.locked_until = Some(Utc::now() - chrono::Duration::hours(1));
    assert!(!user.is_locked());
}

#[test]
fn test_user_to_profile() {
    let user = User::new(
        "profile@example.com".to_string(),
        "hash".to_string(),
        Some("profileuser".to_string()),
        Some("Profile".to_string()),
        Some("User".to_string()),
        None,
    );

    let roles = vec!["user".to_string(), "admin".to_string()];
    let profile = user.to_profile(roles.clone());

    assert_eq!(profile.email, user.email);
    assert_eq!(profile.username, user.username);
    assert_eq!(profile.first_name, user.first_name);
    assert_eq!(profile.last_name, user.last_name);
    assert_eq!(profile.roles, roles);
    assert_eq!(profile.id, user.id);
}

#[test]
fn test_role_creation() {
    let name = "Test Role".to_string();
    let description = Some("Test role description".to_string());
    let created_by_id = Some(Uuid::new_v4());
    
    let role = Role::new(name.clone(), description.clone(), created_by_id);

    assert_eq!(role.name, name);
    assert_eq!(role.description, description);
    assert_eq!(role.created_by_id, created_by_id);
    assert!(role.is_active);
}

#[test]
fn test_permission_creation() {
    let permission = Permission::new(
        "Test Permission".to_string(),
        Some("Test description".to_string()),
        "user".to_string(),
        "read".to_string(),
        None,
    );

    assert_eq!(permission.name, "Test Permission");
    assert_eq!(permission.resource, "user");
    assert_eq!(permission.action, "read");
    assert!(permission.is_active);
}

#[test]
fn test_permission_matching() {
    let permission = Permission::new(
        "User Read".to_string(),
        None,
        "user".to_string(),
        "read".to_string(),
        None,
    );

    assert!(permission.matches("user", "read"));
    assert!(!permission.matches("user", "write"));
    assert!(!permission.matches("admin", "read"));
}

#[test]
fn test_wildcard_permission_matching() {
    let admin_permission = Permission::new(
        "Admin All".to_string(),
        None,
        "*".to_string(),
        "*".to_string(),
        None,
    );

    assert!(admin_permission.matches("user", "read"));
    assert!(admin_permission.matches("user", "write"));
    assert!(admin_permission.matches("admin", "delete"));
    assert!(admin_permission.matches("anything", "action"));
}

#[test]
fn test_resource_wildcard_permission() {
    let user_all_permission = Permission::new(
        "User All".to_string(),
        None,
        "user".to_string(),
        "*".to_string(),
        None,
    );

    assert!(user_all_permission.matches("user", "read"));
    assert!(user_all_permission.matches("user", "write"));
    assert!(user_all_permission.matches("user", "delete"));
    assert!(!user_all_permission.matches("admin", "read"));
}

#[test]
fn test_session_creation() {
    let user_id = Uuid::new_v4();
    let token = "test_token".to_string();
    let refresh_token = "refresh_token".to_string();
    let jwt_expiration = 3600;
    let refresh_expiration = 7200;

    let session = Session::new(
        user_id,
        token.clone(),
        refresh_token.clone(),
        jwt_expiration,
        refresh_expiration,
        Some("192.168.1.1".to_string()),
        Some("Test Agent".to_string()),
    );

    assert_eq!(session.user_id, user_id);
    assert_eq!(session.token, token);
    assert_eq!(session.refresh_token, refresh_token);
    assert!(session.is_active);
    assert!(!session.is_expired());
    assert!(!session.is_refresh_expired());
}

#[test]
fn test_session_expiration() {
    let mut session = Session::new(
        Uuid::new_v4(),
        "token".to_string(),
        "refresh".to_string(),
        -1, // Expired
        3600,
        None,
        None,
    );

    assert!(session.is_expired());
    assert!(!session.is_refresh_expired());

    session.refresh_expires_at = Utc::now() - chrono::Duration::seconds(1);
    assert!(session.is_refresh_expired());
}

#[test]
fn test_api_key_creation() {
    let name = "Test API Key".to_string();
    let key_hash = "hashed_key".to_string();
    let user_id = Some(Uuid::new_v4());

    let api_key = ApiKey::new(
        name.clone(),
        key_hash.clone(),
        user_id,
        None,
        None,
        Some(100),
        Some(vec!["192.168.1.1".to_string()]),
        None,
    );

    assert_eq!(api_key.name, name);
    assert_eq!(api_key.key_hash, key_hash);
    assert_eq!(api_key.user_id, user_id);
    assert!(api_key.is_active);
    assert!(!api_key.is_expired());
    assert_eq!(api_key.rate_limit_per_minute, Some(100));
}

#[test]
fn test_api_key_ip_restrictions() {
    let allowed_ips = vec!["192.168.1.1".to_string(), "10.0.0.1".to_string()];
    let api_key = ApiKey::new(
        "Test Key".to_string(),
        "hash".to_string(),
        None,
        None,
        None,
        None,
        Some(allowed_ips.clone()),
        None,
    );

    assert!(api_key.is_ip_allowed("192.168.1.1"));
    assert!(api_key.is_ip_allowed("10.0.0.1"));
    assert!(!api_key.is_ip_allowed("172.16.0.1"));
    
    let retrieved_ips = api_key.get_allowed_ips();
    assert_eq!(retrieved_ips, allowed_ips);
}

#[test]
fn test_api_key_no_ip_restrictions() {
    let api_key = ApiKey::new(
        "Test Key".to_string(),
        "hash".to_string(),
        None,
        None,
        None,
        None,
        None, // No IP restrictions
        None,
    );

    assert!(api_key.is_ip_allowed("192.168.1.1"));
    assert!(api_key.is_ip_allowed("10.0.0.1"));
    assert!(api_key.is_ip_allowed("any.ip.address"));
}

#[test]
fn test_api_key_usage_tracking() {
    let mut api_key = ApiKey::new(
        "Test Key".to_string(),
        "hash".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(api_key.usage_count, 0);
    assert!(api_key.last_used_at.is_none());

    api_key.update_usage();
    assert_eq!(api_key.usage_count, 1);
    assert!(api_key.last_used_at.is_some());

    api_key.update_usage();
    assert_eq!(api_key.usage_count, 2);
}

#[test]
fn test_action_enum_conversion() {
    let create_action: Action = "create".to_string().into();
    assert!(matches!(create_action, Action::Create));

    let read_action: Action = "READ".to_string().into(); // Case insensitive
    assert!(matches!(read_action, Action::Read));

    let unknown_action: Action = "unknown".to_string().into();
    assert!(matches!(unknown_action, Action::Read)); // Default fallback

    assert_eq!(Action::Create.to_string(), "create");
    assert_eq!(Action::Delete.to_string(), "delete");
}

#[test]
fn test_standard_permissions() {
    let system_admin = Permission::system_admin();
    assert_eq!(system_admin.resource, "*");
    assert_eq!(system_admin.action, "*");

    let user_read = Permission::user_read();
    assert_eq!(user_read.resource, "user");
    assert_eq!(user_read.action, "read");

    let api_write = Permission::api_write();
    assert_eq!(api_write.resource, "api");
    assert_eq!(api_write.action, "write");
}

#[test]
fn test_user_role_creation() {
    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let created_by_id = Some(Uuid::new_v4());

    let user_role = UserRole::new(user_id, role_id, created_by_id);

    assert_eq!(user_role.user_id, user_id);
    assert_eq!(user_role.role_id, role_id);
    assert_eq!(user_role.created_by_id, created_by_id);
}

#[test]
fn test_role_permission_creation() {
    let role_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();
    let created_by_id = Some(Uuid::new_v4());

    let role_permission = RolePermission::new(role_id, permission_id, created_by_id);

    assert_eq!(role_permission.role_id, role_id);
    assert_eq!(role_permission.permission_id, permission_id);
    assert_eq!(role_permission.created_by_id, created_by_id);
}