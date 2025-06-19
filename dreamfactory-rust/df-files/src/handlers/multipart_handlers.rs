use crate::models::{
    error::FileServiceError,
    file_operation::{UploadOptions, UploadPart},
};
use crate::traits::FileService;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use bytes::Bytes;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// Multipart upload initiation request
#[derive(Debug, Deserialize)]
pub struct InitiateMultipartRequest {
    pub content_type: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub checksum: Option<String>,
    pub checksum_algorithm: Option<String>,
    pub encrypt: Option<bool>,
    pub permissions: Option<u32>,
    pub expected_size: Option<u64>,
}

/// Upload part request
#[derive(Debug, Deserialize)]
pub struct UploadPartRequest {
    pub part_number: u32,
    pub checksum: Option<String>,
    pub checksum_algorithm: Option<String>,
}

/// Complete multipart upload request
#[derive(Debug, Deserialize)]
pub struct CompleteMultipartRequest {
    pub parts: Vec<UploadPartInfo>,
}

/// Upload part information for completion
#[derive(Debug, Deserialize, Serialize)]
pub struct UploadPartInfo {
    pub part_number: u32,
    pub etag: String,
    pub size: Option<u64>,
}

/// Multipart upload response
#[derive(Debug, Serialize)]
pub struct MultipartUploadResponse {
    pub upload_id: String,
    pub path: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub upload_url: Option<String>,
}

/// Upload part response
#[derive(Debug, Serialize)]
pub struct UploadPartResponse {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
    pub upload_id: String,
}

/// Complete multipart upload response
#[derive(Debug, Serialize)]
pub struct CompleteMultipartResponse {
    pub success: bool,
    pub path: String,
    pub size: Option<u64>,
    pub etag: Option<String>,
    pub message: String,
}

/// List multipart uploads response
#[derive(Debug, Serialize)]
pub struct ListMultipartUploadsResponse {
    pub uploads: Vec<MultipartUploadInfo>,
    pub total_count: usize,
}

/// Multipart upload information
#[derive(Debug, Serialize)]
pub struct MultipartUploadInfo {
    pub upload_id: String,
    pub path: String,
    pub initiated_at: String,
    pub expires_at: Option<String>,
    pub parts_uploaded: u32,
    pub total_size: Option<u64>,
}

/// Initiate multipart upload handler
pub async fn initiate_multipart_upload(
    State(service): State<Arc<dyn FileService>>,
    Path(file_path): Path<String>,
    Query(params): Query<InitiateMultipartRequest>,
) -> Result<Json<MultipartUploadResponse>, FileServiceError> {
    let mut options = UploadOptions::default();

    if let Some(content_type) = params.content_type {
        options = options.with_content_type(content_type);
    }

    if let Some(metadata) = params.metadata {
        for (key, value) in metadata {
            options = options.with_metadata(key, value);
        }
    }

    if let (Some(checksum), Some(algorithm)) = (params.checksum, params.checksum_algorithm) {
        options = options.with_checksum(checksum, algorithm);
    }

    if let Some(encrypt) = params.encrypt {
        options = options.with_encryption(encrypt);
    }

    if let Some(permissions) = params.permissions {
        options = options.with_permissions(permissions);
    }

    // expected_size field doesn't exist on UploadOptions
    // if let Some(expected_size) = params.expected_size {
    //     options.expected_size = Some(expected_size);
    // }

    let upload = service.initiate_multipart_upload(&file_path, options).await?;

    Ok(Json(MultipartUploadResponse {
        upload_id: upload.upload_id,
        path: upload.path,
        created_at: upload.initiated_at.to_rfc3339(),
        expires_at: None, // expires_at field doesn't exist
        upload_url: None, // upload_url field doesn't exist
    }))
}

/// Upload part handler
pub async fn upload_part(
    State(service): State<Arc<dyn FileService>>,
    Path(upload_id): Path<String>,
    Query(params): Query<UploadPartRequest>,
    body: Bytes,
) -> Result<Json<UploadPartResponse>, FileServiceError> {
    let part = service.upload_part(&upload_id, params.part_number, body).await?;

    Ok(Json(UploadPartResponse {
        part_number: part.part_number,
        etag: part.etag,
        size: part.size,
        upload_id: upload_id,
    }))
}

/// Complete multipart upload handler
pub async fn complete_multipart_upload(
    State(service): State<Arc<dyn FileService>>,
    Path(upload_id): Path<String>,
    Json(request): Json<CompleteMultipartRequest>,
) -> Result<Json<CompleteMultipartResponse>, FileServiceError> {
    let parts: Vec<UploadPart> = request.parts.into_iter().map(|part_info| {
        UploadPart {
            part_number: part_info.part_number,
            etag: part_info.etag,
            size: part_info.size.unwrap_or(0),
            uploaded_at: Utc::now(),
            checksum: None,
        }
    }).collect();

    let result = service.complete_multipart_upload(&upload_id, parts).await?;

    Ok(Json(CompleteMultipartResponse {
        success: result.success,
        path: result.path,
        size: result.bytes_transferred,
        etag: None, // FileOperationResult doesn't have etag field
        message: result.message.unwrap_or_else(|| "Multipart upload completed successfully".to_string()),
    }))
}

