use crate::routing::{ApiRoute, ServiceHandler};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn, instrument};

/// Email service handler - implements /api/v2/email/* endpoints
pub struct EmailServiceHandler;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmailRequest {
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    #[serde(default)]
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub from_name: Option<String>,
    pub from_email: Option<String>,
    pub reply_to_name: Option<String>,
    pub reply_to_email: Option<String>,
    #[serde(default)]
    pub attachments: Vec<EmailAttachment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmailAttachment {
    pub name: String,
    pub content_type: String,
    pub content: String, // Base64 encoded
}

#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub success: bool,
    pub message: String,
    pub message_id: Option<String>,
    pub recipients: Vec<String>,
    pub errors: Vec<String>,
}

impl EmailServiceHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle POST /api/v2/email/_send
    #[instrument(skip(self, body))]
    async fn handle_send(&self, body: Option<Vec<u8>>) -> Result<Value> {
        let email_data = match body {
            Some(data) => {
                let body_str = String::from_utf8(data)?;
                serde_json::from_str::<EmailRequest>(&body_str)?
            }
            None => return Err(anyhow::anyhow!("Request body required for email send operation")),
        };

        // Validate email request
        self.validate_email_request(&email_data)?;

        // Generate a mock message ID
        let message_id = format!("df-email-{}", uuid::Uuid::new_v4());

        info!(
            "Email send request: to={:?}, subject={}, message_id={}",
            email_data.to, email_data.subject, message_id
        );

        // In a real implementation, this would integrate with an email service
        // For now, we'll just validate and return success
        let response = EmailResponse {
            success: true,
            message: "Email sent successfully".to_string(),
            message_id: Some(message_id),
            recipients: email_data.to.clone(),
            errors: vec![],
        };

        Ok(serde_json::to_value(response)?)
    }

    /// Handle GET /api/v2/email/template
    #[instrument(skip(self))]
    async fn handle_list_templates(&self) -> Result<Value> {
        info!("Retrieving email templates");
        
        // Return mock templates
        Ok(json!({
            "resource": [
                {
                    "id": 1,
                    "name": "welcome",
                    "description": "Welcome email template",
                    "subject": "Welcome to DreamFactory!",
                    "body_text": "Welcome to DreamFactory! We're glad you're here.",
                    "body_html": "<h1>Welcome to DreamFactory!</h1><p>We're glad you're here.</p>",
                    "from_name": "DreamFactory",
                    "from_email": "noreply@dreamfactory.com",
                    "is_active": true,
                    "created_date": "2024-01-01T00:00:00Z",
                    "last_modified_date": "2024-01-01T00:00:00Z"
                },
                {
                    "id": 2,
                    "name": "password_reset",
                    "description": "Password reset email template",
                    "subject": "Password Reset Request",
                    "body_text": "Click the link to reset your password: {reset_url}",
                    "body_html": "<p>Click <a href=\"{reset_url}\">here</a> to reset your password.</p>",
                    "from_name": "DreamFactory",
                    "from_email": "noreply@dreamfactory.com",
                    "is_active": true,
                    "created_date": "2024-01-01T00:00:00Z",
                    "last_modified_date": "2024-01-01T00:00:00Z"
                }
            ]
        }))
    }

    /// Handle GET /api/v2/email/template/{id}
    #[instrument(skip(self))]
    async fn handle_get_template(&self, template_id: &str) -> Result<Value> {
        info!("Retrieving email template: {}", template_id);
        
        // Return mock template based on ID
        match template_id {
            "1" | "welcome" => Ok(json!({
                "id": 1,
                "name": "welcome",
                "description": "Welcome email template",
                "subject": "Welcome to DreamFactory!",
                "body_text": "Welcome to DreamFactory! We're glad you're here.",
                "body_html": "<h1>Welcome to DreamFactory!</h1><p>We're glad you're here.</p>",
                "from_name": "DreamFactory",
                "from_email": "noreply@dreamfactory.com",
                "is_active": true,
                "created_date": "2024-01-01T00:00:00Z",
                "last_modified_date": "2024-01-01T00:00:00Z"
            })),
            "2" | "password_reset" => Ok(json!({
                "id": 2,
                "name": "password_reset",
                "description": "Password reset email template",
                "subject": "Password Reset Request",
                "body_text": "Click the link to reset your password: {reset_url}",
                "body_html": "<p>Click <a href=\"{reset_url}\">here</a> to reset your password.</p>",
                "from_name": "DreamFactory",
                "from_email": "noreply@dreamfactory.com",
                "is_active": true,
                "created_date": "2024-01-01T00:00:00Z",
                "last_modified_date": "2024-01-01T00:00:00Z"
            })),
            _ => Err(anyhow::anyhow!("Email template '{}' not found", template_id)),
        }
    }

    /// Validate email request parameters
    fn validate_email_request(&self, email: &EmailRequest) -> Result<()> {
        if email.to.is_empty() {
            return Err(anyhow::anyhow!("At least one recipient email address is required"));
        }

        if email.subject.is_empty() {
            return Err(anyhow::anyhow!("Subject is required"));
        }

        if email.body_text.is_none() && email.body_html.is_none() {
            return Err(anyhow::anyhow!("Either body_text or body_html is required"));
        }

        // Validate email addresses
        for to_email in &email.to {
            if !self.is_valid_email(to_email) {
                return Err(anyhow::anyhow!("Invalid email address: {}", to_email));
            }
        }

        for cc_email in &email.cc {
            if !self.is_valid_email(cc_email) {
                return Err(anyhow::anyhow!("Invalid CC email address: {}", cc_email));
            }
        }

        for bcc_email in &email.bcc {
            if !self.is_valid_email(bcc_email) {
                return Err(anyhow::anyhow!("Invalid BCC email address: {}", bcc_email));
            }
        }

        Ok(())
    }

    /// Basic email validation
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }
}

