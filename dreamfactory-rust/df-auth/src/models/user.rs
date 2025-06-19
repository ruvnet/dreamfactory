use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub last_login_date: Option<DateTime<Utc>>,
    pub created_date: DateTime<Utc>,
    pub last_modified_date: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
    pub last_modified_by_id: Option<Uuid>,
    pub login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub last_login_date: Option<DateTime<Utc>>,
    pub created_date: DateTime<Utc>,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    pub username: Option<String>,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    
    pub username: Option<String>,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: Option<String>,
    
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: Option<bool>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    
    #[validate(length(min = 8, message = "New password must be at least 8 characters long"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    pub username: Option<String>,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl User {
    pub fn new(
        email: String,
        password_hash: String,
        username: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        created_by_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            username,
            password_hash,
            first_name,
            last_name,
            is_active: true,
            is_verified: false,
            last_login_date: None,
            created_date: now,
            last_modified_date: now,
            created_by_id,
            last_modified_by_id: created_by_id,
            login_attempts: 0,
            locked_until: None,
        }
    }

    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    pub fn to_profile(&self, roles: Vec<String>) -> UserProfile {
        UserProfile {
            id: self.id,
            email: self.email.clone(),
            username: self.username.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            is_active: self.is_active,
            is_verified: self.is_verified,
            last_login_date: self.last_login_date,
            created_date: self.created_date,
            roles,
        }
    }
}