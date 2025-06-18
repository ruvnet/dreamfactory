use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, instrument};

/// Service discovery and registration system
pub struct ServiceDiscovery {
    services: HashMap<String, ServiceInfo>,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub endpoints: Vec<EndpointInfo>,
    pub is_active: bool,
    pub health_check_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EndpointInfo {
    pub path: String,
    pub method: String,
    pub description: String,
    pub parameters: Vec<ParameterInfo>,
    pub response_format: String,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

impl ServiceDiscovery {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register a service with its metadata
    #[instrument(skip(self))]
    pub fn register_service(&mut self, service: ServiceInfo) -> Result<()> {
        info!("Registering service: {}", service.name);
        self.services.insert(service.name.clone(), service);
        Ok(())
    }

    /// Get service information by name
    pub fn get_service(&self, name: &str) -> Option<&ServiceInfo> {
        self.services.get(name)
    }

    /// List all registered services  
    pub fn list_services(&self) -> Vec<&ServiceInfo> {
        self.services.values().collect()
    }

    /// Get service discovery information as JSON
    pub fn to_json(&self) -> Value {
        let services: Vec<Value> = self.services
            .values()
            .map(|service| {
                serde_json::json!({
                    "name": service.name,
                    "version": service.version,
                    "description": service.description,
                    "is_active": service.is_active,
                    "health_check_url": service.health_check_url,
                    "endpoints": service.endpoints.iter().map(|endpoint| {
                        serde_json::json!({
                            "path": endpoint.path,
                            "method": endpoint.method,
                            "description": endpoint.description,
                            "parameters": endpoint.parameters.iter().map(|param| {
                                serde_json::json!({
                                    "name": param.name,
                                    "type": param.param_type,
                                    "required": param.required,
                                    "description": param.description
                                })
                            }).collect::<Vec<_>>(),
                            "response_format": endpoint.response_format
                        })
                    }).collect::<Vec<_>>()
                })
            })
            .collect();

        serde_json::json!({
            "services": services,
            "total_count": services.len()
        })
    }

    /// Initialize with default DreamFactory services
    pub fn with_defaults() -> Self {
        let mut discovery = Self::new();
        
        // Register system service
        let system_service = ServiceInfo {
            name: "system".to_string(),
            version: "2.0".to_string(),
            description: "System administration and configuration".to_string(),
            is_active: true,
            health_check_url: Some("/api/v2/system".to_string()),
            endpoints: vec![
                EndpointInfo {
                    path: "/api/v2/system/service".to_string(),
                    method: "GET".to_string(),
                    description: "List all available services".to_string(),
                    parameters: vec![],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/system/admin".to_string(),
                    method: "GET".to_string(),
                    description: "Get system administration information".to_string(),
                    parameters: vec![],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/system/environment".to_string(),
                    method: "GET".to_string(),
                    description: "Get system environment information".to_string(),
                    parameters: vec![],
                    response_format: "application/json".to_string(),
                },
            ],
        };

        // Register cache service
        let cache_service = ServiceInfo {
            name: "cache".to_string(),
            version: "2.0".to_string(),
            description: "Distributed caching service".to_string(),
            is_active: true,
            health_check_url: Some("/api/v2/cache".to_string()),
            endpoints: vec![
                EndpointInfo {
                    path: "/api/v2/cache".to_string(),
                    method: "GET".to_string(),
                    description: "List all cache keys".to_string(),
                    parameters: vec![],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/cache/{key}".to_string(),
                    method: "GET".to_string(),
                    description: "Get cache value by key".to_string(),
                    parameters: vec![
                        ParameterInfo {
                            name: "key".to_string(),
                            param_type: "string".to_string(),
                            required: true,
                            description: "Cache key identifier".to_string(),
                        }
                    ],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/cache/{key}".to_string(),
                    method: "POST".to_string(),
                    description: "Set cache value".to_string(),
                    parameters: vec![
                        ParameterInfo {
                            name: "key".to_string(),
                            param_type: "string".to_string(),
                            required: true,
                            description: "Cache key identifier".to_string(),
                        }
                    ],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/cache/{key}".to_string(),
                    method: "PUT".to_string(),
                    description: "Update cache value".to_string(),
                    parameters: vec![
                        ParameterInfo {
                            name: "key".to_string(),
                            param_type: "string".to_string(),
                            required: true,
                            description: "Cache key identifier".to_string(),
                        }
                    ],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/cache/{key}".to_string(),
                    method: "DELETE".to_string(),
                    description: "Delete cache value".to_string(),
                    parameters: vec![
                        ParameterInfo {
                            name: "key".to_string(),
                            param_type: "string".to_string(),
                            required: true,
                            description: "Cache key identifier".to_string(),
                        }
                    ],
                    response_format: "application/json".to_string(),
                },
            ],
        };

        // Register email service
        let email_service = ServiceInfo {
            name: "email".to_string(),
            version: "2.0".to_string(),
            description: "Email sending and template management".to_string(),
            is_active: true,
            health_check_url: Some("/api/v2/email".to_string()),
            endpoints: vec![
                EndpointInfo {
                    path: "/api/v2/email/_send".to_string(),
                    method: "POST".to_string(),
                    description: "Send email".to_string(),
                    parameters: vec![
                        ParameterInfo {
                            name: "to".to_string(),
                            param_type: "array".to_string(),
                            required: true,
                            description: "Recipient email addresses".to_string(),
                        },
                        ParameterInfo {
                            name: "subject".to_string(),
                            param_type: "string".to_string(),
                            required: true,
                            description: "Email subject".to_string(),
                        },
                        ParameterInfo {
                            name: "body_text".to_string(),
                            param_type: "string".to_string(),
                            required: false,
                            description: "Plain text email body".to_string(),
                        },
                        ParameterInfo {
                            name: "body_html".to_string(),
                            param_type: "string".to_string(),
                            required: false,
                            description: "HTML email body".to_string(),
                        },
                    ],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/email/template".to_string(),
                    method: "GET".to_string(),
                    description: "List email templates".to_string(),
                    parameters: vec![],
                    response_format: "application/json".to_string(),
                },
                EndpointInfo {
                    path: "/api/v2/email/template/{id}".to_string(),
                    method: "GET".to_string(),
                    description: "Get email template by ID".to_string(),
                    parameters: vec![
                        ParameterInfo {
                            name: "id".to_string(),
                            param_type: "string".to_string(),
                            required: true,
                            description: "Template ID or name".to_string(),
                        }
                    ],
                    response_format: "application/json".to_string(),
                },
            ],
        };

        discovery.register_service(system_service).unwrap();
        discovery.register_service(cache_service).unwrap();
        discovery.register_service(email_service).unwrap();

        discovery
    }
}

impl Default for ServiceDiscovery {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_discovery_creation() {
        let discovery = ServiceDiscovery::new();
        assert_eq!(discovery.services.len(), 0);
    }

    #[test]
    fn test_service_registration() {
        let mut discovery = ServiceDiscovery::new();
        
        let service = ServiceInfo {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test service".to_string(),
            is_active: true,
            health_check_url: None,
            endpoints: vec![],
        };

        discovery.register_service(service).unwrap();
        assert_eq!(discovery.services.len(), 1);
        assert!(discovery.get_service("test").is_some());
    }

    #[test]
    fn test_service_listing() {
        let mut discovery = ServiceDiscovery::new();
        
        let service1 = ServiceInfo {
            name: "test1".to_string(),
            version: "1.0".to_string(),
            description: "Test service 1".to_string(),
            is_active: true,
            health_check_url: None,
            endpoints: vec![],
        };

        let service2 = ServiceInfo {
            name: "test2".to_string(),
            version: "1.0".to_string(),
            description: "Test service 2".to_string(),
            is_active: true,
            health_check_url: None,
            endpoints: vec![],
        };

        discovery.register_service(service1).unwrap();
        discovery.register_service(service2).unwrap();

        let services = discovery.list_services();
        assert_eq!(services.len(), 2);
    }

    #[test]
    fn test_default_services() {
        let discovery = ServiceDiscovery::with_defaults();
        
        assert!(discovery.get_service("system").is_some());
        assert!(discovery.get_service("cache").is_some());
        assert!(discovery.get_service("email").is_some());
        
        let services = discovery.list_services();
        assert_eq!(services.len(), 3);
    }

    #[test]
    fn test_service_to_json() {
        let discovery = ServiceDiscovery::with_defaults();
        let json = discovery.to_json();
        
        assert!(json.get("services").is_some());
        assert!(json.get("total_count").is_some());
        
        let services = json.get("services").unwrap().as_array().unwrap();
        assert_eq!(services.len(), 3);
        
        // Check that system service is properly serialized
        let system_service = services.iter()
            .find(|s| s.get("name") == Some(&serde_json::json!("system")))
            .unwrap();
        
        assert_eq!(system_service.get("version"), Some(&serde_json::json!("2.0")));
        assert!(system_service.get("endpoints").is_some());
    }

    #[test]
    fn test_service_get_nonexistent() {
        let discovery = ServiceDiscovery::new();
        assert!(discovery.get_service("nonexistent").is_none());
    }
}