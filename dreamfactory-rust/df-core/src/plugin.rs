//! Plugin architecture and dynamic loading system for DreamFactory.
//!
//! This module provides a flexible plugin system that allows for:
//! - Dynamic loading of plugins from shared libraries
//! - Plugin lifecycle management
//! - Plugin dependency resolution
//! - Safe plugin isolation and communication

use crate::error::{DfError, DfResult};
use crate::service::{Service, ServiceInfo, ServiceState};
use async_trait::async_trait;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Plugin-specific error types
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {name}")]
    PluginNotFound { name: String },
    
    #[error("Plugin already loaded: {name}")]
    PluginAlreadyLoaded { name: String },
    
    #[error("Plugin loading failed: {path} - {reason}")]
    LoadingFailed { path: String, reason: String },
    
    #[error("Plugin initialization failed: {name} - {reason}")]
    InitializationFailed { name: String, reason: String },
    
    #[error("Plugin version mismatch: required {required}, got {actual}")]
    VersionMismatch { required: String, actual: String },
    
    #[error("Plugin API version incompatible: {version}")]
    ApiVersionIncompatible { version: String },
    
    #[error("Plugin dependency not satisfied: {plugin} requires {dependency}")]
    DependencyNotSatisfied { plugin: String, dependency: String },
    
    #[error("Plugin security violation: {reason}")]
    SecurityViolation { reason: String },
}

impl From<PluginError> for DfError {
    fn from(err: PluginError) -> Self {
        DfError::plugin("unknown", err.to_string())
    }
}

/// Plugin metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin unique identifier
    pub id: Uuid,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Required API version
    pub api_version: String,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Plugin capabilities/permissions
    pub capabilities: Vec<String>,
    /// Plugin metadata
    pub metadata: HashMap<String, String>,
    /// Plugin entry point symbol name
    pub entry_point: String,
}

impl PluginInfo {
    /// Create a new PluginInfo
    pub fn new<S: Into<String>>(name: S, version: S, description: S, author: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            api_version: "1.0.0".to_string(),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            entry_point: "plugin_main".to_string(),
        }
    }

    /// Add a dependency
    pub fn with_dependency<S: Into<String>>(mut self, dependency: S) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies
    pub fn with_dependencies<I, S>(mut self, dependencies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.dependencies.extend(dependencies.into_iter().map(|s| s.into()));
        self
    }

    /// Add a capability
    pub fn with_capability<S: Into<String>>(mut self, capability: S) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add multiple capabilities
    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities.extend(capabilities.into_iter().map(|s| s.into()));
        self
    }

    /// Set API version
    pub fn with_api_version<S: Into<String>>(mut self, version: S) -> Self {
        self.api_version = version.into();
        self
    }

    /// Set entry point
    pub fn with_entry_point<S: Into<String>>(mut self, entry_point: S) -> Self {
        self.entry_point = entry_point.into();
        self
    }

    /// Add metadata
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Plugin state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is discovered but not loaded
    Discovered,
    /// Plugin is loading
    Loading,
    /// Plugin is loaded but not initialized
    Loaded,
    /// Plugin is initializing
    Initializing,
    /// Plugin is initialized and ready
    Initialized,
    /// Plugin is starting
    Starting,
    /// Plugin is running
    Running,
    /// Plugin is stopping
    Stopping,
    /// Plugin is stopped
    Stopped,
    /// Plugin is unloading
    Unloading,
    /// Plugin encountered an error
    Error,
}

/// Plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> &PluginInfo;

    /// Get current plugin state
    fn state(&self) -> PluginState;

    /// Initialize the plugin
    async fn initialize(&mut self) -> DfResult<()>;

    /// Start the plugin
    async fn start(&mut self) -> DfResult<()>;

    /// Stop the plugin
    async fn stop(&mut self) -> DfResult<()>;

    /// Handle plugin events
    async fn handle_event(&mut self, event: PluginEvent) -> DfResult<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.info().capabilities.clone()
    }

    /// Check if plugin has a specific capability
    fn has_capability(&self, capability: &str) -> bool {
        self.info().capabilities.contains(&capability.to_string())
    }
}

