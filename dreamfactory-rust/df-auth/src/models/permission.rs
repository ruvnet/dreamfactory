use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub is_active: bool,
    pub created_date: DateTime<Utc>,
    pub last_modified_date: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
    pub last_modified_by_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePermissionRequest {
    #[validate(length(min = 1, max = 255, message = "Permission name must be between 1 and 255 characters"))]
    pub name: String,
    
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
    
    #[validate(length(min = 1, max = 100, message = "Resource must be between 1 and 100 characters"))]
    pub resource: String,
    
    #[validate(length(min = 1, max = 100, message = "Action must be between 1 and 100 characters"))]
    pub action: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePermissionRequest {
    #[validate(length(min = 1, max = 255, message = "Permission name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
    
    #[validate(length(min = 1, max = 100, message = "Resource must be between 1 and 100 characters"))]
    pub resource: Option<String>,
    
    #[validate(length(min = 1, max = 100, message = "Action must be between 1 and 100 characters"))]
    pub action: Option<String>,
    
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    List,
    Execute,
    Admin,
}

impl From<String> for Action {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "create" => Action::Create,
            "read" => Action::Read,
            "update" => Action::Update,
            "delete" => Action::Delete,
            "list" => Action::List,
            "execute" => Action::Execute,
            "admin" => Action::Admin,
            _ => Action::Read, // Default fallback
        }
    }
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Action::Create => "create".to_string(),
            Action::Read => "read".to_string(),
            Action::Update => "update".to_string(),
            Action::Delete => "delete".to_string(),
            Action::List => "list".to_string(),
            Action::Execute => "execute".to_string(),
            Action::Admin => "admin".to_string(),
        }
    }
}

impl Permission {
    pub fn new(
        name: String,
        description: Option<String>,
        resource: String,
        action: String,
        created_by_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            resource,
            action,
            is_active: true,
            created_date: now,
            last_modified_date: now,
            created_by_id,
            last_modified_by_id: created_by_id,
        }
    }

    pub fn matches(&self, resource: &str, action: &str) -> bool {
        self.is_active && 
        (self.resource == "*" || self.resource == resource) &&
        (self.action == "*" || self.action == action)
    }
}

// Standard DreamFactory permissions
impl Permission {
    pub fn system_admin() -> Self {
        Permission::new(
            "System Admin".to_string(),
            Some("Full system administration access".to_string()),
            "*".to_string(),
            "*".to_string(),
            None,
        )
    }

    pub fn user_read() -> Self {
        Permission::new(
            "User Read".to_string(),
            Some("Read user information".to_string()),
            "user".to_string(),
            "read".to_string(),
            None,
        )
    }

    pub fn user_write() -> Self {
        Permission::new(
            "User Write".to_string(),
            Some("Create and update users".to_string()),
            "user".to_string(),
            "write".to_string(),
            None,
        )
    }

    pub fn api_read() -> Self {
        Permission::new(
            "API Read".to_string(),
            Some("Read API resources".to_string()),
            "api".to_string(),
            "read".to_string(),
            None,
        )
    }

    pub fn api_write() -> Self {
        Permission::new(
            "API Write".to_string(),
            Some("Create and update API resources".to_string()),
            "api".to_string(),
            "write".to_string(),
            None,
        )
    }
}