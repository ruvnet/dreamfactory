use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub user_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub rate_limit_per_minute: Option<i32>,
    pub allowed_ips: Option<String>, // JSON array of IPs
    pub created_date: DateTime<Utc>,
    pub last_modified_date: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
    pub last_modified_by_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key: String, // Only returned on creation
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, max = 255, message = "API key name must be between 1 and 255 characters"))]
    pub name: String,
    
    pub user_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit_per_minute: Option<i32>,
    pub allowed_ips: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateApiKeyRequest {
    #[validate(length(min = 1, max = 255, message = "API key name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit_per_minute: Option<i32>,
    pub allowed_ips: Option<Vec<String>>,
}

impl ApiKey {
    pub fn new(
        name: String,
        key_hash: String,
        user_id: Option<Uuid>,
        role_id: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
        rate_limit_per_minute: Option<i32>,
        allowed_ips: Option<Vec<String>>,
        created_by_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            key_hash,
            user_id,
            role_id,
            is_active: true,
            expires_at,
            last_used_at: None,
            usage_count: 0,
            rate_limit_per_minute,
            allowed_ips: allowed_ips.map(|ips| serde_json::to_string(&ips).unwrap_or_default()),
            created_date: now,
            last_modified_date: now,
            created_by_id,
            last_modified_by_id: created_by_id,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        if let Some(allowed_ips_json) = &self.allowed_ips {
            if let Ok(allowed_ips) = serde_json::from_str::<Vec<String>>(allowed_ips_json) {
                return allowed_ips.contains(&ip.to_string());
            }
        }
        // If no IP restrictions, allow all
        true
    }

    pub fn update_usage(&mut self) {
        self.usage_count += 1;
        self.last_used_at = Some(Utc::now());
        self.last_modified_date = Utc::now();
    }

    pub fn get_allowed_ips(&self) -> Vec<String> {
        if let Some(allowed_ips_json) = &self.allowed_ips {
            serde_json::from_str(allowed_ips_json).unwrap_or_default()
        } else {
            vec![]
        }
    }
}