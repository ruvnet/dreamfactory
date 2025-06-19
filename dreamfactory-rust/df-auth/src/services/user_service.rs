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

    pub fn db(&self) -> &Pool<Sqlite> {
        &self.db
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
        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, username, password_hash, first_name, last_name,
                is_active, is_verified, created_date, last_modified_date,
                created_by_id, last_modified_by_id, login_attempts
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(user.is_active)
        .bind(user.is_verified)
        .bind(user.created_date)
        .bind(user.last_modified_date)
        .bind(user.created_by_id.map(|id| id.to_string()))
        .bind(user.last_modified_by_id.map(|id| id.to_string()))
        .bind(user.login_attempts)
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
        let row = sqlx::query(
            "SELECT id, email, username, password_hash, first_name, last_name, is_active, is_verified, last_login_date, created_date, last_modified_date, created_by_id, last_modified_by_id, login_attempts, locked_until FROM users WHERE id = ?"
        )
        .bind(user_id.to_string())
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::UserNotFound)?;

        let user = User {
            id: Uuid::parse_str(&row.get::<String, _>("id")).map_err(|_| AuthError::UserNotFound)?,
            email: row.get("email"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            last_login_date: row.get("last_login_date"),
            created_date: row.get("created_date"),
            last_modified_date: row.get("last_modified_date"),
            created_by_id: row.get::<Option<String>, _>("created_by_id").and_then(|uuid_str| Uuid::parse_str(&uuid_str).ok()),
            last_modified_by_id: row.get::<Option<String>, _>("last_modified_by_id").and_then(|uuid_str| Uuid::parse_str(&uuid_str).ok()),
            login_attempts: row.get("login_attempts"),
            locked_until: row.get("locked_until"),
        };

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let row = sqlx::query(
            "SELECT id, email, username, password_hash, first_name, last_name, is_active, is_verified, last_login_date, created_date, last_modified_date, created_by_id, last_modified_by_id, login_attempts, locked_until FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::UserNotFound)?;

        let user = User {
            id: Uuid::parse_str(&row.get::<String, _>("id")).map_err(|_| AuthError::UserNotFound)?,
            email: row.get("email"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            last_login_date: row.get("last_login_date"),
            created_date: row.get("created_date"),
            last_modified_date: row.get("last_modified_date"),
            created_by_id: row.get::<Option<String>, _>("created_by_id").and_then(|uuid_str| Uuid::parse_str(&uuid_str).ok()),
            last_modified_by_id: row.get::<Option<String>, _>("last_modified_by_id").and_then(|uuid_str| Uuid::parse_str(&uuid_str).ok()),
            login_attempts: row.get("login_attempts"),
            locked_until: row.get("locked_until"),
        };

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
        sqlx::query(
            r#"
            UPDATE users SET
                email = ?, username = ?, password_hash = ?, first_name = ?, last_name = ?,
                is_active = ?, is_verified = ?, last_modified_date = ?, last_modified_by_id = ?
            WHERE id = ?
            "#
        )
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(user.is_active)
        .bind(user.is_verified)
        .bind(user.last_modified_date)
        .bind(user.last_modified_by_id.map(|id| id.to_string()))
        .bind(user.id.to_string())
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id.to_string())
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

            sqlx::query(
                "UPDATE users SET login_attempts = 0, locked_until = NULL, last_login_date = ? WHERE id = ?"
            )
            .bind(user.last_login_date)
            .bind(user.id.to_string())
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

            sqlx::query(
                "UPDATE users SET login_attempts = ?, locked_until = ? WHERE id = ?"
            )
            .bind(user.login_attempts)
            .bind(user.locked_until)
            .bind(user.id.to_string())
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

            Err(AuthError::InvalidCredentials)
        }
    }

    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT r.name
            FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = ? AND r.is_active = TRUE
            "#
        )
        .bind(user_id.to_string())
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(rows.into_iter().map(|row| row.get::<String, _>("name")).collect())
    }

    pub async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT p.resource || '.' || p.action as permission
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = ? AND p.is_active = TRUE
            "#
        )
        .bind(user_id.to_string())
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(rows.into_iter().map(|row| row.get::<String, _>("permission")).collect())
    }

    pub async fn assign_role_to_user(&self, user_id: Uuid, role_id: Uuid, assigned_by_id: Option<Uuid>) -> Result<()> {
        // Check if role assignment already exists
        let existing = sqlx::query(
            "SELECT id FROM user_roles WHERE user_id = ? AND role_id = ?"
        )
        .bind(user_id.to_string())
        .bind(role_id.to_string())
        .fetch_optional(&self.db)
        .await
        .map_err(AuthError::Database)?;

        if existing.is_some() {
            return Ok(()); // Already assigned
        }

        let assignment_id = Uuid::new_v4();
        let created_date = Utc::now();

        sqlx::query(
            "INSERT INTO user_roles (id, user_id, role_id, created_date, created_by_id) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(assignment_id.to_string())
        .bind(user_id.to_string())
        .bind(role_id.to_string())
        .bind(created_date)
        .bind(assigned_by_id.map(|id| id.to_string()))
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn remove_role_from_user(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        sqlx::query(
            "DELETE FROM user_roles WHERE user_id = ? AND role_id = ?"
        )
        .bind(user_id.to_string())
        .bind(role_id.to_string())
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn list_users(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, email, username, password_hash, first_name, last_name, is_active, is_verified, last_login_date, created_date, last_modified_date, created_by_id, last_modified_by_id, login_attempts, locked_until FROM users ORDER BY created_date DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        let users = rows.into_iter().map(|row| {
            User {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                email: row.get("email"),
                username: row.get("username"),
                password_hash: row.get("password_hash"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                is_active: row.get("is_active"),
                is_verified: row.get("is_verified"),
                last_login_date: row.get("last_login_date"),
                created_date: row.get("created_date"),
                last_modified_date: row.get("last_modified_date"),
                created_by_id: row.get::<Option<String>, _>("created_by_id").and_then(|uuid_str| Uuid::parse_str(&uuid_str).ok()),
                last_modified_by_id: row.get::<Option<String>, _>("last_modified_by_id").and_then(|uuid_str| Uuid::parse_str(&uuid_str).ok()),
                login_attempts: row.get("login_attempts"),
                locked_until: row.get("locked_until"),
            }
        }).collect();

        Ok(users)
    }
}