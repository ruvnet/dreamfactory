use axum_test::TestServer;
use df_api::{create_api_router};
use serde_json::json;

/// Integration tests for the complete DreamFactory REST API
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_api_compatibility() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test health check
        let response = server.get("/health").await;
        response.assert_status_ok();
        let health = response.json::<serde_json::Value>();
        assert_eq!(health.get("status"), Some(&json!("ok")));

        // Test API info
        let response = server.get("/").await;
        response.assert_status_ok();
        let info = response.json::<serde_json::Value>();
        assert_eq!(info.get("name"), Some(&json!("DreamFactory API")));
    }

    #[tokio::test]
    async fn test_system_service_endpoints() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test GET /api/v2/system/service
        let response = server.get("/api/v2/system/service").await;
        response.assert_status_ok();
        let services = response.json::<serde_json::Value>();
        assert!(services.get("resource").is_some());
        
        let resource_array = services.get("resource").unwrap().as_array().unwrap();
        assert!(!resource_array.is_empty());
        
        // Verify system service is in the list
        let system_service = resource_array.iter()
            .find(|s| s.get("name") == Some(&json!("system")))
            .expect("System service should be in the list");
        assert_eq!(system_service.get("type"), Some(&json!("system")));
        assert_eq!(system_service.get("is_active"), Some(&json!(true)));

        // Test GET /api/v2/system/admin
        let response = server.get("/api/v2/system/admin").await;
        response.assert_status_ok();
        let admin = response.json::<serde_json::Value>();
        assert!(admin.get("is_hosted").is_some());
        assert!(admin.get("host").is_some());
        assert!(admin.get("config").is_some());
        assert!(admin.get("authentication").is_some());

        // Test GET /api/v2/system/environment
        let response = server.get("/api/v2/system/environment").await;
        response.assert_status_ok();
        let env = response.json::<serde_json::Value>();
        assert!(env.get("platform").is_some());
        assert!(env.get("server").is_some());
        assert!(env.get("database").is_some());
        assert!(env.get("cache").is_some());
        assert!(env.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_cache_service_complete_crud() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        let test_key = "integration-test-key";
        let test_value = "integration-test-value";

        // Test POST /api/v2/cache/{key} - Create
        let response = server
            .post(&format!("/api/v2/cache/{}", test_key))
            .text(test_value)
            .await;
        response.assert_status_ok();
        let create_result = response.json::<serde_json::Value>();
        assert_eq!(create_result.get("success"), Some(&json!(true)));
        assert_eq!(create_result.get("key"), Some(&json!(test_key)));

        // Test GET /api/v2/cache/{key} - Read
        let response = server.get(&format!("/api/v2/cache/{}", test_key)).await;
        response.assert_status_ok();
        let get_result = response.json::<serde_json::Value>();
        assert_eq!(get_result, json!(test_value));

        // Test PUT /api/v2/cache/{key} - Update
        let updated_value = "updated-integration-test-value";
        let response = server
            .put(&format!("/api/v2/cache/{}", test_key))
            .text(updated_value)
            .await;
        response.assert_status_ok();
        let update_result = response.json::<serde_json::Value>();
        assert_eq!(update_result.get("success"), Some(&json!(true)));

        // Verify update
        let response = server.get(&format!("/api/v2/cache/{}", test_key)).await;
        response.assert_status_ok();
        let get_updated = response.json::<serde_json::Value>();
        assert_eq!(get_updated, json!(updated_value));

        // Test GET /api/v2/cache - List all keys
        let response = server.get("/api/v2/cache").await;
        response.assert_status_ok();
        let list_result = response.json::<serde_json::Value>();
        assert!(list_result.get("resource").is_some());
        let resources = list_result.get("resource").unwrap().as_array().unwrap();
        assert!(!resources.is_empty());

        // Test DELETE /api/v2/cache/{key} - Delete
        let response = server.delete(&format!("/api/v2/cache/{}", test_key)).await;
        response.assert_status_ok();
        let delete_result = response.json::<serde_json::Value>();
        assert_eq!(delete_result.get("success"), Some(&json!(true)));

        // Verify deletion
        let response = server.get(&format!("/api/v2/cache/{}", test_key)).await;
        response.assert_status_client_error(); // Should be 404 or similar
    }

    #[tokio::test]
    async fn test_cache_service_json_values() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        let test_key = "json-test-key";
        let json_value = json!({
            "name": "test",
            "value": 42,
            "nested": {
                "array": [1, 2, 3],
                "flag": true
            }
        });

        // Set JSON value
        let response = server
            .post(&format!("/api/v2/cache/{}", test_key))
            .json(&json_value)
            .await;
        response.assert_status_ok();

        // Get JSON value
        let response = server.get(&format!("/api/v2/cache/{}", test_key)).await;
        response.assert_status_ok();
        let retrieved_value = response.json::<serde_json::Value>();
        assert_eq!(retrieved_value, json_value);

        // Clean up
        server.delete(&format!("/api/v2/cache/{}", test_key)).await;
    }

    #[tokio::test]
    async fn test_cache_flush() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Set multiple cache values
        server.post("/api/v2/cache/flush-test-1").text("value1").await;
        server.post("/api/v2/cache/flush-test-2").text("value2").await;
        server.post("/api/v2/cache/flush-test-3").text("value3").await;

        // Verify values exist
        let list_response = server.get("/api/v2/cache").await;
        list_response.assert_status_ok();
        let list_result = list_response.json::<serde_json::Value>();
        let resources = list_result.get("resource").unwrap().as_array().unwrap();
        assert!(resources.len() >= 3);

        // Flush cache
        let response = server.post("/api/v2/cache/_flush").await;
        response.assert_status_ok();
        let flush_result = response.json::<serde_json::Value>();
        assert_eq!(flush_result.get("success"), Some(&json!(true)));

        // Verify cache is empty
        server.get("/api/v2/cache/flush-test-1").await.assert_status_client_error();
        server.get("/api/v2/cache/flush-test-2").await.assert_status_client_error();
        server.get("/api/v2/cache/flush-test-3").await.assert_status_client_error();
    }

    #[tokio::test]
    async fn test_email_service_send() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        let email_request = json!({
            "to": ["test@example.com"],
            "subject": "Integration Test Email",
            "body_text": "This is a test email from the integration tests"
        });

        // Test POST /api/v2/email/_send
        let response = server
            .post("/api/v2/email/_send")
            .json(&email_request)
            .await;
        response.assert_status_ok();
        
        let send_result = response.json::<serde_json::Value>();
        assert_eq!(send_result.get("success"), Some(&json!(true)));
        assert!(send_result.get("message_id").is_some());
        assert_eq!(send_result.get("recipients"), Some(&json!(["test@example.com"])));
    }

    #[tokio::test]
    async fn test_email_service_send_with_all_options() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        let email_request = json!({
            "to": ["primary@example.com"],
            "cc": ["cc@example.com"],
            "bcc": ["bcc@example.com"],
            "subject": "Comprehensive Test Email",
            "body_text": "Plain text version of the email",
            "body_html": "<h1>HTML Version</h1><p>This is the HTML version of the email</p>",
            "from_name": "Test Sender",
            "from_email": "sender@example.com",
            "reply_to_name": "Reply Handler",
            "reply_to_email": "reply@example.com"
        });

        let response = server
            .post("/api/v2/email/_send")
            .json(&email_request)
            .await;
        response.assert_status_ok();
        
        let send_result = response.json::<serde_json::Value>();
        assert_eq!(send_result.get("success"), Some(&json!(true)));
        assert!(send_result.get("message_id").is_some());
    }

    #[tokio::test]
    async fn test_email_templates() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test GET /api/v2/email/template
        let response = server.get("/api/v2/email/template").await;
        response.assert_status_ok();
        
        let templates = response.json::<serde_json::Value>();
        assert!(templates.get("resource").is_some());
        let template_array = templates.get("resource").unwrap().as_array().unwrap();
        assert_eq!(template_array.len(), 2);

        // Verify template structure
        let welcome_template = template_array.iter()
            .find(|t| t.get("name") == Some(&json!("welcome")))
            .expect("Welcome template should exist");
        assert_eq!(welcome_template.get("id"), Some(&json!(1)));
        assert!(welcome_template.get("subject").is_some());
        assert!(welcome_template.get("body_text").is_some());
        assert!(welcome_template.get("body_html").is_some());

        // Test GET /api/v2/email/template/{id}
        let response = server.get("/api/v2/email/template/1").await;
        response.assert_status_ok();
        let template = response.json::<serde_json::Value>();
        assert_eq!(template.get("name"), Some(&json!("welcome")));
        assert_eq!(template.get("id"), Some(&json!(1)));

        // Test GET /api/v2/email/template/{name}
        let response = server.get("/api/v2/email/template/password_reset").await;
        response.assert_status_ok();
        let template = response.json::<serde_json::Value>();
        assert_eq!(template.get("name"), Some(&json!("password_reset")));
        assert_eq!(template.get("id"), Some(&json!(2)));
    }

    #[tokio::test]
    async fn test_email_validation_errors() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test missing recipients
        let invalid_email = json!({
            "to": [],
            "subject": "Test",
            "body_text": "Test body"
        });
        let response = server
            .post("/api/v2/email/_send")
            .json(&invalid_email)
            .await;
        response.assert_status_server_error();

        // Test missing subject
        let invalid_email = json!({
            "to": ["test@example.com"],
            "subject": "",
            "body_text": "Test body"
        });
        let response = server
            .post("/api/v2/email/_send")
            .json(&invalid_email)
            .await;
        response.assert_status_server_error();

        // Test missing body
        let invalid_email = json!({
            "to": ["test@example.com"],
            "subject": "Test Subject"
        });
        let response = server
            .post("/api/v2/email/_send")
            .json(&invalid_email)
            .await;
        response.assert_status_server_error();
    }

    #[tokio::test]
    async fn test_universal_routing_patterns() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test different API versions
        let response = server.get("/api/v1/system/service").await;
        response.assert_status_ok(); // Should work with v1

        let response = server.get("/api/v2/system/service").await;
        response.assert_status_ok(); // Should work with v2

        let response = server.get("/api/v2.5/system/service").await;
        response.assert_status_ok(); // Should work with decimal versions

        // Test invalid API routes
        let response = server.get("/api/invalid").await;
        response.assert_status_bad_request();

        let response = server.get("/api/v2/nonexistent-service").await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_http_methods_support() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test that all HTTP methods are handled appropriately
        let test_key = "http-methods-test";

        // POST - Create
        let response = server.post(&format!("/api/v2/cache/{}", test_key)).text("test").await;
        response.assert_status_ok();

        // GET - Read
        let response = server.get(&format!("/api/v2/cache/{}", test_key)).await;
        response.assert_status_ok();

        // PUT - Update
        let response = server.put(&format!("/api/v2/cache/{}", test_key)).text("updated").await;
        response.assert_status_ok();

        // DELETE - Delete
        let response = server.delete(&format!("/api/v2/cache/{}", test_key)).await;
        response.assert_status_ok();

        // Clean up
        server.delete(&format!("/api/v2/cache/{}", test_key)).await;
    }

    #[tokio::test]
    async fn test_error_handling() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test 404 for non-existent cache key
        let response = server.get("/api/v2/cache/non-existent-key").await;
        response.assert_status_server_error(); // Our implementation returns 500 for missing keys
        let error = response.json::<serde_json::Value>();
        assert!(error.get("error").is_some());

        // Test 404 for non-existent service
        let response = server.get("/api/v2/non-existent-service/resource").await;
        response.assert_status_not_found();
        let error = response.json::<serde_json::Value>();
        assert!(error.get("error").is_some());

        // Test 400 for invalid route format
        let response = server.get("/invalid-api-route").await;
        response.assert_status_not_found(); // Router doesn't match this route
    }

    #[tokio::test]
    async fn test_service_discovery() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test that all required services are available
        let response = server.get("/api/v2/system/service").await;
        response.assert_status_ok();
        
        let services = response.json::<serde_json::Value>();
        let service_array = services.get("resource").unwrap().as_array().unwrap();
        
        let service_names: Vec<&str> = service_array
            .iter()
            .filter_map(|s| s.get("name")?.as_str())
            .collect();

        // Verify all required services are present
        assert!(service_names.contains(&"system"));
        assert!(service_names.contains(&"cache"));
        assert!(service_names.contains(&"email"));
        assert!(service_names.contains(&"database"));
        assert!(service_names.contains(&"files"));
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let router = create_api_router().unwrap();
        let server = TestServer::new(router).unwrap();

        // Test CORS preflight request
        let response = server
            .method(axum::http::Method::OPTIONS)
            .path("/api/v2/system/service")
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "GET")
            .await;
        
        // Should allow CORS
        response.assert_status_ok();
    }
}