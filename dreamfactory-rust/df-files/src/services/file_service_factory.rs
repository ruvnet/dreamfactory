use crate::models::{
    config::{FileServiceConfig, StorageProvider as ConfigStorageProvider},
    error::{FileResult, FileServiceError},
};
use crate::storage::{
    LocalStorageProvider, S3StorageProvider, AzureBlobStorageProvider, GoogleCloudStorageProvider,
};
use crate::traits::FileService;
use std::sync::Arc;

/// Factory for creating file service instances
pub struct FileServiceFactory;

impl FileServiceFactory {
    /// Create a new file service instance based on the configuration
    pub async fn create(config: FileServiceConfig) -> FileResult<Arc<dyn FileService>> {
        match config.provider {
            ConfigStorageProvider::Local => {
                let mut provider = LocalStorageProvider::new(config)?;
                provider.initialize().await?;
                Ok(Arc::new(provider))
            }
            ConfigStorageProvider::S3 => {
                let mut provider = S3StorageProvider::new(config)?;
                provider.initialize().await?;
                Ok(Arc::new(provider))
            }
            ConfigStorageProvider::AzureBlob => {
                let mut provider = AzureBlobStorageProvider::new(config)?;
                provider.initialize().await?;
                Ok(Arc::new(provider))
            }
            ConfigStorageProvider::GoogleCloud => {
                let mut provider = GoogleCloudStorageProvider::new(config)?;
                provider.initialize().await?;
                Ok(Arc::new(provider))
            }
            _ => Err(FileServiceError::ConfigError {
                message: format!("Unsupported storage provider: {:?}", config.provider),
            }),
        }
    }

    /// Validate a configuration before creating a service
    pub fn validate_config(config: &FileServiceConfig) -> FileResult<()> {
        config.validate().map_err(|msg| FileServiceError::ConfigError {
            message: msg,
        })?;

        // Additional provider-specific validation could go here
        match config.provider {
            ConfigStorageProvider::Local => {
                // Local storage validation is handled in the config validation
                Ok(())
            }
            ConfigStorageProvider::S3 => {
                // S3-specific validation
                if let crate::models::config::ProviderConfig::S3(s3_config) = &config.provider_config {
                    if s3_config.bucket.is_empty() {
                        return Err(FileServiceError::ConfigError {
                            message: "S3 bucket name is required".to_string(),
                        });
                    }
                    if s3_config.region.is_empty() {
                        return Err(FileServiceError::ConfigError {
                            message: "S3 region is required".to_string(),
                        });
                    }
                }
                Ok(())
            }
            ConfigStorageProvider::AzureBlob => {
                // Azure Blob-specific validation
                if let crate::models::config::ProviderConfig::AzureBlob(azure_config) = &config.provider_config {
                    if azure_config.account_name.is_empty() {
                        return Err(FileServiceError::ConfigError {
                            message: "Azure account name is required".to_string(),
                        });
                    }
                    if azure_config.container_name.is_empty() {
                        return Err(FileServiceError::ConfigError {
                            message: "Azure container name is required".to_string(),
                        });
                    }
                    if azure_config.account_key.is_none() && azure_config.sas_token.is_none() {
                        return Err(FileServiceError::ConfigError {
                            message: "Either Azure account key or SAS token is required".to_string(),
                        });
                    }
                }
                Ok(())
            }
            ConfigStorageProvider::GoogleCloud => {
                // Google Cloud-specific validation
                if let crate::models::config::ProviderConfig::GoogleCloud(gcp_config) = &config.provider_config {
                    if gcp_config.bucket.is_empty() {
                        return Err(FileServiceError::ConfigError {
                            message: "Google Cloud bucket name is required".to_string(),
                        });
                    }
                }
                Ok(())
            }
            _ => Err(FileServiceError::ConfigError {
                message: format!("Unsupported storage provider: {:?}", config.provider),
            }),
        }
    }

    /// Get supported provider types
    pub fn supported_providers() -> Vec<ConfigStorageProvider> {
        vec![
            ConfigStorageProvider::Local,
            ConfigStorageProvider::S3,
            ConfigStorageProvider::AzureBlob,
            ConfigStorageProvider::GoogleCloud,
        ]
    }

    /// Check if a provider type is supported
    pub fn is_provider_supported(provider: &ConfigStorageProvider) -> bool {
        Self::supported_providers().contains(provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::{FileServiceConfig, LocalConfig, ProviderConfig};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_local_service() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileServiceConfig::local("test".to_string(), temp_dir.path().to_path_buf());
        
        let service = FileServiceFactory::create(config).await;
        assert!(service.is_ok());

        let service = service.unwrap();
        let info = service.service_info();
        assert_eq!(info.provider, "local");
    }

    #[test]
    fn test_validate_local_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileServiceConfig::local("test".to_string(), temp_dir.path().to_path_buf());
        
        let result = FileServiceFactory::validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_s3_config() {
        let config = FileServiceConfig::s3(
            "test".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        );
        
        let result = FileServiceFactory::validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_s3_config() {
        let config = FileServiceConfig::s3(
            "test".to_string(),
            "".to_string(), // Empty bucket name
            "us-east-1".to_string(),
        );
        
        let result = FileServiceFactory::validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_azure_config() {
        let config = FileServiceConfig::azure_blob(
            "test".to_string(),
            "testaccount".to_string(),
            "testcontainer".to_string(),
        );
        
        let result = FileServiceFactory::validate_config(&config);
        assert!(result.is_err()); // Should fail because no auth is provided
    }

    #[test]
    fn test_validate_gcs_config() {
        let config = FileServiceConfig::google_cloud(
            "test".to_string(),
            "test-bucket".to_string(),
        );
        
        let result = FileServiceFactory::validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_supported_providers() {
        let providers = FileServiceFactory::supported_providers();
        assert!(providers.contains(&ConfigStorageProvider::Local));
        assert!(providers.contains(&ConfigStorageProvider::S3));
        assert!(providers.contains(&ConfigStorageProvider::AzureBlob));
        assert!(providers.contains(&ConfigStorageProvider::GoogleCloud));
    }

    #[test]
    fn test_is_provider_supported() {
        assert!(FileServiceFactory::is_provider_supported(&ConfigStorageProvider::Local));
        assert!(FileServiceFactory::is_provider_supported(&ConfigStorageProvider::S3));
        assert!(FileServiceFactory::is_provider_supported(&ConfigStorageProvider::AzureBlob));
        assert!(FileServiceFactory::is_provider_supported(&ConfigStorageProvider::GoogleCloud));
        assert!(!FileServiceFactory::is_provider_supported(&ConfigStorageProvider::Ftp));
    }
}