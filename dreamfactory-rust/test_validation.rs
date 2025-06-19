// Test file to verify RegisterRequest validation works
use validator::Validate;

#[derive(Debug, serde::Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    pub username: Option<String>,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

fn main() {
    // Test valid request
    let valid_request = RegisterRequest {
        email: "test@example.com".to_string(),
        username: Some("testuser".to_string()),
        password: "ValidPassword123".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
    };
    
    match valid_request.validate() {
        Ok(()) => println!("✅ Valid request passes validation"),
        Err(e) => println!("❌ Valid request failed: {:?}", e),
    }
    
    // Test invalid email
    let invalid_email_request = RegisterRequest {
        email: "invalid-email".to_string(),
        username: Some("testuser".to_string()),
        password: "ValidPassword123".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
    };
    
    match invalid_email_request.validate() {
        Ok(()) => println!("❌ Invalid email request should have failed"),
        Err(e) => println!("✅ Invalid email properly rejected: {:?}", e),
    }
    
    // Test short password
    let short_password_request = RegisterRequest {
        email: "test@example.com".to_string(),
        username: Some("testuser".to_string()),
        password: "short".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
    };
    
    match short_password_request.validate() {
        Ok(()) => println!("❌ Short password request should have failed"),
        Err(e) => println!("✅ Short password properly rejected: {:?}", e),
    }
}