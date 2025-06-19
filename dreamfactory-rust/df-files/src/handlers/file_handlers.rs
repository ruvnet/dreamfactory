use crate::models::{
    error::FileServiceError,
    file_operation::{DownloadOptions, UploadOptions},
};
use crate::traits::FileService;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// File upload request
#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub content_type: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub overwrite: Option<bool>,
    pub create_parents: Option<bool>,
    pub permissions: Option<u32>,
    pub checksum: Option<String>,
    pub checksum_algorithm: Option<String>,
    pub encrypt: Option<bool>,
}

/// File download query parameters
#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub range_start: Option<u64>,
    pub range_end: Option<u64>,
    pub if_match: Option<String>,
    pub if_modified_since: Option<String>,
    pub include_metadata: Option<bool>,
    pub verify_checksum: Option<bool>,
}

/// File upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub path: String,
    pub size: Option<u64>,
    pub message: String,
}

/// File download response (for metadata endpoint)
#[derive(Debug, Serialize)]
pub struct FileInfoResponse {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub content_type: String,
    pub created_at: String,
    pub modified_at: String,
    pub is_directory: bool,
    pub etag: Option<String>,
    pub checksum: Option<String>,
    pub checksum_algorithm: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Upload file handler
pub async fn upload_file(
    State(service): State<Arc<dyn FileService>>,
    Path(file_path): Path<String>,
    Query(params): Query<UploadRequest>,
    body: Bytes,
) -> Result<Json<UploadResponse>, FileServiceError> {
    let mut options = UploadOptions::default();

    if let Some(content_type) = params.content_type {
        options = options.with_content_type(content_type);
    }

    if let Some(metadata) = params.metadata {
        for (key, value) in metadata {
            options = options.with_metadata(key, value);
        }
    }

    if let Some(overwrite) = params.overwrite {
        options = options.with_overwrite(overwrite);
    }

    if let Some(create_parents) = params.create_parents {
        options.create_parents = create_parents;
    }

    if let Some(permissions) = params.permissions {
        options = options.with_permissions(permissions);
    }

    if let (Some(checksum), Some(algorithm)) = (params.checksum, params.checksum_algorithm) {
        options = options.with_checksum(checksum, algorithm);
    }

    if let Some(encrypt) = params.encrypt {
        options = options.with_encryption(encrypt);
    }

    let result = service.upload_bytes(&file_path, body, options).await?;

    Ok(Json(UploadResponse {
        success: result.success,
        path: result.path,
        size: result.bytes_transferred,
        message: result.message.unwrap_or_else(|| "File uploaded successfully".to_string()),
    }))
}

/// Download file handler
pub async fn download_file(
    State(service): State<Arc<dyn FileService>>,
    Path(file_path): Path<String>,
    Query(params): Query<DownloadQuery>,
    headers: HeaderMap,
) -> Result<Response, FileServiceError> {
    let mut options = DownloadOptions::default();

    // Set range if specified
    if let (Some(start), end) = (params.range_start, params.range_end) {
        options = options.with_range(start, end);
    }

    // Set conditional headers
    if let Some(if_match) = params.if_match {
        options = options.with_if_match(if_match);
    }

    if let Some(if_modified_since) = params.if_modified_since {
        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&if_modified_since) {
            options = options.with_if_modified_since(date.with_timezone(&chrono::Utc));
        }
    }

    if let Some(verify_checksum) = params.verify_checksum {
        options = options.with_checksum_verification(verify_checksum);
    }

    // Check if client wants streaming response
    let use_streaming = headers.get("accept").map_or(false, |v| {
        v.to_str().unwrap_or("").contains("application/octet-stream")
    });

    if use_streaming {
        // Stream response
        let (stream, file_info) = service.download_stream(&file_path, options).await?;

        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&file_info.content_type).unwrap(),
        );
        response_headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&file_info.size.to_string()).unwrap(),
        );

        if let Some(etag) = &file_info.etag {
            response_headers.insert(header::ETAG, HeaderValue::from_str(etag).unwrap());
        }

        response_headers.insert(
            header::LAST_MODIFIED,
            HeaderValue::from_str(&file_info.modified_at.format("%a, %d %b %Y %H:%M:%S GMT").to_string()).unwrap(),
        );

        let body = Body::from_stream(stream);
        Ok((response_headers, body).into_response())
    } else {
        // Bytes response
        let (content, file_info) = service.download_bytes(&file_path, options).await?;

        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&file_info.content_type).unwrap(),
        );
        response_headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&content.len().to_string()).unwrap(),
        );

        if let Some(etag) = &file_info.etag {
            response_headers.insert(header::ETAG, HeaderValue::from_str(etag).unwrap());
        }

        response_headers.insert(
            header::LAST_MODIFIED,
            HeaderValue::from_str(&file_info.modified_at.format("%a, %d %b %Y %H:%M:%S GMT").to_string()).unwrap(),
        );

        Ok((response_headers, content).into_response())
    }
}

