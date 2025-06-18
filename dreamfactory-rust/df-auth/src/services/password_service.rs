use crate::{AuthError, Result, AuthConfig};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

#[derive(Clone)]
pub struct PasswordService {
    argon2: Argon2<'static>,
    pepper: String,
}

impl PasswordService {
    pub fn new(config: &AuthConfig) -> Self {
        Self {
            argon2: Argon2::default(),
            pepper: config.password_pepper.clone(),
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        // Add pepper to password for additional security
        let peppered_password = format!("{}{}", password, self.pepper);
        
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(peppered_password.as_bytes(), &salt)
            .map_err(AuthError::PasswordHash)?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        // Add pepper to password for verification
        let peppered_password = format!("{}{}", password, self.pepper);
        
        let parsed_hash = PasswordHash::new(hash)
            .map_err(AuthError::PasswordHash)?;

        Ok(self.argon2
            .verify_password(peppered_password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn validate_password_strength(&self, password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(AuthError::Validation(
                "Password must be at least 8 characters long".to_string()
            ));
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if !has_uppercase {
            return Err(AuthError::Validation(
                "Password must contain at least one uppercase letter".to_string()
            ));
        }

        if !has_lowercase {
            return Err(AuthError::Validation(
                "Password must contain at least one lowercase letter".to_string()
            ));
        }

        if !has_digit {
            return Err(AuthError::Validation(
                "Password must contain at least one digit".to_string()
            ));
        }

        if !has_special {
            return Err(AuthError::Validation(
                "Password must contain at least one special character".to_string()
            ));
        }

        Ok(())
    }

    pub fn generate_api_key(&self, length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                 abcdefghijklmnopqrstuvwxyz\
                                 0123456789";
        
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    pub fn hash_api_key(&self, api_key: &str) -> Result<String> {
        // Use same hashing mechanism as passwords but without pepper
        let salt = SaltString::generate(&mut OsRng);
        let key_hash = self.argon2
            .hash_password(api_key.as_bytes(), &salt)
            .map_err(AuthError::PasswordHash)?;

        Ok(key_hash.to_string())
    }

    pub fn verify_api_key(&self, api_key: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(AuthError::PasswordHash)?;

        Ok(self.argon2
            .verify_password(api_key.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthConfig;

    #[test]
    fn test_password_hashing_and_verification() {
        let config = AuthConfig::default();
        let password_service = PasswordService::new(&config);
        
        let password = "TestPassword123!";
        let hash = password_service.hash_password(password).unwrap();
        
        assert!(password_service.verify_password(password, &hash).unwrap());
        assert!(!password_service.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_strength_validation() {
        let config = AuthConfig::default();
        let password_service = PasswordService::new(&config);
        
        // Valid password
        assert!(password_service.validate_password_strength("TestPass123!").is_ok());
        
        // Too short
        assert!(password_service.validate_password_strength("Test1!").is_err());
        
        // No uppercase
        assert!(password_service.validate_password_strength("testpass123!").is_err());
        
        // No lowercase
        assert!(password_service.validate_password_strength("TESTPASS123!").is_err());
        
        // No digit
        assert!(password_service.validate_password_strength("TestPassword!").is_err());
        
        // No special character
        assert!(password_service.validate_password_strength("TestPassword123").is_err());
    }

    #[test]
    fn test_api_key_generation() {
        let config = AuthConfig::default();
        let password_service = PasswordService::new(&config);
        
        let api_key = password_service.generate_api_key(32);
        assert_eq!(api_key.len(), 32);
        
        // Verify it contains only valid characters
        for c in api_key.chars() {
            assert!(c.is_alphanumeric());
        }
    }

    #[test]
    fn test_api_key_hashing_and_verification() {
        let config = AuthConfig::default();
        let password_service = PasswordService::new(&config);
        
        let api_key = "test_api_key_123";
        let hash = password_service.hash_api_key(api_key).unwrap();
        
        assert!(password_service.verify_api_key(api_key, &hash).unwrap());
        assert!(!password_service.verify_api_key("wrong_key", &hash).unwrap());
    }
}