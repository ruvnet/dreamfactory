use crate::{
    AuthError, Result, Permission, CreatePermissionRequest, UpdatePermissionRequest
};
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct PermissionService {
    db: Pool<Sqlite>,
}

impl PermissionService {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    pub async fn create_permission(&self, request: CreatePermissionRequest, created_by_id: Option<Uuid>) -> Result<Permission> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        // Check if permission already exists
        if self.get_permission_by_resource_action(&request.resource, &request.action).await.is_ok() {
            return Err(AuthError::Validation("Permission already exists for this resource and action".to_string()));
        }

        let permission = Permission::new(
            request.name,
            request.description,
            request.resource,
            request.action,
            created_by_id,
        );

        // Insert permission
        sqlx::query(
            r#"
            INSERT INTO permissions (
                id, name, description, resource, action, is_active,
                created_date, last_modified_date, created_by_id, last_modified_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&permission.id)
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(&permission.resource)
        .bind(&permission.action)
        .bind(permission.is_active)
        .bind(permission.created_date)
        .bind(permission.last_modified_date)
        .bind(&permission.created_by_id)
        .bind(&permission.last_modified_by_id)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(permission)
    }

    pub async fn get_permission_by_id(&self, permission_id: Uuid) -> Result<Permission> {
        let permission = sqlx::query_as::<_, Permission>(
            "SELECT * FROM permissions WHERE id = ?"
        )
        .bind(permission_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::Validation("Permission not found".to_string()))?;

        Ok(permission)
    }

    pub async fn get_permission_by_resource_action(&self, resource: &str, action: &str) -> Result<Permission> {
        let permission = sqlx::query_as::<_, Permission>(
            "SELECT * FROM permissions WHERE resource = ? AND action = ?"
        )
        .bind(resource)
        .bind(action)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::Validation("Permission not found".to_string()))?;

        Ok(permission)
    }

    pub async fn update_permission(&self, permission_id: Uuid, request: UpdatePermissionRequest, updated_by_id: Option<Uuid>) -> Result<Permission> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        let mut permission = self.get_permission_by_id(permission_id).await?;

        if let Some(name) = request.name {
            permission.name = name;
        }

        if let Some(description) = request.description {
            permission.description = Some(description);
        }

        if let Some(resource) = request.resource {
            permission.resource = resource;
        }

        if let Some(action) = request.action {
            permission.action = action;
        }

        if let Some(is_active) = request.is_active {
            permission.is_active = is_active;
        }

        permission.last_modified_date = Utc::now();
        permission.last_modified_by_id = updated_by_id;

        // Update in database
        sqlx::query(
            r#"
            UPDATE permissions SET
                name = ?, description = ?, resource = ?, action = ?, is_active = ?,
                last_modified_date = ?, last_modified_by_id = ?
            WHERE id = ?
            "#,
        )
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(&permission.resource)
        .bind(&permission.action)
        .bind(permission.is_active)
        .bind(permission.last_modified_date)
        .bind(&permission.last_modified_by_id)
        .bind(&permission.id)
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(permission)
    }

    pub async fn delete_permission(&self, permission_id: Uuid) -> Result<()> {
        // Check if permission is assigned to any roles
        let role_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM role_permissions WHERE permission_id = ?")
            .bind(permission_id)
            .fetch_one(&self.db)
            .await
            .map_err(AuthError::Database)?;

        if role_count.0 > 0 {
            return Err(AuthError::Validation("Cannot delete permission assigned to roles".to_string()));
        }

        // Delete permission
        sqlx::query("DELETE FROM permissions WHERE id = ?")
            .bind(permission_id)
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn list_permissions(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Permission>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT * FROM permissions ORDER BY resource, action LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(permissions)
    }

    pub async fn list_permissions_by_resource(&self, resource: &str) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT * FROM permissions WHERE resource = ? OR resource = '*' ORDER BY action"
        )
        .bind(resource)
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(permissions)
    }

    pub async fn check_permission(&self, user_permissions: &[String], resource: &str, action: &str) -> bool {
        // Check for admin permissions
        if user_permissions.contains(&"*".to_string()) {
            return true;
        }

        // Check for resource wildcard
        let resource_wildcard = format!("{}.*", resource);
        if user_permissions.contains(&resource_wildcard) {
            return true;
        }

        // Check for specific permission
        let specific_permission = format!("{}.{}", resource, action);
        user_permissions.contains(&specific_permission)
    }

    pub async fn get_roles_with_permission(&self, permission_id: Uuid) -> Result<Vec<crate::Role>> {
        let roles = sqlx::query_as::<_, crate::Role>(
            r#"
            SELECT r.*
            FROM roles r
            JOIN role_permissions rp ON r.id = rp.role_id
            WHERE rp.permission_id = ? AND r.is_active = TRUE
            ORDER BY r.name
            "#,
        )
        .bind(permission_id)
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(roles)
    }

    pub async fn initialize_default_permissions(&self) -> Result<()> {
        let default_permissions = vec![
            Permission::system_admin(),
            Permission::user_read(),
            Permission::user_write(),
            Permission::api_read(),
            Permission::api_write(),
        ];

        for permission in default_permissions {
            // Check if permission already exists
            if self.get_permission_by_resource_action(&permission.resource, &permission.action).await.is_err() {
                // Insert permission
                sqlx::query(
                    r#"
                    INSERT INTO permissions (
                        id, name, description, resource, action, is_active,
                        created_date, last_modified_date, created_by_id, last_modified_by_id
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&permission.id)
                .bind(&permission.name)
                .bind(&permission.description)
                .bind(&permission.resource)
                .bind(&permission.action)
                .bind(permission.is_active)
                .bind(permission.created_date)
                .bind(permission.last_modified_date)
                .bind(&permission.created_by_id)
                .bind(&permission.last_modified_by_id)
                .execute(&self.db)
                .await
                .map_err(AuthError::Database)?;
            }
        }

        Ok(())
    }
}