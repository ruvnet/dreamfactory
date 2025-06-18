use crate::{AuthError, Result, Claims, AuthConfig};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    expiration: i64,
}

impl JwtService {
    pub fn new(config: &AuthConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = false;

        Self {
            encoding_key,
            decoding_key,
            validation,
            expiration: config.jwt_expiration,
        }
    }

    pub fn generate_token(
        &self,
        user_id: Uuid,
        email: String,
        session_id: Uuid,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Result<String> {
        let claims = Claims::new(
            user_id,
            email,
            session_id,
            roles,
            permissions,
            self.expiration,
        );

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(AuthError::Jwt)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })
    }

    pub fn refresh_token(
        &self,
        old_token: &str,
        new_session_id: Uuid,
    ) -> Result<String> {
        // Decode the old token (ignore expiration for refresh)
        let mut validation = self.validation.clone();
        validation.validate_exp = false;
        
        let old_claims = decode::<Claims>(old_token, &self.decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|_| AuthError::InvalidToken)?;

        // Generate new token with same user info but new session and expiration
        let user_id = Uuid::parse_str(&old_claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        self.generate_token(
            user_id,
            old_claims.email,
            new_session_id,
            old_claims.roles,
            old_claims.permissions,
        )
    }

    pub fn get_expiration(&self) -> i64 {
        self.expiration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthConfig;

    #[test]
    fn test_generate_and_validate_token() {
        let config = AuthConfig::default();
        let jwt_service = JwtService::new(&config);
        
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let roles = vec!["user".to_string()];
        let permissions = vec!["read".to_string()];

        // Generate token
        let token = jwt_service.generate_token(
            user_id,
            email.clone(),
            session_id,
            roles.clone(),
            permissions.clone(),
        ).unwrap();

        // Validate token
        let claims = jwt_service.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.jti, session_id.to_string());
        assert_eq!(claims.roles, roles);
        assert_eq!(claims.permissions, permissions);
    }

    #[test]
    fn test_invalid_token() {
        let config = AuthConfig::default();
        let jwt_service = JwtService::new(&config);
        
        let result = jwt_service.validate_token("invalid.token.here");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
    }

    #[test]
    fn test_expired_token() {
        let mut config = AuthConfig::default();
        config.jwt_expiration = -1; // Expired immediately
        let jwt_service = JwtService::new(&config);
        
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let roles = vec!["user".to_string()];
        let permissions = vec!["read".to_string()];

        let token = jwt_service.generate_token(
            user_id,
            email,
            session_id,
            roles,
            permissions,
        ).unwrap();

        // Wait a moment to ensure expiration
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let result = jwt_service.validate_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::TokenExpired));
    }
}