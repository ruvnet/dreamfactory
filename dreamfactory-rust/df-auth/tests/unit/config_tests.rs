use df_auth::*;
use std::env;

#[test]
fn test_auth_config_default() {
    let config = AuthConfig::default();
    
    assert_eq!(config.jwt_expiration, 3600);
    assert_eq!(config.refresh_token_expiration, 604800);
    assert_eq!(config.api_key_length, 32);
    assert_eq!(config.session_timeout, 1800);
    assert_eq!(config.max_login_attempts, 5);
    assert_eq!(config.lockout_duration, 900);
}

#[test]
fn test_auth_config_validation_valid() {
    let mut config = AuthConfig::default();
    config.jwt_secret = "a".repeat(32); // 32 characters
    
    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_auth_config_validation_jwt_secret_too_short() {
    let mut config = AuthConfig::default();
    config.jwt_secret = "short".to_string(); // Less than 32 characters
    
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least 32 characters"));
}

#[test]
fn test_auth_config_validation_negative_expiration() {
    let mut config = AuthConfig::default();
    config.jwt_secret = "a".repeat(32);
    config.jwt_expiration = -1;
    
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("positive"));
}

#[test]
fn test_auth_config_validation_empty_pepper() {
    let mut config = AuthConfig::default();
    config.jwt_secret = "a".repeat(32);
    config.password_pepper = "".to_string();
    
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_auth_config_from_environment() {
    // Set environment variables
    env::set_var("JWT_SECRET", "test_secret_32_characters_long!");
    env::set_var("JWT_EXPIRATION", "7200");
    env::set_var("DATABASE_URL", "sqlite:test.db");
    env::set_var("PASSWORD_PEPPER", "test_pepper");
    env::set_var("API_KEY_LENGTH", "64");
    env::set_var("SESSION_TIMEOUT", "3600");
    env::set_var("MAX_LOGIN_ATTEMPTS", "3");
    env::set_var("LOCKOUT_DURATION", "1800");
    
    let config = AuthConfig::new();
    
    assert_eq!(config.jwt_secret, "test_secret_32_characters_long!");
    assert_eq!(config.jwt_expiration, 7200);
    assert_eq!(config.database_url, "sqlite:test.db");
    assert_eq!(config.password_pepper, "test_pepper");
    assert_eq!(config.api_key_length, 64);
    assert_eq!(config.session_timeout, 3600);
    assert_eq!(config.max_login_attempts, 3);
    assert_eq!(config.lockout_duration, 1800);
    
    // Clean up
    env::remove_var("JWT_SECRET");
    env::remove_var("JWT_EXPIRATION");
    env::remove_var("DATABASE_URL");
    env::remove_var("PASSWORD_PEPPER");
    env::remove_var("API_KEY_LENGTH");
    env::remove_var("SESSION_TIMEOUT");
    env::remove_var("MAX_LOGIN_ATTEMPTS");
    env::remove_var("LOCKOUT_DURATION");
}

#[test]
fn test_auth_config_invalid_env_values() {
    // Set invalid environment variables
    env::set_var("JWT_EXPIRATION", "not_a_number");
    env::set_var("API_KEY_LENGTH", "invalid");
    
    let config = AuthConfig::new();
    
    // Should fall back to defaults
    assert_eq!(config.jwt_expiration, 3600);
    assert_eq!(config.api_key_length, 32);
    
    // Clean up
    env::remove_var("JWT_EXPIRATION");
    env::remove_var("API_KEY_LENGTH");
}

#[test]
fn test_auth_config_clone() {
    let config1 = AuthConfig::default();
    let config2 = config1.clone();
    
    assert_eq!(config1.jwt_secret, config2.jwt_secret);
    assert_eq!(config1.jwt_expiration, config2.jwt_expiration);
    assert_eq!(config1.database_url, config2.database_url);
}

#[test]
fn test_auth_config_debug() {
    let config = AuthConfig::default();
    let debug_str = format!("{:?}", config);
    
    // Should contain field names
    assert!(debug_str.contains("jwt_expiration"));
    assert!(debug_str.contains("database_url"));
    // Should not contain sensitive data in plain text
    assert!(debug_str.contains("jwt_secret"));
}