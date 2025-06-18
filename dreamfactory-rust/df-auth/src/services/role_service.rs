use crate::{
    AuthError, Result, Role, CreateRoleRequest, UpdateRoleRequest, AssignRoleRequest
};
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct RoleService {
    db: Pool<Sqlite>,
}

impl RoleService {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    pub async fn create_role(&self, request: CreateRoleRequest, created_by_id: Option<Uuid>) -> Result<Role> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        // Check if role name already exists
        if self.get_role_by_name(&request.name).await.is_ok() {
            return Err(AuthError::Validation("Role name already exists".to_string()));
        }

        let role = Role::new(request.name, request.description, created_by_id);

        // Insert role
        sqlx::query!(
            r#"
            INSERT INTO roles (
                id, name, description, is_active, created_date, last_modified_date,
                created_by_id, last_modified_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            role.id,
            role.name,
            role.description,
            role.is_active,
            role.created_date,
            role.last_modified_date,
            role.created_by_id,
            role.last_modified_by_id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        // Assign permissions if provided
        if let Some(permission_ids) = request.permission_ids {
            for permission_id in permission_ids {
                self.assign_permission_to_role(role.id, permission_id, created_by_id).await?;
            }
        }

        Ok(role)
    }

    pub async fn get_role_by_id(&self, role_id: Uuid) -> Result<Role> {
        let role = sqlx::query_as!(
            Role,
            "SELECT * FROM roles WHERE id = ?",
            role_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::RoleNotFound)?;

        Ok(role)
    }

    pub async fn get_role_by_name(&self, name: &str) -> Result<Role> {
        let role = sqlx::query_as!(
            Role,
            "SELECT * FROM roles WHERE name = ?",
            name
        )
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::RoleNotFound)?;

        Ok(role)
    }

    pub async fn update_role(&self, role_id: Uuid, request: UpdateRoleRequest, updated_by_id: Option<Uuid>) -> Result<Role> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        let mut role = self.get_role_by_id(role_id).await?;

        if let Some(name) = request.name {
            // Check if new name is already taken by another role
            if let Ok(existing_role) = self.get_role_by_name(&name).await {
                if existing_role.id != role_id {
                    return Err(AuthError::Validation("Role name already exists".to_string()));
                }
            }
            role.name = name;
        }

        if let Some(description) = request.description {
            role.description = Some(description);
        }

        if let Some(is_active) = request.is_active {
            role.is_active = is_active;
        }

        role.last_modified_date = Utc::now();
        role.last_modified_by_id = updated_by_id;

        // Update in database
        sqlx::query!(
            r#"
            UPDATE roles SET
                name = ?, description = ?, is_active = ?, last_modified_date = ?, last_modified_by_id = ?
            WHERE id = ?
            "#,
            role.name,
            role.description,
            role.is_active,
            role.last_modified_date,
            role.last_modified_by_id,
            role.id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        // Update permissions if provided
        if let Some(permission_ids) = request.permission_ids {
            // Remove existing permissions
            sqlx::query!("DELETE FROM role_permissions WHERE role_id = ?", role_id)
                .execute(&self.db)
                .await
                .map_err(AuthError::Database)?;

            // Add new permissions
            for permission_id in permission_ids {
                self.assign_permission_to_role(role_id, permission_id, updated_by_id).await?;
            }
        }

        Ok(role)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        // Check if role is assigned to any users
        let user_count = sqlx::query!("SELECT COUNT(*) as count FROM user_roles WHERE role_id = ?", role_id)
            .fetch_one(&self.db)
            .await
            .map_err(AuthError::Database)?;

        if user_count.count > 0 {
            return Err(AuthError::Validation("Cannot delete role assigned to users".to_string()));
        }

        // Delete role permissions first
        sqlx::query!("DELETE FROM role_permissions WHERE role_id = ?", role_id)
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

        // Delete role
        sqlx::query!("DELETE FROM roles WHERE id = ?", role_id)
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn list_roles(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Role>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let roles = sqlx::query_as!(
            Role,
            "SELECT * FROM roles ORDER BY name LIMIT ? OFFSET ?",
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(roles)
    }

    pub async fn get_role_permissions(&self, role_id: Uuid) -> Result<Vec<crate::Permission>> {
        let permissions = sqlx::query_as!(
            crate::Permission,
            r#"
            SELECT p.*
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = ? AND p.is_active = TRUE
            ORDER BY p.name
            "#,
            role_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(permissions)
    }

    pub async fn assign_permission_to_role(&self, role_id: Uuid, permission_id: Uuid, assigned_by_id: Option<Uuid>) -> Result<()> {
        // Check if assignment already exists
        let existing = sqlx::query!(
            "SELECT id FROM role_permissions WHERE role_id = ? AND permission_id = ?",
            role_id,
            permission_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(AuthError::Database)?;

        if existing.is_some() {
            return Ok(()); // Already assigned
        }

        let assignment_id = Uuid::new_v4();
        let created_date = Utc::now();

        sqlx::query!(
            "INSERT INTO role_permissions (id, role_id, permission_id, created_date, created_by_id) VALUES (?, ?, ?, ?, ?)",
            assignment_id,
            role_id,
            permission_id,
            created_date,
            assigned_by_id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn remove_permission_from_role(&self, role_id: Uuid, permission_id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM role_permissions WHERE role_id = ? AND permission_id = ?",
            role_id,
            permission_id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn get_users_with_role(&self, role_id: Uuid) -> Result<Vec<crate::User>> {
        let users = sqlx::query_as!(
            crate::User,
            r#"
            SELECT u.*
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            WHERE ur.role_id = ? AND u.is_active = TRUE
            ORDER BY u.email
            "#,
            role_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(users)
    }
}