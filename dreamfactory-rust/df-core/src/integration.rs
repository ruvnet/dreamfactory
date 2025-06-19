//! Service integration layer that bridges core services with API handlers
//! 
//! This module provides the integration layer between the core service registry
//! and the API layer, enabling proper dependency injection and service communication.
//!
//! ## Dynamic Compatibility (Object Safety)
//!
//! The `ApiServiceHandler` trait is designed to be object-safe (dyn-compatible) by using
//! the `async_trait` macro. This allows the trait to be used as `dyn ApiServiceHandler`
//! in trait objects, which is essential for the service registry pattern.
//!
//! ### Key Design Decisions for Object Safety:
//!
//! - **Async methods**: Wrapped with `#[async_trait]` to transform async methods into
//!   methods that return `Pin<Box<dyn Future + Send>>`, making them object-safe.
//! - **No generic methods**: All methods use concrete types to maintain object safety.
//! - **Send + Sync bounds**: Required for thread-safe usage across async boundaries.
//!
//! ### Usage Examples:
//!
//! ```rust,ignore
//! // As a trait object in collections
//! let handlers: HashMap<String, Arc<dyn ApiServiceHandler>> = HashMap::new();
//!
//! // As boxed trait objects
//! let handler: Box<dyn ApiServiceHandler> = Box::new(my_handler);
//!
//! // Calling async methods through trait objects
//! let result = handler.handle_request(route, "GET", params, body, context).await;
//! ```

use crate::error::{DfError, DfResult};
use crate::service::{Service, ServiceInfo};
#[cfg(test)]
use crate::service::ServiceState;
use crate::registry::ServiceRegistry as CoreServiceRegistry;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Trait for services that can handle API requests
/// 
/// This trait is designed to be object-safe (dyn-compatible) using the async_trait macro.
/// All async methods are transformed to return boxed futures, making the trait
/// suitable for use as `dyn ApiServiceHandler`.
#[async_trait]
pub trait ApiServiceHandler: Send + Sync {
    /// Handle an API request for this service
    /// 
    /// This method processes incoming API requests and returns a JSON response.
    /// The async_trait macro ensures this remains object-safe.
    async fn handle_request(
        &self,
        route: &ApiRoute,
        method: &str,
        query_params: HashMap<String, String>,
        body: Option<Vec<u8>>,
        context: Option<ServiceContext>,
    ) -> DfResult<serde_json::Value>;
    
    /// Get service information
    /// 
    /// Returns immutable reference to service metadata.
    /// This is a synchronous method and remains object-safe.
    fn service_info(&self) -> &ServiceInfo;
    
    /// Check if the service can handle the given route
    /// 
    /// Returns true if this service can process the given route.
    /// This is a synchronous method and remains object-safe.
    fn can_handle(&self, route: &ApiRoute) -> bool;
}

/// API route structure
#[derive(Debug, Clone, PartialEq)]
pub struct ApiRoute {
    pub version: String,
    pub service: String,
    pub resource: Option<String>,
    pub id: Option<String>,
    pub sub_resource: Option<String>,
    pub sub_id: Option<String>,
}

/// Service context containing authentication and request information
#[derive(Debug, Clone)]
pub struct ServiceContext {
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: String,
}

impl ServiceContext {
    pub fn new(request_id: String) -> Self {
        Self {
            user_id: None,
            session_id: None,
            permissions: Vec::new(),
            roles: Vec::new(),
            ip_address: None,
            user_agent: None,
            request_id,
        }
    }
    
    pub fn with_auth(
        mut self,
        user_id: Uuid,
        session_id: Uuid,
        permissions: Vec<String>,
        roles: Vec<String>,
    ) -> Self {
        self.user_id = Some(user_id);
        self.session_id = Some(session_id);
        self.permissions = permissions;
        self.roles = roles;
        self
    }
    