/// Get file information handler
pub async fn get_file_info(
    State(service): State<Arc<dyn FileService>>,
    Path(file_path): Path<String>,
) -> Result<Json<FileInfoResponse>, FileServiceError> {
    let file_info = service.get_file_info(&file_path).await?;

    Ok(Json(FileInfoResponse {
        path: file_info.path,
        name: file_info.name,
        size: file_info.size,
        content_type: file_info.content_type,
        created_at: file_info.created_at.to_rfc3339(),
        modified_at: file_info.modified_at.to_rfc3339(),
        is_directory: file_info.is_directory,
        etag: file_info.etag,
        checksum: file_info.checksum,
        checksum_algorithm: file_info.checksum_algorithm,
        metadata: file_info.metadata,
    }))
}

/// Check if file exists handler
pub async fn file_exists(
    State(service): State<Arc<dyn FileService>>,
    Path(file_path): Path<String>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let exists = service.exists(&file_path).await?;

    Ok(Json(serde_json::json!({
        "exists": exists,
        "path": file_path
    })))
}

/// Delete file handler
pub async fn delete_file(
    State(service): State<Arc<dyn FileService>>,
    Path(file_path): Path<String>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let result = service.delete_file(&file_path).await?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "path": result.path,
        "message": result.message.unwrap_or_else(|| "File deleted successfully".to_string())
    })))
}

/// Copy file handler
#[derive(Debug, Deserialize)]
pub struct CopyRequest {
    pub destination: String,
    pub overwrite: Option<bool>,
    pub preserve_metadata: Option<bool>,
    pub preserve_permissions: Option<bool>,
}

pub async fn copy_file(
    State(service): State<Arc<dyn FileService>>,
    Path(source_path): Path<String>,
    Json(request): Json<CopyRequest>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let mut options = crate::models::file_operation::CopyOptions::default();
    
    if let Some(overwrite) = request.overwrite {
        options.overwrite = overwrite;
    }
    
    if let Some(preserve_metadata) = request.preserve_metadata {
        options.preserve_metadata = preserve_metadata;
    }
    
    if let Some(preserve_permissions) = request.preserve_permissions {
        options.preserve_permissions = preserve_permissions;
    }

    let result = service.copy_file(&source_path, &request.destination, options).await?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "source": source_path,
        "destination": request.destination,
        "bytes_transferred": result.bytes_transferred,
        "message": result.message.unwrap_or_else(|| "File copied successfully".to_string())
    })))
}

/// Move file handler
#[derive(Debug, Deserialize)]
pub struct MoveRequest {
    pub destination: String,
    pub overwrite: Option<bool>,
    pub preserve_metadata: Option<bool>,
    pub preserve_permissions: Option<bool>,
}

pub async fn move_file(
    State(service): State<Arc<dyn FileService>>,
    Path(source_path): Path<String>,
    Json(request): Json<MoveRequest>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let mut options = crate::models::file_operation::CopyOptions::default();
    
    if let Some(overwrite) = request.overwrite {
        options.overwrite = overwrite;
    }
    
    if let Some(preserve_metadata) = request.preserve_metadata {
        options.preserve_metadata = preserve_metadata;
    }
    
    if let Some(preserve_permissions) = request.preserve_permissions {
        options.preserve_permissions = preserve_permissions;
    }

    let result = service.move_file(&source_path, &request.destination, options).await?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "source": source_path,
        "destination": request.destination,
        "bytes_transferred": result.bytes_transferred,
        "message": result.message.unwrap_or_else(|| "File moved successfully".to_string())
    })))
}

