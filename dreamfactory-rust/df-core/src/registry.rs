//! Service registry and dependency injection container for DreamFactory.
//!
//! This module provides a centralized registry for managing services and their dependencies,
//! with automatic dependency resolution and lifecycle management.

use crate::error::{DfError, DfResult};
use crate::service::{Service, ServiceInfo, ServiceState};
use async_trait::async_trait;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry-specific error types
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Service not found: {name}")]
    ServiceNotFound { name: String },
    
    #[error("Service already registered: {name}")]
    ServiceAlreadyRegistered { name: String },
    
    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },
    
    #[error("Dependency not satisfied: {service} depends on {dependency}")]
    DependencyNotSatisfied { service: String, dependency: String },
    
    #[error("Service type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Invalid service state for operation: {state:?}")]
    InvalidState { state: ServiceState },
}

impl From<RegistryError> for DfError {
    fn from(err: RegistryError) -> Self {
        DfError::registry(err.to_string())
    }
}

/// Service container entry
struct ServiceEntry {
    service: Box<dyn Service>,
    type_id: TypeId,
    type_name: String,
}

impl ServiceEntry {
    /// Get the service type ID
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Get the service type name
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Check if this service matches the given type
    pub fn is_type<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
}

/// Service factory trait for creating services
#[async_trait]
pub trait ServiceFactory: Send + Sync {
    /// Create a new service instance
    async fn create(&self) -> DfResult<Box<dyn Service>>;
    
    /// Get the service type name
    fn type_name(&self) -> &str;
    
    /// Get the service type ID
    fn type_id(&self) -> TypeId;
}

/// Implementation of ServiceFactory for closures
pub struct ClosureServiceFactory<F, T>
where
    F: Fn() -> T + Send + Sync,
    T: Service + 'static,
{
    factory_fn: F,
    type_name: String,
}

impl<F, T> ClosureServiceFactory<F, T>
where
    F: Fn() -> T + Send + Sync,
    T: Service + 'static,
{
    pub fn new(factory_fn: F) -> Self {
        Self {
            factory_fn,
            type_name: std::any::type_name::<T>().to_string(),
        }
    }
}

