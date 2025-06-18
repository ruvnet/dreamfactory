use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // Subject (user ID)
    pub email: String,   // User email
    pub exp: usize,      // Expiration time
    pub iat: usize,      // Issued at
    pub jti: String,     // JWT ID (session ID)
    pub roles: Vec<String>, // User roles
    pub permissions: Vec<String>, // User permissions
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub email: String,
    pub session_id: Uuid,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub session_token: String,
    pub session_id: String,
    pub user_id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

impl Claims {
    pub fn new(
        user_id: Uuid,
        email: String,
        session_id: Uuid,
        roles: Vec<String>,
        permissions: Vec<String>,
        expiration: i64,
    ) -> Self {
        let now = Utc::now().timestamp() as usize;
        Self {
            sub: user_id.to_string(),
            email,
            exp: now + expiration as usize,
            iat: now,
            jti: session_id.to_string(),
            roles,
            permissions,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as usize;
        now >= self.exp
    }
}

impl AuthContext {
    pub fn new(
        user_id: Uuid,
        email: String,
        session_id: Uuid,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Self {
        let is_admin = roles.iter().any(|role| {
            role.to_lowercase().contains("admin") || role.to_lowercase().contains("super")
        }) || permissions.iter().any(|perm| {
            perm == "*" || perm.contains("admin")
        });

        Self {
            user_id,
            email,
            session_id,
            roles,
            permissions,
            is_admin,
        }
    }

    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        // Admin has all permissions
        if self.is_admin {
            return true;
        }

        // Check for wildcard permissions
        if self.permissions.contains(&"*".to_string()) {
            return true;
        }

        // Check for specific resource permissions
        let resource_wildcard = format!("{}.*", resource);
        if self.permissions.contains(&resource_wildcard) {
            return true;
        }

        // Check for specific action permissions
        let specific_permission = format!("{}.{}", resource, action);
        self.permissions.contains(&specific_permission)
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    pub fn from_claims(claims: &Claims) -> Result<Self, uuid::Error> {
        let user_id = Uuid::parse_str(&claims.sub)?;
        let session_id = Uuid::parse_str(&claims.jti)?;
        
        Ok(Self::new(
            user_id,
            claims.email.clone(),
            session_id,
            claims.roles.clone(),
            claims.permissions.clone(),
        ))
    }
}