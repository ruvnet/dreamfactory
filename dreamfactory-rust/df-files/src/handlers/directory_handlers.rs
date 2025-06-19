use crate::models::{
    error::FileServiceError,
    file_operation::{CopyOptions, ListOptions},
    file_info::DirectoryListing,
};
use crate::traits::FileService;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Directory creation request
#[derive(Debug, Deserialize)]
pub struct CreateDirectoryRequest {
    pub create_parents: Option<bool>,
    pub permissions: Option<u32>,
}

/// Directory listing query parameters
#[derive(Debug, Deserialize)]
pub struct ListDirectoryQuery {
    pub recursive: Option<bool>,
    pub include_hidden: Option<bool>,
    pub max_depth: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub name_pattern: Option<String>,
    pub content_type_filter: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,
}

/// Directory copy/move request
#[derive(Debug, Deserialize)]
pub struct DirectoryOperationRequest {
    pub destination: String,
    pub overwrite: Option<bool>,
    pub preserve_metadata: Option<bool>,
    pub preserve_permissions: Option<bool>,
    pub recursive: Option<bool>,
}

/// Directory deletion query parameters
#[derive(Debug, Deserialize)]
pub struct DeleteDirectoryQuery {
    pub recursive: Option<bool>,
    pub force: Option<bool>,
}

/// Directory creation response
#[derive(Debug, Serialize)]
pub struct CreateDirectoryResponse {
    pub success: bool,
    pub path: String,
    pub message: String,
}

/// Create directory handler
pub async fn create_directory(
    State(service): State<Arc<dyn FileService>>,
    Path(dir_path): Path<String>,
    Query(_params): Query<CreateDirectoryRequest>,
) -> Result<Json<CreateDirectoryResponse>, FileServiceError> {
    // Note: create_parents and permissions options would need to be added to FileService trait
    // For now, we'll use the basic create_directory method
    let result = service.create_directory(&dir_path).await?;

    Ok(Json(CreateDirectoryResponse {
        success: result.success,
        path: result.path,
        message: result.message.unwrap_or_else(|| "Directory created successfully".to_string()),
    }))
}

/// List directory contents handler
pub async fn list_directory(
    State(service): State<Arc<dyn FileService>>,
    Path(dir_path): Path<String>,
    Query(params): Query<ListDirectoryQuery>,
) -> Result<Json<DirectoryListing>, FileServiceError> {
    let mut options = ListOptions::default();

    if let Some(recursive) = params.recursive {
        options.recursive = recursive;
    }

    if let Some(include_hidden) = params.include_hidden {
        options.include_hidden = include_hidden;
    }

    if let Some(max_depth) = params.max_depth {
        options.max_depth = Some(max_depth);
    }

    if let Some(sort_by) = params.sort_by {
        use crate::models::file_operation::SortBy;
        options.sort_by = match sort_by.to_lowercase().as_str() {
            "name" => SortBy::Name,
            "size" => SortBy::Size,
            "modified" => SortBy::Modified,
            "created" => SortBy::Created,
            "type" => SortBy::Type,
            _ => SortBy::Name, // default
        };
    }

    if let Some(sort_order) = params.sort_order {
        use crate::models::file_operation::SortOrder;
        options.sort_order = match sort_order.to_lowercase().as_str() {
            "desc" | "descending" => SortOrder::Descending,
            "asc" | "ascending" => SortOrder::Ascending,
            _ => SortOrder::Ascending, // default
        };
    }

    if let Some(limit) = params.limit {
        options.limit = Some(limit);
    }

    if let Some(offset) = params.offset {
        options.offset = Some(offset);
    }

    if let Some(name_pattern) = params.name_pattern {
        options.name_pattern = Some(name_pattern);
    }

    if let Some(content_type_filter) = params.content_type_filter {
        options.content_type_filter = Some(content_type_filter);
    }

    if let Some(min_size) = params.min_size {
        options.min_size = Some(min_size);
    }

    if let Some(max_size) = params.max_size {
        options.max_size = Some(max_size);
    }

    if let Some(modified_after) = params.modified_after {
        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&modified_after) {
            options.modified_after = Some(date.with_timezone(&chrono::Utc));
        }
    }

    if let Some(modified_before) = params.modified_before {
        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&modified_before) {
            options.modified_before = Some(date.with_timezone(&chrono::Utc));
        }
    }

    let listing = service.list_directory(&dir_path, options).await?;
    Ok(Json(listing))
}