/// Convert FileServiceError to HTTP response
impl IntoResponse for FileServiceError {
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        let body = Json(serde_json::json!({
            "error": {
                "code": self.status_code(),
                "message": self.to_string(),
                "type": match self {
                    FileServiceError::FileNotFound { .. } => "file_not_found",
                    FileServiceError::DirectoryNotFound { .. } => "directory_not_found",
                    FileServiceError::FileAlreadyExists { .. } => "file_already_exists",
                    FileServiceError::DirectoryAlreadyExists { .. } => "directory_already_exists",
                    FileServiceError::PermissionDenied { .. } => "permission_denied",
                    FileServiceError::InvalidPath { .. } => "invalid_path",
                    FileServiceError::FileTooLarge { .. } => "file_too_large",
                    FileServiceError::InsufficientSpace { .. } => "insufficient_space",
                    FileServiceError::UnsupportedFileType { .. } => "unsupported_file_type",
                    FileServiceError::InvalidMultipartUpload { .. } => "invalid_multipart_upload",
                    FileServiceError::IncompleteUpload { .. } => "incomplete_upload",
                    FileServiceError::ChecksumMismatch { .. } => "checksum_mismatch",
                    FileServiceError::CloudStorageError { .. } => "cloud_storage_error",
                    FileServiceError::NetworkError { .. } => "network_error",
                    FileServiceError::ConfigError { .. } => "config_error",
                    FileServiceError::IoError { .. } => "io_error",
                    FileServiceError::SerializationError { .. } => "serialization_error",
                    FileServiceError::AuthenticationError { .. } => "authentication_error",
                    FileServiceError::AuthorizationError { .. } => "authorization_error",
                    FileServiceError::ServiceUnavailable { .. } => "service_unavailable",
                    FileServiceError::TimeoutError { .. } => "timeout_error",
                    FileServiceError::InternalError { .. } => "internal_error",
                }
            }
        }));

        (status_code, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::LocalStorageProvider;
    use crate::models::config::FileServiceConfig;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        routing::{get, post, put, delete},
        Router,
    };
    use bytes::Bytes;
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
            .route("/files/*path", post(upload_file))
            .route("/files/*path", get(download_file))
            .route("/files/*path", delete(delete_file))
            .route("/files/*path/info", get(get_file_info))
            .route("/files/*path/exists", get(file_exists))
            .route("/files/*path/copy", post(copy_file))
            .route("/files/*path/move", post(move_file))
            .with_state(service);

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_upload_file_handler() {
        let (app, _temp_dir) = create_test_app().await;

        let request = Request::builder()
            .method(Method::POST)
            .uri("/files/test.txt?content_type=text/plain&overwrite=true")
            .body(Body::from("Hello, World!"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let upload_response: UploadResponse = serde_json::from_slice(&body).unwrap();
        assert!(upload_response.success);
        assert_eq!(upload_response.path, "/test.txt");
    }

    #[tokio::test]
    async fn test_download_file_handler() {
        let (app, _temp_dir) = create_test_app().await;

        // First upload a file
        let upload_request = Request::builder()
            .method(Method::POST)
            .uri("/files/test.txt")
            .body(Body::from("Hello, World!"))
            .unwrap();

        let _upload_response = app.clone().oneshot(upload_request).await.unwrap();

        // Then download it
        let download_request = Request::builder()
            .method(Method::GET)
            .uri("/files/test.txt")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(download_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, Bytes::from("Hello, World!"));
    }

    #[tokio::test]
    async fn test_file_info_handler() {
        let (app, _temp_dir) = create_test_app().await;

        // First upload a file
        let upload_request = Request::builder()
            .method(Method::POST)
            .uri("/files/test.txt")
            .body(Body::from("Hello, World!"))
            .unwrap();

        let _upload_response = app.clone().oneshot(upload_request).await.unwrap();

        // Then get file info
        let info_request = Request::builder()
            .method(Method::GET)
            .uri("/files/test.txt/info")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(info_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let file_info: FileInfoResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(file_info.path, "/test.txt");
        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.size, 13);
        assert!(!file_info.is_directory);
    }

    #[tokio::test]
    async fn test_file_exists_handler() {
        let (app, _temp_dir) = create_test_app().await;

        // Check non-existent file
        let exists_request = Request::builder()
            .method(Method::GET)
            .uri("/files/nonexistent.txt/exists")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(exists_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["exists"], false);

        // Upload a file
        let upload_request = Request::builder()
            .method(Method::POST)
            .uri("/files/test.txt")
            .body(Body::from("Hello, World!"))
            .unwrap();

        let _upload_response = app.clone().oneshot(upload_request).await.unwrap();

        // Check existing file
        let exists_request = Request::builder()
            .method(Method::GET)
            .uri("/files/test.txt/exists")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(exists_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["exists"], true);
    }

    #[tokio::test]
    async fn test_delete_file_handler() {
        let (app, _temp_dir) = create_test_app().await;

        // First upload a file
        let upload_request = Request::builder()
            .method(Method::POST)
            .uri("/files/test.txt")
            .body(Body::from("Hello, World!"))
            .unwrap();

        let _upload_response = app.clone().oneshot(upload_request).await.unwrap();

        // Then delete it
        let delete_request = Request::builder()
            .method(Method::DELETE)
            .uri("/files/test.txt")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(delete_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["success"], true);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let (app, _temp_dir) = create_test_app().await;

        // Try to download non-existent file
        let download_request = Request::builder()
            .method(Method::GET)
            .uri("/files/nonexistent.txt")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(download_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error["error"]["type"], "file_not_found");
        assert_eq!(error["error"]["code"], 404);
    }
}