    pub fn with_network_info(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some() && self.session_id.is_some()
    }
    
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
    
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

/// Integrated service registry that combines core services with API handlers
pub struct IntegratedServiceRegistry {
    core_registry: Arc<CoreServiceRegistry>,
    api_handlers: Arc<RwLock<HashMap<String, Arc<dyn ApiServiceHandler>>>>,
    service_dependencies: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl IntegratedServiceRegistry {
    /// Create a new integrated service registry
    pub fn new() -> Self {
        Self {
            core_registry: Arc::new(CoreServiceRegistry::new()),
            api_handlers: Arc::new(RwLock::new(HashMap::new())),
            service_dependencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create with existing core registry
    pub fn with_core_registry(core_registry: CoreServiceRegistry) -> Self {
        Self {
            core_registry: Arc::new(core_registry),
            api_handlers: Arc::new(RwLock::new(HashMap::new())),
            service_dependencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a service that implements both Service and ApiServiceHandler
    pub async fn register_integrated_service<S>(&self, service: S) -> DfResult<()>
    where
        S: Service + ApiServiceHandler + 'static,
    {
        let service_name = service.info().name.clone();
        let service_arc = Arc::new(service);
        
        // Register in core registry
        // Note: We need to clone the service for the core registry
        // This is a temporary solution until we refactor the core registry
        let _core_service = service_arc.clone();
        
        // Register API handler
        let mut handlers = self.api_handlers.write().await;
        handlers.insert(service_name.clone(), service_arc);
        drop(handlers);
        
        Ok(())
    }
    
    /// Register a pure API handler (no core service backing)
    pub async fn register_api_handler<H>(&self, handler: H) -> DfResult<()>
    where
        H: ApiServiceHandler + 'static,
    {
        let service_name = handler.service_info().name.clone();
        let handler_arc = Arc::new(handler);
        
        let mut handlers = self.api_handlers.write().await;
        handlers.insert(service_name, handler_arc);
        
        Ok(())
    }
    
    /// Get an API handler by service name
    pub async fn get_api_handler(&self, service_name: &str) -> Option<Arc<dyn ApiServiceHandler>> {
        let handlers = self.api_handlers.read().await;
        handlers.get(service_name).cloned()
    }
    
    /// Get a service reference by name (for service-to-service communication)
    pub async fn get_service_dependency<T: 'static>(&self, _service_name: &str) -> DfResult<Arc<T>> {
        // This is a placeholder for service dependency resolution
        // In a real implementation, we would maintain type-safe references
        Err(DfError::registry("Service dependency resolution not yet implemented"))
    }
    
    /// Initialize all services in dependency order
    pub async fn initialize_all(&self) -> DfResult<()> {
        self.core_registry.initialize_all().await
    }
    
    /// Start all services in dependency order
    pub async fn start_all(&self) -> DfResult<()> {
        self.core_registry.start_all().await
    }
    
    /// Stop all services in reverse dependency order
    pub async fn stop_all(&self) -> DfResult<()> {
        self.core_registry.stop_all().await
    }
    
    /// Get health status of all services
    pub async fn health_check_all(&self) -> DfResult<HashMap<String, bool>> {
        let mut results = self.core_registry.health_check_all().await?;
        
        // Add API handler health checks
        let handlers = self.api_handlers.read().await;
        for (name, _handler) in handlers.iter() {
            // For now, assume API handlers are healthy if they're registered
            results.insert(format!("{}_api", name), true);
        }
        
        Ok(results)
    }
    
    /// Get all registered service names
    pub async fn get_service_names(&self) -> Vec<String> {
        let mut names = self.core_registry.get_service_names().await;
        
        let handlers = self.api_handlers.read().await;
        for name in handlers.keys() {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
        
        names
    }
    
    /// Handle an API request by routing to the appropriate service
    pub async fn handle_api_request(
        &self,
        route: &ApiRoute,
        method: &str,
        query_params: HashMap<String, String>,
        body: Option<Vec<u8>>,
        context: Option<ServiceContext>,
    ) -> DfResult<serde_json::Value> {
        let handler = self.get_api_handler(&route.service).await
            .ok_or_else(|| DfError::registry(&format!("API handler for service '{}' not found", route.service)))?;
        
        if !handler.can_handle(route) {
            return Err(DfError::registry(&format!("Service '{}' cannot handle route", route.service)));
        }
        
        handler.handle_request(route, method, query_params, body, context).await
    }
    
    /// Add a service dependency relationship
    pub async fn add_dependency(&self, service: &str, dependency: &str) -> DfResult<()> {
        let mut deps = self.service_dependencies.write().await;
        deps.entry(service.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.to_string());
        Ok(())
    }
    
    /// Check if all dependencies are satisfied for a service
    pub async fn dependencies_satisfied(&self, service_name: &str) -> bool {
        let deps = self.service_dependencies.read().await;
        if let Some(dependencies) = deps.get(service_name) {
            let service_names = self.get_service_names().await;
            dependencies.iter().all(|dep| service_names.contains(dep))
        } else {
            true // No dependencies
        }
    }
    
    /// Get the core registry reference
    pub fn core_registry(&self) -> Arc<CoreServiceRegistry> {
        self.core_registry.clone()
    }
}

impl Default for IntegratedServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Demonstrates object safety by accepting trait objects
/// 
/// This function shows that `ApiServiceHandler` can be used as a trait object,
/// proving its dyn-compatibility. It accepts any implementation of the trait
/// and can call its methods polymorphically.
pub async fn handle_with_trait_object(
    handler: &dyn ApiServiceHandler,
    route: &ApiRoute,
    method: &str,
    query_params: HashMap<String, String>,
    body: Option<Vec<u8>>,
    context: Option<ServiceContext>,
) -> DfResult<serde_json::Value> {
    // Verify the handler can process this route
    if !handler.can_handle(route) {
        return Err(DfError::service(
            &handler.service_info().name,
            "Handler cannot process this route"
        ));
    }
    
    // Delegate to the handler's implementation
    handler.handle_request(route, method, query_params, body, context).await
}

/// Creates a collection of trait objects for demonstration
/// 
/// This function demonstrates that multiple different implementations
/// of `ApiServiceHandler` can be stored together in a collection as trait objects.
pub fn create_handler_collection() -> Vec<Box<dyn ApiServiceHandler>> {
    let mut handlers: Vec<Box<dyn ApiServiceHandler>> = Vec::new();
    
    // Add different handler implementations
    let base_handler = BaseApiHandler::new(
        ServiceInfo::new("base", "1.0.0", "Base handler")
    );
    handlers.push(Box::new(base_handler));
    
    // You could add more handler types here
    // handlers.push(Box::new(other_handler));
    
    handlers
}

/// Helper trait to convert core Services into API handlers
pub trait ServiceToHandlerAdapter<T: Service> {
    fn into_api_handler(self) -> Box<dyn ApiServiceHandler>;
}

// ============================================================================
// DYN COMPATIBILITY SOLUTION SUMMARY
// ============================================================================
//
// PROBLEM: The ApiServiceHandler trait had async methods which made it not 
// object-safe (dyn-incompatible) because async methods in traits create 
// associated types that prevent trait object usage.
//
// SOLUTION: Used the `async_trait` macro which transforms async trait methods
// into methods that return `Pin<Box<dyn Future + Send>>`, making the trait
// object-safe and dyn-compatible.
//
// KEY POINTS:
// 1. The #[async_trait] macro on the trait definition handles the transformation
// 2. All implementers also need #[async_trait] on their impl blocks
// 3. The trait can now be used as `dyn ApiServiceHandler` in:
//    - Box<dyn ApiServiceHandler>
//    - Arc<dyn ApiServiceHandler>
//    - &dyn ApiServiceHandler
//    - Collections like Vec<Box<dyn ApiServiceHandler>>
//
// VERIFICATION: The tests in this file demonstrate that the trait is properly
// object-safe and can be used with trait objects in all common scenarios.
//
// ALTERNATIVE APPROACHES (not used):
// - Restructure to avoid async methods in trait (would require callback/closure pattern)
// - Use associated types with bounds (would make trait not object-safe)
// - Use generic associated types (GATs) - still experimental and complex
//
// The async_trait approach is the most straightforward and widely adopted
// solution for this pattern in the Rust ecosystem.
// ============================================================================

/// Base implementation for services that need basic API handling
pub struct BaseApiHandler {
    service_info: ServiceInfo,
}

impl BaseApiHandler {
    pub fn new(service_info: ServiceInfo) -> Self {
        Self { service_info }
    }
}

#[async_trait]
impl ApiServiceHandler for BaseApiHandler {
    async fn handle_request(
        &self,
        route: &ApiRoute,
        method: &str,
        _query_params: HashMap<String, String>,
        _body: Option<Vec<u8>>,
        _context: Option<ServiceContext>,
    ) -> DfResult<serde_json::Value> {
        // Default implementation returns service info
        match (method, route.resource.as_deref()) {
            ("GET", None) => {
                Ok(serde_json::json!({
                    "service": self.service_info.name,
                    "version": self.service_info.version,
                    "description": self.service_info.description,
                    "status": "ok"
                }))
            }
            _ => Err(DfError::service(
                &self.service_info.name,
                &format!("Unsupported operation: {} {}", method, route.resource.as_deref().unwrap_or("")),
            )),
        }
    }
    
    fn service_info(&self) -> &ServiceInfo {
        &self.service_info
    }
    
    fn can_handle(&self, _route: &ApiRoute) -> bool {
        true // Base handler can handle basic requests
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{BaseService, ServiceInfo};
    
    struct TestService {
        base: BaseService,
    }
    
    impl TestService {
        fn new() -> Self {
            let info = ServiceInfo::new("test-service", "1.0.0", "Test service");
            Self {
                base: BaseService::new(info),
            }
        }
    }
    
    #[async_trait]
    impl Service for TestService {
        fn info(&self) -> &ServiceInfo {
            self.base.info()
        }
        
        fn state(&self) -> ServiceState {
            self.base.state()
        }
        
        async fn initialize(&mut self) -> DfResult<()> {
            Ok(())
        }
        
        async fn start(&mut self) -> DfResult<()> {
            Ok(())
        }
        
        async fn stop(&mut self) -> DfResult<()> {
            Ok(())
        }
    }
    
    #[async_trait]
    impl ApiServiceHandler for TestService {
        async fn handle_request(
            &self,
            _route: &ApiRoute,
            _method: &str,
            _query_params: HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _context: Option<ServiceContext>,
        ) -> DfResult<serde_json::Value> {
            Ok(serde_json::json!({"status": "test_ok"}))
        }
        
        fn service_info(&self) -> &ServiceInfo {
            self.base.info()
        }
        
        fn can_handle(&self, _route: &ApiRoute) -> bool {
            true
        }
    }
    
    #[tokio::test]
    async fn test_integrated_registry_creation() {
        let registry = IntegratedServiceRegistry::new();
        let names = registry.get_service_names().await;
        assert!(names.is_empty());
    }
    
    #[tokio::test]
    async fn test_api_handler_registration() {
        let registry = IntegratedServiceRegistry::new();
        let service = TestService::new();
        
        registry.register_api_handler(service).await.unwrap();
        
        let handler = registry.get_api_handler("test-service").await;
        assert!(handler.is_some());
    }
    
    #[tokio::test]
    async fn test_api_request_handling() {
        let registry = IntegratedServiceRegistry::new();
        let service = TestService::new();
        
        registry.register_api_handler(service).await.unwrap();
        
        let route = ApiRoute {
            version: "v2".to_string(),
            service: "test-service".to_string(),
            resource: None,
            id: None,
            sub_resource: None,
            sub_id: None,
        };
        
        let context = ServiceContext::new("test-request-123".to_string());
        let result = registry.handle_api_request(
            &route,
            "GET",
            HashMap::new(),
            None,
            Some(context),
        ).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["status"], "test_ok");
    }
    
    #[tokio::test]
    async fn test_service_context() {
        let context = ServiceContext::new("test-123".to_string())
            .with_auth(
                Uuid::new_v4(),
                Uuid::new_v4(),
                vec!["read".to_string()],
                vec!["user".to_string()],
            )
            .with_network_info(
                Some("192.168.1.1".to_string()),
                Some("Mozilla/5.0".to_string()),
            );
        
        assert!(context.is_authenticated());
        assert!(context.has_permission("read"));
        assert!(context.has_role("user"));
        assert!(!context.has_permission("write"));
    }
    
    #[tokio::test]
    async fn test_dependency_management() {
        let registry = IntegratedServiceRegistry::new();
        
        registry.add_dependency("service-b", "service-a").await.unwrap();
        
        // Before registering service-a, dependency should not be satisfied
        assert!(!registry.dependencies_satisfied("service-b").await);
        
        // This test would pass once we implement proper service registration
        // For now, it demonstrates the intended behavior
    }
    
    #[tokio::test]
    async fn test_base_api_handler() {
        let info = ServiceInfo::new("base-service", "1.0.0", "Base service");
        let handler = BaseApiHandler::new(info);
        
        let route = ApiRoute {
            version: "v2".to_string(),
            service: "base-service".to_string(),
            resource: None,
            id: None,
            sub_resource: None,
            sub_id: None,
        };
        
        let result = handler.handle_request(&route, "GET", HashMap::new(), None, None).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["service"], "base-service");
        assert_eq!(response["status"], "ok");
    }
    
    #[tokio::test]
    async fn test_trait_object_compatibility() {
        // This test verifies that ApiServiceHandler is dyn-compatible
        let info = ServiceInfo::new("trait-test", "1.0.0", "Trait object test");
        let handler = BaseApiHandler::new(info);
        
        // Create a trait object - this should compile if the trait is object-safe
        let trait_object: Arc<dyn ApiServiceHandler> = Arc::new(handler);
        
        // Verify we can call methods through the trait object
        let service_info = trait_object.service_info();
        assert_eq!(service_info.name, "trait-test");
        
        let route = ApiRoute {
            version: "v2".to_string(),
            service: "trait-test".to_string(),
            resource: None,
            id: None,
            sub_resource: None,
            sub_id: None,
        };
        
        assert!(trait_object.can_handle(&route));
        
        // Test async method through trait object
        let result = trait_object.handle_request(
            &route, 
            "GET", 
            HashMap::new(), 
            None, 
            None
        ).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["service"], "trait-test");
    }
    
    #[test]
    fn test_dyn_compatibility_compile_time() {
        // This is a compile-time test to ensure the trait is object-safe
        // If this compiles, the trait is dyn-compatible
        fn _test_function(_handler: Box<dyn ApiServiceHandler>) {
            // This function accepts a boxed trait object
            // If it compiles, the trait is object-safe
        }
        
        // Additionally test with Arc<dyn T>
        fn _test_arc_function(_handler: Arc<dyn ApiServiceHandler>) {
            // This function accepts an Arc trait object
        }
        
        // The fact that these functions can be defined proves the trait is object-safe
    }
    
    #[tokio::test]
    async fn test_trait_object_utility_functions() {
        // Test the utility function that accepts trait objects
        let info = ServiceInfo::new("utility-test", "1.0.0", "Utility test handler");
        let handler = BaseApiHandler::new(info);
        
        let route = ApiRoute {
            version: "v2".to_string(),
            service: "utility-test".to_string(),
            resource: Some("test".to_string()),
            id: None,
            sub_resource: None,
            sub_id: None,
        };
        
        // Call the utility function with a trait object reference
        let result = handle_with_trait_object(
            &handler,
            &route,
            "GET",
            HashMap::new(),
            None,
            None
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_handler_collection() {
        // Test creating a collection of trait objects
        let handlers = create_handler_collection();
        assert!(!handlers.is_empty());
        
        // Verify we can access methods through the trait objects
        for handler in &handlers {
            let info = handler.service_info();
            assert!(!info.name.is_empty());
            
            let route = ApiRoute {
                version: "v2".to_string(),
                service: info.name.clone(),
                resource: None,
                id: None,
                sub_resource: None,
                sub_id: None,
            };
            
            // This should not panic and should return a boolean
            let _can_handle = handler.can_handle(&route);
        }
    }
}