/// Delete directory handler
pub async fn delete_directory(
    State(service): State<Arc<dyn FileService>>,
    Path(dir_path): Path<String>,
    Query(params): Query<DeleteDirectoryQuery>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let recursive = params.recursive.unwrap_or(false);
    let result = service.delete_directory(&dir_path, recursive).await?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "path": result.path,
        "message": result.message.unwrap_or_else(|| "Directory deleted successfully".to_string())
    })))
}

/// Copy directory handler
pub async fn copy_directory(
    State(service): State<Arc<dyn FileService>>,
    Path(source_path): Path<String>,
    Json(request): Json<DirectoryOperationRequest>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let mut options = CopyOptions::default();
    
    if let Some(overwrite) = request.overwrite {
        options.overwrite = overwrite;
    }
    
    if let Some(preserve_metadata) = request.preserve_metadata {
        options.preserve_metadata = preserve_metadata;
    }
    
    if let Some(preserve_permissions) = request.preserve_permissions {
        options.preserve_permissions = preserve_permissions;
    }

    let result = service.copy_directory(&source_path, &request.destination, options).await?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "source": source_path,
        "destination": request.destination,
        "bytes_transferred": result.bytes_transferred,
        "message": result.message.unwrap_or_else(|| "Directory copied successfully".to_string())
    })))
}

/// Move directory handler
pub async fn move_directory(
    State(service): State<Arc<dyn FileService>>,
    Path(source_path): Path<String>,
    Json(request): Json<DirectoryOperationRequest>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let mut options = CopyOptions::default();
    
    if let Some(overwrite) = request.overwrite {
        options.overwrite = overwrite;
    }
    
    if let Some(preserve_metadata) = request.preserve_metadata {
        options.preserve_metadata = preserve_metadata;
    }
    
    if let Some(preserve_permissions) = request.preserve_permissions {
        options.preserve_permissions = preserve_permissions;
    }

    let result = service.move_directory(&source_path, &request.destination, options).await?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "source": source_path,
        "destination": request.destination,
        "bytes_transferred": result.bytes_transferred,
        "message": result.message.unwrap_or_else(|| "Directory moved successfully".to_string())
    })))
}

/// Check if directory exists handler
pub async fn directory_exists(
    State(service): State<Arc<dyn FileService>>,
    Path(dir_path): Path<String>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let exists = service.exists(&dir_path).await?;

    Ok(Json(serde_json::json!({
        "exists": exists,
        "path": dir_path
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::LocalStorageProvider;
    use crate::models::config::FileServiceConfig;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        routing::{get, post, delete},
        Router,
    };
    use std::sync::Arc;
    use tempfile::TempDir;
    use tower::ServiceExt;

    async fn create_test_app() -> (Router, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = FileServiceConfig::local("test".to_string(), temp_dir.path().to_path_buf());
        let mut provider = LocalStorageProvider::new(config).unwrap();
        provider.initialize().await.unwrap();
        let service: Arc<dyn FileService> = Arc::new(provider);

        let app = Router::new()
            .route("/directories/*path", post(create_directory))
            .route("/directories/*path", get(list_directory))
            .route("/directories/*path", delete(delete_directory))
            .route("/directories/*path/exists", get(directory_exists))
            .route("/directories/*path/copy", post(copy_directory))
            .route("/directories/*path/move", post(move_directory))
            .with_state(service);

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_create_directory_handler() {
        let (app, _temp_dir) = create_test_app().await;

        let request = Request::builder()
            .method(Method::POST)
            .uri("/directories/test_dir")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let create_response: CreateDirectoryResponse = serde_json::from_slice(&body).unwrap();
        assert!(create_response.success);
        assert_eq!(create_response.path, "/test_dir");
    }

    #[tokio::test]
    async fn test_directory_exists_handler() {
        let (app, _temp_dir) = create_test_app().await;

        // Check non-existent directory
        let exists_request = Request::builder()
            .method(Method::GET)
            .uri("/directories/nonexistent/exists")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(exists_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["exists"], false);

        // Create directory
        let create_request = Request::builder()
            .method(Method::POST)
            .uri("/directories/test_dir")
            .body(Body::empty())
            .unwrap();

        let _create_response = app.clone().oneshot(create_request).await.unwrap();

        // Check existing directory
        let exists_request = Request::builder()
            .method(Method::GET)
            .uri("/directories/test_dir/exists")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(exists_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["exists"], true);
    }
}