/// Abort multipart upload handler
pub async fn abort_multipart_upload(
    State(service): State<Arc<dyn FileService>>,
    Path(upload_id): Path<String>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    let result = service.abort_multipart_upload(&upload_id).await?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "upload_id": upload_id,
        "message": result.message.unwrap_or_else(|| "Multipart upload aborted successfully".to_string())
    })))
}

/// List multipart uploads handler
pub async fn list_multipart_uploads(
    State(service): State<Arc<dyn FileService>>,
) -> Result<Json<ListMultipartUploadsResponse>, FileServiceError> {
    let uploads = service.list_multipart_uploads().await?;

    let upload_infos: Vec<MultipartUploadInfo> = uploads.into_iter().map(|upload| {
        MultipartUploadInfo {
            upload_id: upload.upload_id,
            path: upload.path,
            initiated_at: upload.initiated_at.to_rfc3339(),
            expires_at: None, // expires_at field doesn't exist
            parts_uploaded: upload.parts.len() as u32,
            total_size: upload.total_size,
        }
    }).collect();

    let total_count = upload_infos.len();

    Ok(Json(ListMultipartUploadsResponse {
        uploads: upload_infos,
        total_count,
    }))
}

/// Get multipart upload status handler
pub async fn get_multipart_upload_status(
    State(service): State<Arc<dyn FileService>>,
    Path(upload_id): Path<String>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    // Get all uploads and find the one with matching ID
    let uploads = service.list_multipart_uploads().await?;
    
    if let Some(upload) = uploads.into_iter().find(|u| u.upload_id == upload_id) {
        Ok(Json(serde_json::json!({
            "upload_id": upload.upload_id,
            "path": upload.path,
            "created_at": upload.initiated_at.to_rfc3339(),
            "expires_at": null, // expires_at field doesn't exist
            "parts": upload.parts,
            "total_size": upload.total_size,
            "status": "active"
        })))
    } else {
        Err(FileServiceError::InvalidMultipartUpload {
            reason: format!("Multipart upload {} not found", upload_id),
        })
    }
}

/// List parts of a multipart upload handler
pub async fn list_upload_parts(
    State(service): State<Arc<dyn FileService>>,
    Path(upload_id): Path<String>,
) -> Result<Json<serde_json::Value>, FileServiceError> {
    // Get all uploads and find the one with matching ID
    let uploads = service.list_multipart_uploads().await?;
    
    if let Some(upload) = uploads.into_iter().find(|u| u.upload_id == upload_id) {
        Ok(Json(serde_json::json!({
            "upload_id": upload.upload_id,
            "parts": upload.parts,
            "total_parts": upload.parts.len()
        })))
    } else {
        Err(FileServiceError::InvalidMultipartUpload {
            reason: format!("Multipart upload {} not found", upload_id),
        })
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
        routing::{get, post, delete},
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
            .route("/multipart/initiate/*path", post(initiate_multipart_upload))
            .route("/multipart/:upload_id/parts", post(upload_part))
            .route("/multipart/:upload_id/complete", post(complete_multipart_upload))
            .route("/multipart/:upload_id/abort", delete(abort_multipart_upload))
            .route("/multipart/:upload_id/status", get(get_multipart_upload_status))
            .route("/multipart/:upload_id/parts/list", get(list_upload_parts))
            .route("/multipart/uploads", get(list_multipart_uploads))
            .with_state(service);

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_initiate_multipart_upload_handler() {
        let (app, _temp_dir) = create_test_app().await;

        let request = Request::builder()
            .method(Method::POST)
            .uri("/multipart/initiate/large_file.bin?content_type=application/octet-stream&expected_size=1048576")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let upload_response: MultipartUploadResponse = serde_json::from_slice(&body).unwrap();
        assert!(!upload_response.upload_id.is_empty());
        assert_eq!(upload_response.path, "/large_file.bin");
    }

    #[tokio::test]
    async fn test_list_multipart_uploads_handler() {
        let (app, _temp_dir) = create_test_app().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/multipart/uploads")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let list_response: ListMultipartUploadsResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(list_response.total_count, list_response.uploads.len());
    }

    #[tokio::test]
    async fn test_multipart_upload_not_found() {
        let (app, _temp_dir) = create_test_app().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/multipart/nonexistent_upload_id/status")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error["error"]["type"], "invalid_multipart_upload");
    }
}