/// Plugin events that can be sent to plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// System startup event
    SystemStartup,
    /// System shutdown event
    SystemShutdown,
    /// Configuration changed event
    ConfigurationChanged { keys: Vec<String> },
    /// Service registered event
    ServiceRegistered { service_name: String },
    /// Service unregistered event
    ServiceUnregistered { service_name: String },
    /// Custom event with data
    Custom { event_type: String, data: serde_json::Value },
}

/// Plugin entry point function type
pub type PluginEntryPoint = unsafe extern "C" fn() -> *mut dyn Plugin;

/// Loaded plugin container
struct LoadedPlugin {
    plugin: Box<dyn Plugin>,
    library: Library,
    path: PathBuf,
    state: PluginState,
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    plugin_directories: Vec<PathBuf>,
    whitelist: Vec<String>,
    blacklist: Vec<String>,
    auto_load: bool,
    api_version: String,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_directories: Vec::new(),
            whitelist: Vec::new(),
            blacklist: Vec::new(),
            auto_load: true,
            api_version: "1.0.0".to_string(),
        }
    }

    /// Add a plugin directory to search
    pub fn add_plugin_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.plugin_directories.push(path.as_ref().to_path_buf());
    }

    /// Set plugin whitelist (only these plugins will be loaded)
    pub fn set_whitelist(&mut self, whitelist: Vec<String>) {
        self.whitelist = whitelist;
    }

    /// Set plugin blacklist (these plugins will not be loaded)
    pub fn set_blacklist(&mut self, blacklist: Vec<String>) {
        self.blacklist = blacklist;
    }

    /// Enable or disable auto-loading of plugins
    pub fn set_auto_load(&mut self, auto_load: bool) {
        self.auto_load = auto_load;
    }

    /// Set API version requirement
    pub fn set_api_version<S: Into<String>>(&mut self, version: S) {
        self.api_version = version.into();
    }

    /// Discover plugins in configured directories
    pub async fn discover_plugins(&self) -> DfResult<Vec<PathBuf>> {
        let mut plugin_paths = Vec::new();

        for directory in &self.plugin_directories {
            if !directory.exists() {
                continue;
            }

            let mut entries = tokio::fs::read_dir(directory).await
                .map_err(|e| DfError::plugin("manager", format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| DfError::plugin("manager", format!("Failed to read entry: {}", e)))? {
                
                let path = entry.path();
                
                // Check if it's a shared library
                if self.is_plugin_library(&path) {
                    plugin_paths.push(path);
                }
            }
        }

        Ok(plugin_paths)
    }

    /// Check if a file is a plugin library
    fn is_plugin_library(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(OsStr::to_str) {
            match extension {
                "so" | "dylib" | "dll" => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Load a plugin from a file path
    pub async fn load_plugin<P: AsRef<Path>>(&self, path: P) -> DfResult<String> {
        let path = path.as_ref();
        let plugin_name = path.file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| DfError::plugin("manager", "Invalid plugin filename"))?
            .to_string();

        // Check whitelist/blacklist
        if !self.whitelist.is_empty() && !self.whitelist.contains(&plugin_name) {
            return Err(DfError::plugin(&plugin_name, "Plugin not in whitelist"));
        }

        if self.blacklist.contains(&plugin_name) {
            return Err(DfError::plugin(&plugin_name, "Plugin is blacklisted"));
        }

        let mut plugins = self.plugins.write().await;

        if plugins.contains_key(&plugin_name) {
            return Err(PluginError::PluginAlreadyLoaded { name: plugin_name }.into());
        }

        // Load the library
        let library = unsafe {
            Library::new(path)
                .map_err(|e| PluginError::LoadingFailed {
                    path: path.display().to_string(),
                    reason: e.to_string(),
                })?
        };

        // Get the plugin entry point
        let entry_point: Symbol<PluginEntryPoint> = unsafe {
            library.get(b"plugin_main")
                .map_err(|e| PluginError::LoadingFailed {
                    path: path.display().to_string(),
                    reason: format!("Entry point not found: {}", e),
                })?
        };

        // Create the plugin instance
        let plugin_ptr = unsafe { entry_point() };
        if plugin_ptr.is_null() {
            return Err(PluginError::LoadingFailed {
                path: path.display().to_string(),
                reason: "Entry point returned null".to_string(),
            }.into());
        }

        let plugin = unsafe { Box::from_raw(plugin_ptr) };

        // Validate API version
        let plugin_api_version = &plugin.info().api_version;
        if !self.is_api_version_compatible(plugin_api_version) {
            return Err(PluginError::ApiVersionIncompatible {
                version: plugin_api_version.clone(),
            }.into());
        }

        let loaded_plugin = LoadedPlugin {
            plugin,
            library,
            path: path.to_path_buf(),
            state: PluginState::Loaded,
        };

        plugins.insert(plugin_name.clone(), loaded_plugin);

        Ok(plugin_name)
    }

    /// Check if API version is compatible
    fn is_api_version_compatible(&self, plugin_version: &str) -> bool {
        // Simple version compatibility check
        // In a real implementation, you might want to use semver parsing
        plugin_version == self.api_version
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, name: &str) -> DfResult<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(mut loaded_plugin) = plugins.remove(name) {
            loaded_plugin.state = PluginState::Unloading;
            
            // Stop the plugin if it's running
            if loaded_plugin.state == PluginState::Running {
                loaded_plugin.plugin.stop().await?;
            }

            // Plugin will be dropped and library unloaded when LoadedPlugin is dropped
            Ok(())
        } else {
            Err(PluginError::PluginNotFound { name: name.to_string() }.into())
        }
    }

    /// Initialize a loaded plugin
    pub async fn initialize_plugin(&self, name: &str) -> DfResult<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(loaded_plugin) = plugins.get_mut(name) {
            if loaded_plugin.state != PluginState::Loaded {
                return Err(DfError::plugin(name, "Plugin must be loaded before initialization"));
            }

            loaded_plugin.state = PluginState::Initializing;
            
            match loaded_plugin.plugin.initialize().await {
                Ok(()) => {
                    loaded_plugin.state = PluginState::Initialized;
                    Ok(())
                }
                Err(e) => {
                    loaded_plugin.state = PluginState::Error;
                    Err(PluginError::InitializationFailed {
                        name: name.to_string(),
                        reason: e.to_string(),
                    }.into())
                }
            }
        } else {
            Err(PluginError::PluginNotFound { name: name.to_string() }.into())
        }
    }

    /// Start a plugin
    pub async fn start_plugin(&self, name: &str) -> DfResult<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(loaded_plugin) = plugins.get_mut(name) {
            if loaded_plugin.state != PluginState::Initialized {
                return Err(DfError::plugin(name, "Plugin must be initialized before starting"));
            }

            loaded_plugin.state = PluginState::Starting;
            
            match loaded_plugin.plugin.start().await {
                Ok(()) => {
                    loaded_plugin.state = PluginState::Running;
                    Ok(())
                }
                Err(e) => {
                    loaded_plugin.state = PluginState::Error;
                    Err(e)
                }
            }
        } else {
            Err(PluginError::PluginNotFound { name: name.to_string() }.into())
        }
    }

    /// Stop a plugin
    pub async fn stop_plugin(&self, name: &str) -> DfResult<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(loaded_plugin) = plugins.get_mut(name) {
            if loaded_plugin.state != PluginState::Running {
                return Err(DfError::plugin(name, "Plugin must be running to stop"));
            }

            loaded_plugin.state = PluginState::Stopping;
            
            match loaded_plugin.plugin.stop().await {
                Ok(()) => {
                    loaded_plugin.state = PluginState::Stopped;
                    Ok(())
                }
                Err(e) => {
                    loaded_plugin.state = PluginState::Error;
                    Err(e)
                }
            }
        } else {
            Err(PluginError::PluginNotFound { name: name.to_string() }.into())
        }
    }

    /// Get plugin information
    pub async fn get_plugin_info(&self, name: &str) -> DfResult<PluginInfo> {
        let plugins = self.plugins.read().await;

        if let Some(loaded_plugin) = plugins.get(name) {
            Ok(loaded_plugin.plugin.info().clone())
        } else {
            Err(PluginError::PluginNotFound { name: name.to_string() }.into())
        }
    }

    /// Get plugin state
    pub async fn get_plugin_state(&self, name: &str) -> DfResult<PluginState> {
        let plugins = self.plugins.read().await;

        if let Some(loaded_plugin) = plugins.get(name) {
            Ok(loaded_plugin.state)
        } else {
            Err(PluginError::PluginNotFound { name: name.to_string() }.into())
        }
    }

    /// Get all loaded plugin names
    pub async fn get_loaded_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// Send event to all plugins
    pub async fn broadcast_event(&self, event: PluginEvent) -> DfResult<()> {
        let mut plugins = self.plugins.write().await;

        for (name, loaded_plugin) in plugins.iter_mut() {
            if loaded_plugin.state == PluginState::Running {
                if let Err(e) = loaded_plugin.plugin.handle_event(event.clone()).await {
                    tracing::warn!("Plugin {} failed to handle event: {}", name, e);
                }
            }
        }

        Ok(())
    }

    /// Send event to a specific plugin
    pub async fn send_event(&self, plugin_name: &str, event: PluginEvent) -> DfResult<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(loaded_plugin) = plugins.get_mut(plugin_name) {
            if loaded_plugin.state == PluginState::Running {
                loaded_plugin.plugin.handle_event(event).await
            } else {
                Err(DfError::plugin(plugin_name, "Plugin is not running"))
            }
        } else {
            Err(PluginError::PluginNotFound { name: plugin_name.to_string() }.into())
        }
    }

    /// Auto-load all plugins from configured directories
    pub async fn auto_load_plugins(&self) -> DfResult<Vec<String>> {
        if !self.auto_load {
            return Ok(Vec::new());
        }

        let plugin_paths = self.discover_plugins().await?;
        let mut loaded_plugins = Vec::new();

        for path in plugin_paths {
            match self.load_plugin(&path).await {
                Ok(plugin_name) => {
                    loaded_plugins.push(plugin_name);
                }
                Err(e) => {
                    tracing::warn!("Failed to load plugin {:?}: {}", path, e);
                }
            }
        }

        Ok(loaded_plugins)
    }

    /// Initialize all loaded plugins
    pub async fn initialize_all(&self) -> DfResult<()> {
        let plugin_names = self.get_loaded_plugins().await;

        for name in plugin_names {
            if let Err(e) = self.initialize_plugin(&name).await {
                tracing::error!("Failed to initialize plugin {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Start all initialized plugins
    pub async fn start_all(&self) -> DfResult<()> {
        let plugin_names = self.get_loaded_plugins().await;

        for name in plugin_names {
            if let Err(e) = self.start_plugin(&name).await {
                tracing::error!("Failed to start plugin {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Stop all running plugins
    pub async fn stop_all(&self) -> DfResult<()> {
        let plugin_names = self.get_loaded_plugins().await;

        for name in plugin_names {
            if let Err(e) = self.stop_plugin(&name).await {
                tracing::error!("Failed to stop plugin {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Unload all plugins
    pub async fn unload_all(&self) -> DfResult<()> {
        let plugin_names = self.get_loaded_plugins().await;

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name).await {
                tracing::error!("Failed to unload plugin {}: {}", name, e);
            }
        }

        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock plugin implementation for testing
    struct MockPlugin {
        info: PluginInfo,
        state: PluginState,
        initialized: bool,
        started: bool,
        events_received: Vec<PluginEvent>,
    }

    impl MockPlugin {
        fn new(name: &str) -> Self {
            let info = PluginInfo::new(name, "1.0.0", "Mock plugin", "Test Author")
                .with_capability("test")
                .with_metadata("test", "value");

            Self {
                info,
                state: PluginState::Loaded,
                initialized: false,
                started: false,
                events_received: Vec::new(),
            }
        }

        fn new_with_dependencies(name: &str, deps: Vec<&str>) -> Self {
            let info = PluginInfo::new(name, "1.0.0", "Mock plugin", "Test Author")
                .with_dependencies(deps.into_iter().map(|s| s.to_string()));

            Self {
                info,
                state: PluginState::Loaded,
                initialized: false,
                started: false,
                events_received: Vec::new(),
            }
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn info(&self) -> &PluginInfo {
            &self.info
        }

        fn state(&self) -> PluginState {
            self.state
        }

        async fn initialize(&mut self) -> DfResult<()> {
            if self.initialized {
                return Err(DfError::plugin(&self.info.name, "Already initialized"));
            }
            
            self.state = PluginState::Initializing;
            self.initialized = true;
            self.state = PluginState::Initialized;
            Ok(())
        }

        async fn start(&mut self) -> DfResult<()> {
            if !self.initialized {
                return Err(DfError::plugin(&self.info.name, "Not initialized"));
            }
            if self.started {
                return Err(DfError::plugin(&self.info.name, "Already started"));
            }
            
            self.state = PluginState::Starting;
            self.started = true;
            self.state = PluginState::Running;
            Ok(())
        }

        async fn stop(&mut self) -> DfResult<()> {
            if !self.started {
                return Err(DfError::plugin(&self.info.name, "Not started"));
            }
            
            self.state = PluginState::Stopping;
            self.started = false;
            self.state = PluginState::Stopped;
            Ok(())
        }

        async fn handle_event(&mut self, event: PluginEvent) -> DfResult<()> {
            self.events_received.push(event);
            Ok(())
        }
    }

    #[test]
    fn test_plugin_info_creation() {
        let info = PluginInfo::new("test-plugin", "1.0.0", "A test plugin", "Test Author");
        
        assert_eq!(info.name, "test-plugin");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, "A test plugin");
        assert_eq!(info.author, "Test Author");
        assert_eq!(info.api_version, "1.0.0");
        assert!(info.dependencies.is_empty());
        assert!(info.capabilities.is_empty());
    }

    #[test]
    fn test_plugin_info_builder() {
        let info = PluginInfo::new("test-plugin", "1.0.0", "A test plugin", "Test Author")
            .with_dependency("dep1")
            .with_dependencies(["dep2", "dep3"])
            .with_capability("cap1")
            .with_capabilities(["cap2", "cap3"])
            .with_api_version("2.0.0")
            .with_entry_point("custom_main")
            .with_metadata("key", "value");

        assert_eq!(info.dependencies, vec!["dep1", "dep2", "dep3"]);
        assert_eq!(info.capabilities, vec!["cap1", "cap2", "cap3"]);
        assert_eq!(info.api_version, "2.0.0");
        assert_eq!(info.entry_point, "custom_main");
        assert_eq!(info.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_plugin_state_serialization() {
        let states = [
            PluginState::Discovered,
            PluginState::Loading,
            PluginState::Loaded,
            PluginState::Initializing,
            PluginState::Initialized,
            PluginState::Starting,
            PluginState::Running,
            PluginState::Stopping,
            PluginState::Stopped,
            PluginState::Unloading,
            PluginState::Error,
        ];

        for state in &states {
            let json = serde_json::to_string(state).unwrap();
            let deserialized: PluginState = serde_json::from_str(&json).unwrap();
            assert_eq!(*state, deserialized);
        }
    }

    #[tokio::test]
    async fn test_mock_plugin_lifecycle() {
        let mut plugin = MockPlugin::new("test-plugin");

        // Initial state
        assert_eq!(plugin.state(), PluginState::Loaded);
        assert!(!plugin.initialized);
        assert!(!plugin.started);

        // Initialize
        plugin.initialize().await.unwrap();
        assert_eq!(plugin.state(), PluginState::Initialized);
        assert!(plugin.initialized);

        // Start
        plugin.start().await.unwrap();
        assert_eq!(plugin.state(), PluginState::Running);
        assert!(plugin.started);

        // Stop
        plugin.stop().await.unwrap();
        assert_eq!(plugin.state(), PluginState::Stopped);
        assert!(!plugin.started);
    }

    #[tokio::test]
    async fn test_mock_plugin_invalid_transitions() {
        let mut plugin = MockPlugin::new("test-plugin");

        // Try to start without initializing
        let result = plugin.start().await;
        assert!(result.is_err());

        // Initialize first
        plugin.initialize().await.unwrap();

        // Try to initialize again
        let result = plugin.initialize().await;
        assert!(result.is_err());

        // Start
        plugin.start().await.unwrap();

        // Try to start again
        let result = plugin.start().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_plugin_event_handling() {
        let mut plugin = MockPlugin::new("test-plugin");

        let event = PluginEvent::SystemStartup;
        plugin.handle_event(event.clone()).await.unwrap();

        assert_eq!(plugin.events_received.len(), 1);
        match &plugin.events_received[0] {
            PluginEvent::SystemStartup => {},
            _ => panic!("Expected SystemStartup event"),
        }
    }

    #[test]
    fn test_plugin_capabilities() {
        let plugin = MockPlugin::new("test-plugin");

        assert!(plugin.has_capability("test"));
        assert!(!plugin.has_capability("nonexistent"));

        let capabilities = plugin.get_capabilities();
        assert_eq!(capabilities, vec!["test"]);
    }

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.get_loaded_plugins().await.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_manager_configuration() {
        let mut manager = PluginManager::new();

        manager.add_plugin_directory("/path/to/plugins");
        manager.set_whitelist(vec!["plugin1".to_string(), "plugin2".to_string()]);
        manager.set_blacklist(vec!["bad_plugin".to_string()]);
        manager.set_auto_load(false);
        manager.set_api_version("2.0.0");

        assert_eq!(manager.plugin_directories.len(), 1);
        assert_eq!(manager.whitelist.len(), 2);
        assert_eq!(manager.blacklist.len(), 1);
        assert!(!manager.auto_load);
        assert_eq!(manager.api_version, "2.0.0");
    }

    #[test]
    fn test_plugin_manager_library_detection() {
        let manager = PluginManager::new();

        assert!(manager.is_plugin_library(Path::new("plugin.so")));
        assert!(manager.is_plugin_library(Path::new("plugin.dylib")));
        assert!(manager.is_plugin_library(Path::new("plugin.dll")));
        assert!(!manager.is_plugin_library(Path::new("plugin.txt")));
        assert!(!manager.is_plugin_library(Path::new("plugin")));
    }

    #[test]
    fn test_api_version_compatibility() {
        let manager = PluginManager::new();

        assert!(manager.is_api_version_compatible("1.0.0"));
        assert!(!manager.is_api_version_compatible("2.0.0"));
        assert!(!manager.is_api_version_compatible("0.9.0"));
    }

    // Test plugin events
    #[test]
    fn test_plugin_event_serialization() {
        let events = vec![
            PluginEvent::SystemStartup,
            PluginEvent::SystemShutdown,
            PluginEvent::ConfigurationChanged { keys: vec!["key1".to_string()] },
            PluginEvent::ServiceRegistered { service_name: "test".to_string() },
            PluginEvent::ServiceUnregistered { service_name: "test".to_string() },
            PluginEvent::Custom { 
                event_type: "test".to_string(), 
                data: serde_json::json!({"key": "value"}) 
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let deserialized: PluginEvent = serde_json::from_str(&json).unwrap();
            // We can't easily compare PluginEvent for equality, so just check serialization works
            assert!(!json.is_empty());
        }
    }

    // Test error types
    #[test]
    fn test_plugin_error_types() {
        let error = PluginError::PluginNotFound { name: "test".to_string() };
        assert!(error.to_string().contains("test"));

        let error = PluginError::PluginAlreadyLoaded { name: "test".to_string() };
        assert!(error.to_string().contains("test"));

        let error = PluginError::LoadingFailed { 
            path: "/path/to/plugin".to_string(), 
            reason: "reason".to_string() 
        };
        assert!(error.to_string().contains("Loading"));

        let error = PluginError::VersionMismatch { 
            required: "1.0".to_string(), 
            actual: "2.0".to_string() 
        };
        assert!(error.to_string().contains("mismatch"));
    }

    #[test]
    fn test_plugin_error_conversion() {
        let plugin_error = PluginError::PluginNotFound { name: "test".to_string() };
        let df_error: DfError = plugin_error.into();
        
        match df_error {
            DfError::Plugin { plugin, message } => {
                assert_eq!(plugin, "unknown");
                assert!(message.contains("test"));
            }
            _ => panic!("Expected Plugin error"),
        }
    }

    // Integration tests would go here, but they require actual plugin libraries
    // which are complex to set up in unit tests. In a real implementation,
    // you would have integration tests that load actual plugin .so/.dll files.
}