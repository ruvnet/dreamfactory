use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Email message structure
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EmailMessage {
    #[validate(email)]
    pub from: String,
    #[validate(length(min = 1))]
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    #[validate(length(min = 1))]
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Option<Vec<EmailAttachment>>,
    pub headers: Option<HashMap<String, String>>,
}

/// Email attachment structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

/// Email send result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

/// Email template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: String,
    pub name: String,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub variables: Vec<String>,
}

/// Email service trait
#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(&self, message: EmailMessage) -> Result<EmailSendResult>;
    async fn send_templated_email(&self, template_id: &str, variables: HashMap<String, String>, to: Vec<String>) -> Result<EmailSendResult>;
    async fn validate_email(&self, email: &str) -> bool;
    async fn get_templates(&self) -> Result<Vec<EmailTemplate>>;
}

/// Mock email service for testing/development
#[derive(Debug)]
pub struct MockEmailService {
    templates: Vec<EmailTemplate>,
}

impl MockEmailService {
    pub fn new() -> Self {
        let templates = vec![
            EmailTemplate {
                id: "welcome".to_string(),
                name: "Welcome Email".to_string(),
                subject: "Welcome to {{app_name}}!".to_string(),
                body_text: Some("Welcome {{user_name}} to {{app_name}}!".to_string()),
                body_html: Some("<h1>Welcome {{user_name}} to {{app_name}}!</h1>".to_string()),
                variables: vec!["user_name".to_string(), "app_name".to_string()],
            },
            EmailTemplate {
                id: "password_reset".to_string(),
                name: "Password Reset".to_string(),
                subject: "Reset your password".to_string(),
                body_text: Some("Click here to reset your password: {{reset_link}}".to_string()),
                body_html: Some("<p>Click <a href=\"{{reset_link}}\">here</a> to reset your password.</p>".to_string()),
                variables: vec!["reset_link".to_string()],
            },
        ];

        Self { templates }
    }

    fn render_template(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

impl Default for MockEmailService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmailService for MockEmailService {
    async fn send_email(&self, message: EmailMessage) -> Result<EmailSendResult> {
        // Validate the message
        message.validate().map_err(|e| anyhow::anyhow!("Invalid email message: {}", e))?;

        // In a real implementation, this would send the email via SMTP or an email service
        // For now, we just return a mock success result
        Ok(EmailSendResult {
            success: true,
            message_id: Some(format!("mock-{}", uuid::Uuid::new_v4())),
            error: None,
        })
    }

    async fn send_templated_email(&self, template_id: &str, variables: HashMap<String, String>, to: Vec<String>) -> Result<EmailSendResult> {
        let template = self.templates.iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_id))?;

        let subject = self.render_template(&template.subject, &variables);
        let body_text = template.body_text.as_ref().map(|t| self.render_template(t, &variables));
        let body_html = template.body_html.as_ref().map(|t| self.render_template(t, &variables));

        let message = EmailMessage {
            from: "noreply@dreamfactory.local".to_string(),
            to,
            cc: None,
            bcc: None,
            subject,
            body_text,
            body_html,
            attachments: None,
            headers: None,
        };

        self.send_email(message).await
    }

    async fn validate_email(&self, email: &str) -> bool {
        validator::validate_email(email)
    }

    async fn get_templates(&self) -> Result<Vec<EmailTemplate>> {
        Ok(self.templates.clone())
    }
}

pub fn hello() -> &'static str {
    "df-email"
}