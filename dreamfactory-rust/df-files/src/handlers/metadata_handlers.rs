use crate::models::{error::FileServiceError, file_info::FileInfo};
use axum::{extract::Path, response::Json};
use serde_json::{json, Value};
use tracing::{info, instrument};

/// Get metadata for a file or directory
#[instrument]
pub async fn get_metadata(
    Path(path): Path<String>,
) -> Result<Json<Value>, FileServiceError> {
    info!("Getting metadata for path: {}", path);
    
    // For now, return mock metadata
    let metadata = FileInfo::new(
        path.clone(),
        path.split('/').last().unwrap_or(&path).to_string(),
    );
    
    Ok(Json(json!({
        "resource": [metadata]
    })))
}

/// Set metadata for a file or directory
#[instrument]
pub async fn set_metadata(
    Path(path): Path<String>,
    Json(metadata): Json<Value>,
) -> Result<Json<Value>, FileServiceError> {
    info!("Setting metadata for path: {}", path);
    
    // For now, return success
    Ok(Json(json!({
        "success": true,
        "path": path,
        "metadata": metadata
    })))
}

/// Delete metadata for a file or directory
#[instrument]
pub async fn delete_metadata(
    Path(path): Path<String>,
) -> Result<Json<Value>, FileServiceError> {
    info!("Deleting metadata for path: {}", path);
    
    // For now, return success
    Ok(Json(json!({
        "success": true,
        "path": path
    })))
}