#[async_trait]
impl<F, T> ServiceFactory for ClosureServiceFactory<F, T>
where
    F: Fn() -> T + Send + Sync,
    T: Service + 'static,
{
    async fn create(&self) -> DfResult<Box<dyn Service>> {
        let service = (self.factory_fn)();
        Ok(Box::new(service))
    }
    
    fn type_name(&self) -> &str {
        &self.type_name
    }
    
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// Service registry for managing services and their dependencies
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceEntry>>>,
    factories: Arc<RwLock<HashMap<String, Box<dyn ServiceFactory>>>>,
    dependency_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
    startup_order: Arc<RwLock<Vec<String>>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(HashMap::new())),
            startup_order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a service instance
    pub async fn register_service<S: Service + 'static>(&self, service: S) -> DfResult<()> {
        let info = service.info().clone();
        let name = info.name.clone();
        
        let mut services = self.services.write().await;
        
        if services.contains_key(&name) {
            return Err(RegistryError::ServiceAlreadyRegistered { name }.into());
        }

        let entry = ServiceEntry {
            service: Box::new(service),
            type_id: TypeId::of::<S>(),
            type_name: std::any::type_name::<S>().to_string(),
        };

        services.insert(name.clone(), entry);
        
        // Update dependency graph
        let mut graph = self.dependency_graph.write().await;
        graph.insert(name, info.dependencies);
        
        drop(services);
        drop(graph);
        
        // Recalculate startup order
        self.calculate_startup_order().await?;
        
        Ok(())
    }

    /// Register a service factory
    pub async fn register_factory<F>(&self, name: String, factory: F) -> DfResult<()>
    where
        F: ServiceFactory + 'static,
    {
        let mut factories = self.factories.write().await;
        factories.insert(name, Box::new(factory));
        Ok(())
    }

    /// Register a service factory using a closure
    pub async fn register_closure_factory<F, T>(&self, name: String, factory_fn: F) -> DfResult<()>
    where
        F: Fn() -> T + Send + Sync + 'static,
        T: Service + 'static,
    {
        let factory = ClosureServiceFactory::new(factory_fn);
        self.register_factory(name, factory).await
    }

    /// Get a service by name
    pub async fn get_service(&self, name: &str) -> DfResult<Box<dyn Service>> {
        let services = self.services.read().await;
        
        if let Some(_entry) = services.get(name) {
            // We can't clone the service directly, so we'll return an error for now
            // In a real implementation, we might want to use Arc<Mutex<>> or similar
            return Err(DfError::registry("Cannot clone service - use get_service_ref instead"));
        }
        
        drop(services);
        
        // Try to create from factory
        let factories = self.factories.read().await;
        if let Some(factory) = factories.get(name) {
            let service = factory.create().await?;
            drop(factories);
            
            // Register the created service
            let _info = service.info().clone();
            self.register_service_boxed(service).await?;
            
            // Return a reference to the registered service
            self.get_service_ref(name).await
        } else {
            Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
        }
    }

    /// Get a service reference by name (internal helper)
    async fn get_service_ref(&self, _name: &str) -> DfResult<Box<dyn Service>> {
        Err(DfError::registry("get_service_ref not implemented - services are owned by registry"))
    }

    /// Register a boxed service (internal helper)
    async fn register_service_boxed(&self, service: Box<dyn Service>) -> DfResult<()> {
        let info = service.info().clone();
        let name = info.name.clone();
        
        let mut services = self.services.write().await;
        
        if services.contains_key(&name) {
            return Err(RegistryError::ServiceAlreadyRegistered { name }.into());
        }

        let entry = ServiceEntry {
            service,
            type_id: TypeId::of::<()>(), // We don't know the concrete type
            type_name: "Unknown".to_string(),
        };

        services.insert(name.clone(), entry);
        
        // Update dependency graph
        let mut graph = self.dependency_graph.write().await;
        graph.insert(name, info.dependencies);
        
        drop(services);
        drop(graph);
        
        // Recalculate startup order
        self.calculate_startup_order().await?;
        
        Ok(())
    }

    /// Get a typed service by name
    pub async fn get_typed_service<T: Service + 'static>(&self, _name: &str) -> DfResult<Option<&T>> {
        // This would require unsafe code to return a proper reference
        // For now, we'll return an error indicating this isn't implemented
        Err(DfError::registry("get_typed_service not implemented - use service IDs or different architecture"))
    }

    /// Check if a service is registered
    pub async fn has_service(&self, name: &str) -> bool {
        let services = self.services.read().await;
        services.contains_key(name) || {
            drop(services);
            let factories = self.factories.read().await;
            factories.contains_key(name)
        }
    }

    /// Get all registered service names
    pub async fn get_service_names(&self) -> Vec<String> {
        let services = self.services.read().await;
        let factories = self.factories.read().await;
        
        let mut names: HashSet<String> = services.keys().cloned().collect();
        names.extend(factories.keys().cloned());
        names.into_iter().collect()
    }

    /// Get service information by name
    pub async fn get_service_info(&self, name: &str) -> DfResult<ServiceInfo> {
        let services = self.services.read().await;
        if let Some(entry) = services.get(name) {
            Ok(entry.service.info().clone())
        } else {
            Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
        }
    }

    /// Get service type information by name
    pub async fn get_service_type_info(&self, name: &str) -> DfResult<(TypeId, String)> {
        let services = self.services.read().await;
        if let Some(entry) = services.get(name) {
            Ok((entry.type_id(), entry.type_name().to_string()))
        } else {
            Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
        }
    }

    /// Check if a service matches a specific type
    pub async fn is_service_type<T: 'static>(&self, name: &str) -> DfResult<bool> {
        let services = self.services.read().await;
        if let Some(entry) = services.get(name) {
            Ok(entry.is_type::<T>())
        } else {
            Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
        }
    }

    /// Get service state by name
    pub async fn get_service_state(&self, name: &str) -> DfResult<ServiceState> {
        let services = self.services.read().await;
        if let Some(entry) = services.get(name) {
            Ok(entry.service.state())
        } else {
            Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
        }
    }

    /// Initialize all services in dependency order
    pub async fn initialize_all(&self) -> DfResult<()> {
        let startup_order = self.startup_order.read().await.clone();
        
        for service_name in &startup_order {
            self.initialize_service(service_name).await?;
        }
        
        Ok(())
    }

    /// Initialize a specific service
    pub async fn initialize_service(&self, name: &str) -> DfResult<()> {
        let mut services = self.services.write().await;
        if let Some(entry) = services.get_mut(name) {
            entry.service.initialize().await
        } else {
            // Try to create from factory first
            drop(services);
            self.get_service(name).await?;
            
            let mut services = self.services.write().await;
            if let Some(entry) = services.get_mut(name) {
                entry.service.initialize().await
            } else {
                Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
            }
        }
    }

    /// Start all services in dependency order
    pub async fn start_all(&self) -> DfResult<()> {
        let startup_order = self.startup_order.read().await.clone();
        
        for service_name in &startup_order {
            self.start_service(service_name).await?;
        }
        
        Ok(())
    }

    /// Start a specific service
    pub async fn start_service(&self, name: &str) -> DfResult<()> {
        let mut services = self.services.write().await;
        if let Some(entry) = services.get_mut(name) {
            entry.service.start().await
        } else {
            Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
        }
    }

    /// Stop all services in reverse dependency order
    pub async fn stop_all(&self) -> DfResult<()> {
        let startup_order = self.startup_order.read().await.clone();
        
        // Stop in reverse order
        for service_name in startup_order.iter().rev() {
            if let Err(e) = self.stop_service(service_name).await {
                tracing::error!("Failed to stop service {}: {}", service_name, e);
                // Continue stopping other services
            }
        }
        
        Ok(())
    }

    /// Stop a specific service
    pub async fn stop_service(&self, name: &str) -> DfResult<()> {
        let mut services = self.services.write().await;
        if let Some(entry) = services.get_mut(name) {
            entry.service.stop().await
        } else {
            Err(RegistryError::ServiceNotFound { name: name.to_string() }.into())
        }
    }

    /// Perform health check on all services
    pub async fn health_check_all(&self) -> DfResult<HashMap<String, bool>> {
        let services = self.services.read().await;
        let mut results = HashMap::new();
        
        for (name, entry) in services.iter() {
            match entry.service.health_check().await {
                Ok(healthy) => {
                    results.insert(name.clone(), healthy);
                }
                Err(_) => {
                    results.insert(name.clone(), false);
                }
            }
        }
        
        Ok(results)
    }

    /// Calculate startup order based on dependencies
    async fn calculate_startup_order(&self) -> DfResult<()> {
        let graph = self.dependency_graph.read().await;
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut order = Vec::new();
        
        // Topological sort with cycle detection
        for service_name in graph.keys() {
            if !visited.contains(service_name) {
                self.visit_service(
                    service_name,
                    &graph,
                    &mut visited,
                    &mut visiting,
                    &mut order,
                )?;
            }
        }
        
        let mut startup_order = self.startup_order.write().await;
        *startup_order = order;
        
        Ok(())
    }

    /// Recursive helper for topological sort
    fn visit_service(
        &self,
        service_name: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> DfResult<()> {
        if visiting.contains(service_name) {
            // Cycle detected
            let cycle = vec![service_name.to_string()];
            return Err(RegistryError::CircularDependency { cycle }.into());
        }
        
        if visited.contains(service_name) {
            return Ok(());
        }
        
        visiting.insert(service_name.to_string());
        
        if let Some(dependencies) = graph.get(service_name) {
            for dep in dependencies {
                if !graph.contains_key(dep) {
                    return Err(RegistryError::DependencyNotSatisfied {
                        service: service_name.to_string(),
                        dependency: dep.clone(),
                    }.into());
                }
                
                self.visit_service(dep, graph, visited, visiting, order)?;
            }
        }
        
        visiting.remove(service_name);
        visited.insert(service_name.to_string());
        order.push(service_name.to_string());
        
        Ok(())
    }

    /// Get startup order
    pub async fn get_startup_order(&self) -> Vec<String> {
        self.startup_order.read().await.clone()
    }

    /// Get dependency graph
    pub async fn get_dependency_graph(&self) -> HashMap<String, Vec<String>> {
        self.dependency_graph.read().await.clone()
    }

    /// Remove a service from the registry
    pub async fn remove_service(&self, name: &str) -> DfResult<()> {
        let mut services = self.services.write().await;
        if services.remove(name).is_none() {
            return Err(RegistryError::ServiceNotFound { name: name.to_string() }.into());
        }
        
        let mut graph = self.dependency_graph.write().await;
        graph.remove(name);
        
        drop(services);
        drop(graph);
        
        // Recalculate startup order
        self.calculate_startup_order().await?;
        
        Ok(())
    }

    /// Clear all services
    pub async fn clear(&self) {
        let mut services = self.services.write().await;
        let mut factories = self.factories.write().await;
        let mut graph = self.dependency_graph.write().await;
        let mut order = self.startup_order.write().await;
        
        services.clear();
        factories.clear();
        graph.clear();
        order.clear();
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{BaseService, ServiceInfo};
    use tokio_test;

    // Test service implementations
    struct TestServiceA {
        base: BaseService,
    }

    impl TestServiceA {
        fn new() -> Self {
            let info = ServiceInfo::new("service-a", "1.0.0", "Test service A");
            Self {
                base: BaseService::new(info),
            }
        }
    }

    #[async_trait]
    impl Service for TestServiceA {
        fn info(&self) -> &ServiceInfo {
            self.base.info()
        }

        fn state(&self) -> ServiceState {
            self.base.state()
        }

        async fn initialize(&mut self) -> DfResult<()> {
            self.base.initialize().await
        }

        async fn start(&mut self) -> DfResult<()> {
            self.base.start().await
        }

        async fn stop(&mut self) -> DfResult<()> {
            self.base.stop().await
        }
    }

    struct TestServiceB {
        base: BaseService,
    }

    impl TestServiceB {
        fn new() -> Self {
            let info = ServiceInfo::new("service-b", "1.0.0", "Test service B")
                .with_dependency("service-a");
            Self {
                base: BaseService::new(info),
            }
        }
    }

    #[async_trait]
    impl Service for TestServiceB {
        fn info(&self) -> &ServiceInfo {
            self.base.info()
        }

        fn state(&self) -> ServiceState {
            self.base.state()
        }

        async fn initialize(&mut self) -> DfResult<()> {
            self.base.initialize().await
        }

        async fn start(&mut self) -> DfResult<()> {
            self.base.start().await
        }

        async fn stop(&mut self) -> DfResult<()> {
            self.base.stop().await
        }
    }

    struct TestServiceC {
        base: BaseService,
    }

    impl TestServiceC {
        fn new() -> Self {
            let info = ServiceInfo::new("service-c", "1.0.0", "Test service C")
                .with_dependencies(["service-a", "service-b"]);
            Self {
                base: BaseService::new(info),
            }
        }
    }

    #[async_trait]
    impl Service for TestServiceC {
        fn info(&self) -> &ServiceInfo {
            self.base.info()
        }

        fn state(&self) -> ServiceState {
            self.base.state()
        }

        async fn initialize(&mut self) -> DfResult<()> {
            self.base.initialize().await
        }

        async fn start(&mut self) -> DfResult<()> {
            self.base.start().await
        }

        async fn stop(&mut self) -> DfResult<()> {
            self.base.stop().await
        }
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ServiceRegistry::new();
        assert!(registry.get_service_names().await.is_empty());
    }

    #[tokio::test]
    async fn test_service_registration() {
        let registry = ServiceRegistry::new();
        let service = TestServiceA::new();
        
        registry.register_service(service).await.unwrap();
        
        assert!(registry.has_service("service-a").await);
        assert_eq!(registry.get_service_names().await, vec!["service-a"]);
    }

    #[tokio::test]
    async fn test_duplicate_service_registration() {
        let registry = ServiceRegistry::new();
        let service1 = TestServiceA::new();
        let service2 = TestServiceA::new();
        
        registry.register_service(service1).await.unwrap();
        let result = registry.register_service(service2).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DfError::Registry { .. }));
    }

    #[tokio::test]
    async fn test_service_info_retrieval() {
        let registry = ServiceRegistry::new();
        let service = TestServiceA::new();
        
        registry.register_service(service).await.unwrap();
        
        let info = registry.get_service_info("service-a").await.unwrap();
        assert_eq!(info.name, "service-a");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, "Test service A");
    }

    #[tokio::test]
    async fn test_service_state_retrieval() {
        let registry = ServiceRegistry::new();
        let service = TestServiceA::new();
        
        registry.register_service(service).await.unwrap();
        
        let state = registry.get_service_state("service-a").await.unwrap();
        assert_eq!(state, ServiceState::Created);
    }

    #[tokio::test]
    async fn test_dependency_order_calculation() {
        let registry = ServiceRegistry::new();
        
        registry.register_service(TestServiceC::new()).await.unwrap();
        registry.register_service(TestServiceB::new()).await.unwrap(); 
        registry.register_service(TestServiceA::new()).await.unwrap();
        
        let startup_order = registry.get_startup_order().await;
        
        // service-a should come first (no dependencies)
        // service-b should come second (depends on service-a)
        // service-c should come last (depends on both)
        assert_eq!(startup_order[0], "service-a");
        assert_eq!(startup_order[1], "service-b");
        assert_eq!(startup_order[2], "service-c");
    }

    #[tokio::test]
    async fn test_missing_dependency_error() {
        let registry = ServiceRegistry::new();
        
        // Register service-b which depends on service-a, but don't register service-a
        let result = registry.register_service(TestServiceB::new()).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let registry = ServiceRegistry::new();
        
        // Create services with circular dependency
        let info_a = ServiceInfo::new("service-a", "1.0.0", "Test service A")
            .with_dependency("service-b");
        let service_a = BaseService::new(info_a);
        
        let info_b = ServiceInfo::new("service-b", "1.0.0", "Test service B")
            .with_dependency("service-a");
        let service_b = BaseService::new(info_b);
        
        registry.register_service(service_a).await.unwrap();
        let result = registry.register_service(service_b).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        let registry = ServiceRegistry::new();
        registry.register_service(TestServiceA::new()).await.unwrap();
        
        // Initialize
        registry.initialize_service("service-a").await.unwrap();
        let state = registry.get_service_state("service-a").await.unwrap();
        assert_eq!(state, ServiceState::Initialized);
        
        // Start
        registry.start_service("service-a").await.unwrap();
        let state = registry.get_service_state("service-a").await.unwrap();
        assert_eq!(state, ServiceState::Running);
        
        // Stop
        registry.stop_service("service-a").await.unwrap();
        let state = registry.get_service_state("service-a").await.unwrap();
        assert_eq!(state, ServiceState::Stopped);
    }

    #[tokio::test]
    async fn test_initialize_all_services() {
        let registry = ServiceRegistry::new();
        
        registry.register_service(TestServiceA::new()).await.unwrap();
        registry.register_service(TestServiceB::new()).await.unwrap();
        registry.register_service(TestServiceC::new()).await.unwrap();
        
        registry.initialize_all().await.unwrap();
        
        // All services should be initialized
        assert_eq!(registry.get_service_state("service-a").await.unwrap(), ServiceState::Initialized);
        assert_eq!(registry.get_service_state("service-b").await.unwrap(), ServiceState::Initialized);
        assert_eq!(registry.get_service_state("service-c").await.unwrap(), ServiceState::Initialized);
    }

    #[tokio::test]
    async fn test_start_all_services() {
        let registry = ServiceRegistry::new();
        
        registry.register_service(TestServiceA::new()).await.unwrap();
        registry.register_service(TestServiceB::new()).await.unwrap();
        registry.register_service(TestServiceC::new()).await.unwrap();
        
        registry.initialize_all().await.unwrap();
        registry.start_all().await.unwrap();
        
        // All services should be running
        assert_eq!(registry.get_service_state("service-a").await.unwrap(), ServiceState::Running);
        assert_eq!(registry.get_service_state("service-b").await.unwrap(), ServiceState::Running);
        assert_eq!(registry.get_service_state("service-c").await.unwrap(), ServiceState::Running);
    }

    #[tokio::test]
    async fn test_stop_all_services() {
        let registry = ServiceRegistry::new();
        
        registry.register_service(TestServiceA::new()).await.unwrap();
        registry.register_service(TestServiceB::new()).await.unwrap();
        registry.register_service(TestServiceC::new()).await.unwrap();
        
        registry.initialize_all().await.unwrap();
        registry.start_all().await.unwrap();
        registry.stop_all().await.unwrap();
        
        // All services should be stopped
        assert_eq!(registry.get_service_state("service-a").await.unwrap(), ServiceState::Stopped);
        assert_eq!(registry.get_service_state("service-b").await.unwrap(), ServiceState::Stopped);
        assert_eq!(registry.get_service_state("service-c").await.unwrap(), ServiceState::Stopped);
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let registry = ServiceRegistry::new();
        
        registry.register_service(TestServiceA::new()).await.unwrap();
        registry.register_service(TestServiceB::new()).await.unwrap();
        
        registry.initialize_all().await.unwrap();
        registry.start_all().await.unwrap();
        
        let health_results = registry.health_check_all().await.unwrap();
        
        assert_eq!(health_results.len(), 2);
        assert_eq!(health_results.get("service-a"), Some(&true));
        assert_eq!(health_results.get("service-b"), Some(&true));
    }

    #[tokio::test]
    async fn test_service_removal() {
        let registry = ServiceRegistry::new();
        registry.register_service(TestServiceA::new()).await.unwrap();
        
        assert!(registry.has_service("service-a").await);
        
        registry.remove_service("service-a").await.unwrap();
        
        assert!(!registry.has_service("service-a").await);
        assert!(registry.get_service_names().await.is_empty());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_service() {
        let registry = ServiceRegistry::new();
        
        let result = registry.remove_service("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_clear() {
        let registry = ServiceRegistry::new();
        
        registry.register_service(TestServiceA::new()).await.unwrap();
        registry.register_service(TestServiceB::new()).await.unwrap();
        
        assert_eq!(registry.get_service_names().await.len(), 2);
        
        registry.clear().await;
        
        assert!(registry.get_service_names().await.is_empty());
        assert!(registry.get_startup_order().await.is_empty());
    }

    #[tokio::test]
    async fn test_factory_registration() {
        let registry = ServiceRegistry::new();
        
        registry.register_closure_factory("service-a".to_string(), || TestServiceA::new()).await.unwrap();
        
        assert!(registry.has_service("service-a").await);
    }

    #[tokio::test]
    async fn test_service_not_found() {
        let registry = ServiceRegistry::new();
        
        let result = registry.get_service_info("nonexistent").await;
        assert!(result.is_err());
        
        let result = registry.get_service_state("nonexistent").await;
        assert!(result.is_err());
        
        let result = registry.initialize_service("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dependency_graph_retrieval() {
        let registry = ServiceRegistry::new();
        
        registry.register_service(TestServiceA::new()).await.unwrap();
        registry.register_service(TestServiceB::new()).await.unwrap();
        registry.register_service(TestServiceC::new()).await.unwrap();
        
        let graph = registry.get_dependency_graph().await;
        
        assert_eq!(graph.get("service-a"), Some(&vec![]));
        assert_eq!(graph.get("service-b"), Some(&vec!["service-a".to_string()]));
        assert_eq!(graph.get("service-c"), Some(&vec!["service-a".to_string(), "service-b".to_string()]));
    }

    // Test error cases
    #[test]
    fn test_registry_error_types() {
        let error = RegistryError::ServiceNotFound { name: "test".to_string() };
        assert!(error.to_string().contains("test"));
        
        let error = RegistryError::ServiceAlreadyRegistered { name: "test".to_string() };
        assert!(error.to_string().contains("test"));
        
        let error = RegistryError::CircularDependency { cycle: vec!["a".to_string(), "b".to_string()] };
        assert!(error.to_string().contains("Circular"));
        
        let error = RegistryError::DependencyNotSatisfied { 
            service: "a".to_string(), 
            dependency: "b".to_string() 
        };
        assert!(error.to_string().contains("not satisfied"));
    }

    #[test]
    fn test_registry_error_conversion() {
        let registry_error = RegistryError::ServiceNotFound { name: "test".to_string() };
        let df_error: DfError = registry_error.into();
        
        match df_error {
            DfError::Registry { message } => {
                assert!(message.contains("test"));
            }
            _ => panic!("Expected Registry error"),
        }
    }
}