impl ServiceHandler for EmailServiceHandler {
    #[instrument(skip(self, query_params, body))]
    async fn handle_request(
        &self,
        route: &ApiRoute,
        method: &str,
        _query_params: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<Value> {
        match (&route.resource, &route.id, method) {
            // Handle email send
            (Some(resource), None, "POST") if resource == "_send" => {
                self.handle_send(body).await
            }
            // Handle template operations
            (Some(resource), None, "GET") if resource == "template" => {
                self.handle_list_templates().await
            }
            (Some(resource), Some(template_id), "GET") if resource == "template" => {
                self.handle_get_template(template_id).await
            }
            // Handle email service info
            (None, None, "GET") => {
                Ok(json!({
                    "name": "email",
                    "label": "Email Service",
                    "description": "Email sending and template management service",
                    "version": env!("CARGO_PKG_VERSION"),
                    "endpoints": [
                        {
                            "path": "/api/v2/email/_send",
                            "method": "POST",
                            "description": "Send email"
                        },
                        {
                            "path": "/api/v2/email/template",
                            "method": "GET",
                            "description": "List email templates"
                        },
                        {
                            "path": "/api/v2/email/template/{id}",
                            "method": "GET",
                            "description": "Get email template by ID"
                        }
                    ]
                }))
            }
            // Unknown operations
            (resource, id, method) => {
                Err(anyhow::anyhow!(
                    "Unsupported email operation: {} {} {}",
                    method,
                    resource.as_deref().unwrap_or("(root)"),
                    id.as_deref().unwrap_or("")
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::ApiRoute;

    #[tokio::test]
    async fn test_email_send_success() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("_send".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let email_json = json!({
            "to": ["test@example.com"],
            "subject": "Test Email",
            "body_text": "This is a test email"
        });

        let result = handler
            .handle_request(&route, "POST", HashMap::new(), Some(email_json.to_string().as_bytes().to_vec()))
            .await
            .unwrap();

        assert_eq!(result.get("success"), Some(&json!(true)));
        assert!(result.get("message_id").is_some());
        assert_eq!(result.get("recipients"), Some(&json!(["test@example.com"])));
    }

    #[tokio::test]
    async fn test_email_send_with_html() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("_send".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let email_json = json!({
            "to": ["test@example.com"],
            "cc": ["cc@example.com"],
            "subject": "Test HTML Email",
            "body_html": "<h1>Test</h1><p>This is a test email</p>",
            "from_name": "Test Sender",
            "from_email": "sender@example.com"
        });

        let result = handler
            .handle_request(&route, "POST", HashMap::new(), Some(email_json.to_string().as_bytes().to_vec()))
            .await
            .unwrap();

        assert_eq!(result.get("success"), Some(&json!(true)));
        assert!(result.get("message_id").is_some());
    }

    #[tokio::test]
    async fn test_email_send_validation_error() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("_send".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let email_json = json!({
            "to": [],
            "subject": "Test Email",
            "body_text": "This is a test email"
        });

        let result = handler
            .handle_request(&route, "POST", HashMap::new(), Some(email_json.to_string().as_bytes().to_vec()))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_send_missing_body() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("_send".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "POST", HashMap::new(), None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_list_templates() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("template".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        let templates = result.get("resource").unwrap().as_array().unwrap();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].get("name"), Some(&json!("welcome")));
        assert_eq!(templates[1].get("name"), Some(&json!("password_reset")));
    }

    #[tokio::test]
    async fn test_email_get_template() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("template".to_string()),
            id: Some("1".to_string()),
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(result.get("name"), Some(&json!("welcome")));
        assert_eq!(result.get("id"), Some(&json!(1)));
    }

    #[tokio::test]
    async fn test_email_get_template_by_name() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("template".to_string()),
            id: Some("password_reset".to_string()),
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(result.get("name"), Some(&json!("password_reset")));
        assert_eq!(result.get("id"), Some(&json!(2)));
    }

    #[tokio::test]
    async fn test_email_get_nonexistent_template() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: Some("template".to_string()),
            id: Some("nonexistent".to_string()),
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_service_info() {
        let handler = EmailServiceHandler::new();
        
        let route = ApiRoute {
            version: "2".to_string(),
            service: "email".to_string(),
            resource: None,
            id: None,
            sub_resource: None,
            sub_id: None,
        };

        let result = handler
            .handle_request(&route, "GET", HashMap::new(), None)
            .await
            .unwrap();

        assert_eq!(result.get("name"), Some(&json!("email")));
        assert!(result.get("endpoints").is_some());
    }

    #[tokio::test]
    async fn test_email_validation() {
        let handler = EmailServiceHandler::new();
        
        // Valid emails
        assert!(handler.is_valid_email("test@example.com"));
        assert!(handler.is_valid_email("user.name@domain.co.uk"));
        
        // Invalid emails
        assert!(!handler.is_valid_email("invalid"));
        assert!(!handler.is_valid_email("@domain.com"));
        assert!(!handler.is_valid_email("user@"));
        assert!(!handler.is_valid_email(""));
    }
}