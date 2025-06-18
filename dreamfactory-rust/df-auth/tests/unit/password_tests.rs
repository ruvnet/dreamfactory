use df_auth::*;

#[test]
fn test_password_service_creation() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    // Test passes if no panic occurs
}

#[test]
fn test_password_hashing() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "TestPassword123!";
    let hash = password_service.hash_password(password);
    
    assert!(hash.is_ok());
    let hash_str = hash.unwrap();
    assert!(!hash_str.is_empty());
    assert_ne!(hash_str, password);
    assert!(hash_str.starts_with("$argon2"), "Should be Argon2 hash");
}

#[test]
fn test_password_verification_success() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "TestPassword123!";
    let hash = password_service.hash_password(password).unwrap();
    
    let is_valid = password_service.verify_password(password, &hash);
    assert!(is_valid.is_ok());
    assert!(is_valid.unwrap());
}

#[test]
fn test_password_verification_failure() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "TestPassword123!";
    let wrong_password = "WrongPassword123!";
    let hash = password_service.hash_password(password).unwrap();
    
    let is_valid = password_service.verify_password(wrong_password, &hash);
    assert!(is_valid.is_ok());
    assert!(!is_valid.unwrap());
}

#[test]
fn test_password_strength_validation_valid() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let strong_password = "StrongPassword123!";
    let result = password_service.validate_password_strength(strong_password);
    assert!(result.is_ok());
}

#[test]
fn test_password_strength_validation_too_short() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let short_password = "Short1!";
    let result = password_service.validate_password_strength(short_password);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least 8 characters"));
}

#[test]
fn test_password_strength_validation_no_uppercase() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "lowercase123!";
    let result = password_service.validate_password_strength(password);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("uppercase"));
}

#[test]
fn test_password_strength_validation_no_lowercase() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "UPPERCASE123!";
    let result = password_service.validate_password_strength(password);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("lowercase"));
}

#[test]
fn test_password_strength_validation_no_digit() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "NoDigitsHere!";
    let result = password_service.validate_password_strength(password);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("digit"));
}

#[test]
fn test_password_strength_validation_no_special() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "NoSpecialChars123";
    let result = password_service.validate_password_strength(password);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("special character"));
}

#[test]
fn test_api_key_generation() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let api_key = password_service.generate_api_key(32);
    assert_eq!(api_key.len(), 32);
    
    // Test that all characters are alphanumeric
    for c in api_key.chars() {
        assert!(c.is_alphanumeric());
    }
}

#[test]
fn test_api_key_generation_different_lengths() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let key_16 = password_service.generate_api_key(16);
    let key_64 = password_service.generate_api_key(64);
    
    assert_eq!(key_16.len(), 16);
    assert_eq!(key_64.len(), 64);
    assert_ne!(key_16, key_64);
}

#[test]
fn test_api_key_uniqueness() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let key1 = password_service.generate_api_key(32);
    let key2 = password_service.generate_api_key(32);
    
    assert_ne!(key1, key2);
}

#[test]
fn test_api_key_hashing() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let api_key = "test_api_key_123";
    let hash = password_service.hash_api_key(api_key);
    
    assert!(hash.is_ok());
    let hash_str = hash.unwrap();
    assert!(!hash_str.is_empty());
    assert_ne!(hash_str, api_key);
    assert!(hash_str.starts_with("$argon2"));
}

#[test]
fn test_api_key_verification_success() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let api_key = "test_api_key_123";
    let hash = password_service.hash_api_key(api_key).unwrap();
    
    let is_valid = password_service.verify_api_key(api_key, &hash);
    assert!(is_valid.is_ok());
    assert!(is_valid.unwrap());
}

#[test]
fn test_api_key_verification_failure() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let api_key = "test_api_key_123";
    let wrong_key = "wrong_api_key_456";
    let hash = password_service.hash_api_key(api_key).unwrap();
    
    let is_valid = password_service.verify_api_key(wrong_key, &hash);
    assert!(is_valid.is_ok());
    assert!(!is_valid.unwrap());
}

#[test]
fn test_password_with_pepper() {
    let mut config = AuthConfig::default();
    config.password_pepper = "test_pepper".to_string();
    let password_service = PasswordService::new(&config);
    
    let password = "TestPassword123!";
    let hash = password_service.hash_password(password).unwrap();
    
    // Should verify correctly with the same service (same pepper)
    assert!(password_service.verify_password(password, &hash).unwrap());
    
    // Should fail with different pepper
    let mut config2 = AuthConfig::default();
    config2.password_pepper = "different_pepper".to_string();
    let password_service2 = PasswordService::new(&config2);
    
    assert!(!password_service2.verify_password(password, &hash).unwrap());
}

#[test]
fn test_hash_consistency() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "ConsistentPassword123!";
    let hash1 = password_service.hash_password(password).unwrap();
    let hash2 = password_service.hash_password(password).unwrap();
    
    // Hashes should be different (due to salt) but both should verify
    assert_ne!(hash1, hash2);
    assert!(password_service.verify_password(password, &hash1).unwrap());
    assert!(password_service.verify_password(password, &hash2).unwrap());
}

#[test]
fn test_invalid_hash_format() {
    let config = AuthConfig::default();
    let password_service = PasswordService::new(&config);
    
    let password = "TestPassword123!";
    let invalid_hash = "not_a_valid_hash";
    
    let result = password_service.verify_password(password, invalid_hash);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::PasswordHash(_)));
}