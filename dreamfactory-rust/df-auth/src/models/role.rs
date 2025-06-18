use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_date: DateTime<Utc>,
    pub last_modified_date: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
    pub last_modified_by_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_date: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_date: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 255, message = "Role name must be between 1 and 255 characters"))]
    pub name: String,
    
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
    
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 255, message = "Role name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
    
    pub is_active: Option<bool>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

impl Role {
    pub fn new(
        name: String,
        description: Option<String>,
        created_by_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            is_active: true,
            created_date: now,
            last_modified_date: now,
            created_by_id,
            last_modified_by_id: created_by_id,
        }
    }
}

impl UserRole {
    pub fn new(user_id: Uuid, role_id: Uuid, created_by_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            role_id,
            created_date: Utc::now(),
            created_by_id,
        }
    }
}

impl RolePermission {
    pub fn new(role_id: Uuid, permission_id: Uuid, created_by_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role_id,
            permission_id,
            created_date: Utc::now(),
            created_by_id,
        }
    }
}