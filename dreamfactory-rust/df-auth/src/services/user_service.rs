use crate::{
    AuthError, Result, User, UserProfile, CreateUserRequest, UpdateUserRequest, 
    PasswordService, AuthConfig
};
use chrono::Utc;
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct UserService {
    db: Pool<Sqlite>,
    password_service: PasswordService,
    config: AuthConfig,
}

impl UserService {
    pub fn new(db: Pool<Sqlite>, config: AuthConfig) -> Self {
        let password_service = PasswordService::new(&config);
        Self {
            db,
            password_service,
            config,
        }
    }

    pub async fn create_user(&self, request: CreateUserRequest, created_by_id: Option<Uuid>) -> Result<User> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        // Validate password strength
        self.password_service.validate_password_strength(&request.password)?;

        // Check if user already exists
        if self.get_user_by_email(&request.email).await.is_ok() {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash password
        let password_hash = self.password_service.hash_password(&request.password)?;

        // Create user
        let user = User::new(
            request.email,
            password_hash,
            request.username,
            request.first_name,
            request.last_name,
            created_by_id,
        );

        // Insert into database
        sqlx::query!(
            r#"
            INSERT INTO users (
                id, email, username, password_hash, first_name, last_name,
                is_active, is_verified, created_date, last_modified_date,
                created_by_id, last_modified_by_id, login_attempts
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            user.id,
            user.email,
            user.username,
            user.password_hash,
            user.first_name,
            user.last_name,
            user.is_active,
            user.is_verified,
            user.created_date,
            user.last_modified_date,
            user.created_by_id,
            user.last_modified_by_id,
            user.login_attempts
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        // Assign default role if specified
        if let Some(role_id) = request.role_id {
            self.assign_role_to_user(user.id, role_id, created_by_id).await?;
        }

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = ?",
            user_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::UserNotFound)?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = ?",
            email
        )
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::UserNotFound)?;

        Ok(user)
    }

    pub async fn update_user(&self, user_id: Uuid, request: UpdateUserRequest, updated_by_id: Option<Uuid>) -> Result<User> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        let mut user = self.get_user_by_id(user_id).await?;

        // Update fields
        if let Some(email) = request.email {
            // Check if new email is already taken by another user
            if let Ok(existing_user) = self.get_user_by_email(&email).await {
                if existing_user.id != user_id {
                    return Err(AuthError::UserAlreadyExists);
                }
            }
            user.email = email;
        }

        if let Some(username) = request.username {
            user.username = Some(username);
        }

        if let Some(password) = request.password {
            self.password_service.validate_password_strength(&password)?;
            user.password_hash = self.password_service.hash_password(&password)?;
        }

        if let Some(first_name) = request.first_name {
            user.first_name = Some(first_name);
        }

        if let Some(last_name) = request.last_name {
            user.last_name = Some(last_name);
        }

        if let Some(is_active) = request.is_active {
            user.is_active = is_active;
        }

        if let Some(is_verified) = request.is_verified {
            user.is_verified = is_verified;
        }

        user.last_modified_date = Utc::now();
        user.last_modified_by_id = updated_by_id;

        // Update in database
        sqlx::query!(
            r#"
            UPDATE users SET
                email = ?, username = ?, password_hash = ?, first_name = ?, last_name = ?,
                is_active = ?, is_verified = ?, last_modified_date = ?, last_modified_by_id = ?
            WHERE id = ?
            "#,
            user.email,
            user.username,
            user.password_hash,
            user.first_name,
            user.last_name,
            user.is_active,
            user.is_verified,
            user.last_modified_date,
            user.last_modified_by_id,
            user.id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn get_user_profile(&self, user_id: Uuid) -> Result<UserProfile> {
        let user = self.get_user_by_id(user_id).await?;
        let roles = self.get_user_roles(user_id).await?;
        
        Ok(user.to_profile(roles))
    }

    pub async fn verify_password(&self, user_id: Uuid, password: &str) -> Result<bool> {
        let user = self.get_user_by_id(user_id).await?;
        self.password_service.verify_password(password, &user.password_hash)
    }

    pub async fn authenticate_user(&self, email: &str, password: &str) -> Result<User> {
        let mut user = self.get_user_by_email(email).await?;

        // Check if user is locked
        if user.is_locked() {
            return Err(AuthError::Unauthorized);
        }

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::Unauthorized);
        }

        // Verify password
        let is_valid = self.password_service.verify_password(password, &user.password_hash)?;

        if is_valid {
            // Reset login attempts and update last login
            user.login_attempts = 0;
            user.locked_until = None;
            user.last_login_date = Some(Utc::now());

            sqlx::query!(
                "UPDATE users SET login_attempts = 0, locked_until = NULL, last_login_date = ? WHERE id = ?",
                user.last_login_date,
                user.id
            )
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

            Ok(user)
        } else {
            // Increment login attempts
            user.login_attempts += 1;

            // Lock user if max attempts reached
            if user.login_attempts >= self.config.max_login_attempts as i32 {
                user.locked_until = Some(Utc::now() + chrono::Duration::seconds(self.config.lockout_duration));
            }

            sqlx::query!(
                "UPDATE users SET login_attempts = ?, locked_until = ? WHERE id = ?",
                user.login_attempts,
                user.locked_until,
                user.id
            )
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

            Err(AuthError::InvalidCredentials)
        }
    }

    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows = sqlx::query!(
            r#"
            SELECT r.name
            FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = ? AND r.is_active = TRUE
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(rows.into_iter().map(|row| row.name).collect())
    }

    pub async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT p.resource || '.' || p.action as permission
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = ? AND p.is_active = TRUE
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(rows.into_iter().map(|row| row.permission).collect())
    }

    pub async fn assign_role_to_user(&self, user_id: Uuid, role_id: Uuid, assigned_by_id: Option<Uuid>) -> Result<()> {
        // Check if role assignment already exists
        let existing = sqlx::query!(
            "SELECT id FROM user_roles WHERE user_id = ? AND role_id = ?",
            user_id,
            role_id
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
            "INSERT INTO user_roles (id, user_id, role_id, created_date, created_by_id) VALUES (?, ?, ?, ?, ?)",
            assignment_id,
            user_id,
            role_id,
            created_date,
            assigned_by_id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn remove_role_from_user(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM user_roles WHERE user_id = ? AND role_id = ?",
            user_id,
            role_id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn list_users(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let users = sqlx::query_as!(
            User,
            "SELECT * FROM users ORDER BY created_date DESC LIMIT ? OFFSET ?",
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(users)
    }
}