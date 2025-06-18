use crate::{
    AuthError, Result, ApiKey, ApiKeyResponse, CreateApiKeyRequest, UpdateApiKeyRequest,
    PasswordService, AuthConfig
};
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct ApiKeyService {
    db: Pool<Sqlite>,
    password_service: PasswordService,
    config: AuthConfig,
}

impl ApiKeyService {
    pub fn new(db: Pool<Sqlite>, config: AuthConfig) -> Self {
        let password_service = PasswordService::new(&config);
        Self {
            db,
            password_service,
            config,
        }
    }

    pub async fn create_api_key(&self, request: CreateApiKeyRequest, created_by_id: Option<Uuid>) -> Result<ApiKeyResponse> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        // Generate API key
        let api_key = self.password_service.generate_api_key(self.config.api_key_length);
        let key_hash = self.password_service.hash_api_key(&api_key)?;

        let api_key_record = ApiKey::new(
            request.name,
            key_hash,
            request.user_id,
            request.role_id,
            request.expires_at,
            request.rate_limit_per_minute,
            request.allowed_ips,
            created_by_id,
        );

        // Insert into database
        sqlx::query!(
            r#"
            INSERT INTO api_keys (
                id, name, key_hash, user_id, role_id, is_active, expires_at,
                last_used_at, usage_count, rate_limit_per_minute, allowed_ips,
                created_date, last_modified_date, created_by_id, last_modified_by_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            api_key_record.id,
            api_key_record.name,
            api_key_record.key_hash,
            api_key_record.user_id,
            api_key_record.role_id,
            api_key_record.is_active,
            api_key_record.expires_at,
            api_key_record.last_used_at,
            api_key_record.usage_count,
            api_key_record.rate_limit_per_minute,
            api_key_record.allowed_ips,
            api_key_record.created_date,
            api_key_record.last_modified_date,
            api_key_record.created_by_id,
            api_key_record.last_modified_by_id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(ApiKeyResponse {
            id: api_key_record.id,
            name: api_key_record.name,
            key: api_key, // Only returned on creation
            is_active: api_key_record.is_active,
            expires_at: api_key_record.expires_at,
            created_date: api_key_record.created_date,
        })
    }

    pub async fn get_api_key_by_id(&self, api_key_id: Uuid) -> Result<ApiKey> {
        let api_key = sqlx::query_as!(
            ApiKey,
            "SELECT * FROM api_keys WHERE id = ?",
            api_key_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|_| AuthError::ApiKeyNotFound)?;

        Ok(api_key)
    }

    pub async fn authenticate_api_key(&self, key: &str, ip_address: Option<&str>) -> Result<ApiKey> {
        // Get all active API keys to check against
        let api_keys = sqlx::query_as!(
            ApiKey,
            "SELECT * FROM api_keys WHERE is_active = TRUE"
        )
        .fetch_all(&self.db)
        .await
        .map_err(AuthError::Database)?;

        for mut api_key in api_keys {
            // Verify key hash
            if self.password_service.verify_api_key(key, &api_key.key_hash)? {
                // Check if expired
                if api_key.is_expired() {
                    return Err(AuthError::ApiKeyExpired);
                }

                // Check IP restrictions
                if let Some(ip) = ip_address {
                    if !api_key.is_ip_allowed(ip) {
                        return Err(AuthError::Forbidden);
                    }
                }

                // Update usage
                api_key.update_usage();
                self.update_api_key_usage(api_key.id, api_key.usage_count, api_key.last_used_at).await?;

                return Ok(api_key);
            }
        }

        Err(AuthError::ApiKeyNotFound)
    }

    pub async fn update_api_key(&self, api_key_id: Uuid, request: UpdateApiKeyRequest, updated_by_id: Option<Uuid>) -> Result<ApiKey> {
        request.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

        let mut api_key = self.get_api_key_by_id(api_key_id).await?;

        if let Some(name) = request.name {
            api_key.name = name;
        }

        if let Some(is_active) = request.is_active {
            api_key.is_active = is_active;
        }

        if let Some(expires_at) = request.expires_at {
            api_key.expires_at = Some(expires_at);
        }

        if let Some(rate_limit) = request.rate_limit_per_minute {
            api_key.rate_limit_per_minute = Some(rate_limit);
        }

        if let Some(allowed_ips) = request.allowed_ips {
            api_key.allowed_ips = Some(serde_json::to_string(&allowed_ips).unwrap_or_default());
        }

        api_key.last_modified_date = Utc::now();
        api_key.last_modified_by_id = updated_by_id;

        // Update in database
        sqlx::query!(
            r#"
            UPDATE api_keys SET
                name = ?, is_active = ?, expires_at = ?, rate_limit_per_minute = ?,
                allowed_ips = ?, last_modified_date = ?, last_modified_by_id = ?
            WHERE id = ?
            "#,
            api_key.name,
            api_key.is_active,
            api_key.expires_at,
            api_key.rate_limit_per_minute,
            api_key.allowed_ips,
            api_key.last_modified_date,
            api_key.last_modified_by_id,
            api_key.id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(api_key)
    }

    pub async fn delete_api_key(&self, api_key_id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM api_keys WHERE id = ?", api_key_id)
            .execute(&self.db)
            .await
            .map_err(AuthError::Database)?;

        Ok(())
    }

    pub async fn list_api_keys(&self, user_id: Option<Uuid>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ApiKey>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let api_keys = if let Some(user_id) = user_id {
            sqlx::query_as!(
                ApiKey,
                "SELECT * FROM api_keys WHERE user_id = ? ORDER BY created_date DESC LIMIT ? OFFSET ?",
                user_id,
                limit,
                offset
            )
            .fetch_all(&self.db)
            .await
        } else {
            sqlx::query_as!(
                ApiKey,
                "SELECT * FROM api_keys ORDER BY created_date DESC LIMIT ? OFFSET ?",
                limit,
                offset
            )
            .fetch_all(&self.db)
            .await
        };

        api_keys.map_err(AuthError::Database)
    }

    pub async fn get_user_permissions_for_api_key(&self, api_key: &ApiKey) -> Result<Vec<String>> {
        if let Some(user_id) = api_key.user_id {
            // Get permissions from user's roles
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
        } else if let Some(role_id) = api_key.role_id {
            // Get permissions from specific role
            let rows = sqlx::query!(
                r#"
                SELECT p.resource || '.' || p.action as permission
                FROM permissions p
                JOIN role_permissions rp ON p.id = rp.permission_id
                WHERE rp.role_id = ? AND p.is_active = TRUE
                "#,
                role_id
            )
            .fetch_all(&self.db)
            .await
            .map_err(AuthError::Database)?;

            Ok(rows.into_iter().map(|row| row.permission).collect())
        } else {
            // No permissions if no user or role associated
            Ok(vec![])
        }
    }

    pub async fn cleanup_expired_api_keys(&self) -> Result<u64> {
        let now = Utc::now();
        let result = sqlx::query!(
            "DELETE FROM api_keys WHERE expires_at IS NOT NULL AND expires_at < ?",
            now
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(result.rows_affected())
    }

    async fn update_api_key_usage(&self, api_key_id: Uuid, usage_count: i64, last_used_at: Option<chrono::DateTime<Utc>>) -> Result<()> {
        sqlx::query!(
            "UPDATE api_keys SET usage_count = ?, last_used_at = ?, last_modified_date = ? WHERE id = ?",
            usage_count,
            last_used_at,
            Utc::now(),
            api_key_id
        )
        .execute(&self.db)
        .await
        .map_err(AuthError::Database)?;

        Ok(())
    }
}