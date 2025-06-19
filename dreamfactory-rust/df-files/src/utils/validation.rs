use crate::models::error::{FileServiceError, FileResult};
use std::path::Path;

/// Validate that a file path is safe (no directory traversal)
pub fn validate_path_safety(path: &str) -> FileResult<()> {
    // Check for directory traversal attempts
    if path.contains("..") {
        return Err(FileServiceError::InvalidPath {
            path: path.to_string(),
        });
    }
    
    // Check for absolute paths (should be relative to container root)
    if path.starts_with('/') {
        return Err(FileServiceError::InvalidPath {
            path: path.to_string(),
        });
    }
    
    // Check for empty path
    if path.is_empty() {
        return Err(FileServiceError::InvalidPath {
            path: path.to_string(),
        });
    }
    
    Ok(())
}

/// Validate file name for invalid characters
pub fn validate_filename(filename: &str) -> FileResult<()> {
    if filename.is_empty() {
        return Err(FileServiceError::InvalidPath {
            path: filename.to_string(),
        });
    }
    
    // Check for invalid characters
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0'];
    if filename.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(FileServiceError::InvalidPath {
            path: filename.to_string(),
        });
    }
    
    // Check for reserved names (Windows)
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
    ];
    
    let filename_upper = filename.to_uppercase();
    let name_part = filename_upper.split('.').next().unwrap_or(&filename_upper);
    
    if reserved_names.contains(&name_part) {
        return Err(FileServiceError::InvalidPath {
            path: filename.to_string(),
        });
    }
    
    Ok(())
}

/// Validate file size limits
pub fn validate_file_size(size: u64, max_size: Option<u64>) -> FileResult<()> {
    if let Some(max) = max_size {
        if size > max {
            return Err(FileServiceError::FileTooLarge {
                size,
                max_size: max,
            });
        }
    }
    
    Ok(())
}

/// Validate file extension against allowed types
pub fn validate_file_extension(filename: &str, allowed_extensions: &[&str]) -> FileResult<()> {
    if allowed_extensions.is_empty() {
        return Ok(()); // No restrictions
    }
    
    let path = Path::new(filename);
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    let extension_lower = extension.to_lowercase();
    
    if !allowed_extensions.iter().any(|&allowed| {
        allowed.to_lowercase() == extension_lower
    }) {
        return Err(FileServiceError::UnsupportedFileType {
            mime_type: format!("file extension not allowed: {}", extension_lower),
        });
    }
    
    Ok(())
}

/// Validate content type against allowed MIME types
pub fn validate_content_type(content_type: &str, allowed_types: &[&str]) -> FileResult<()> {
    if allowed_types.is_empty() {
        return Ok(()); // No restrictions
    }
    
    if !allowed_types.contains(&content_type) {
        return Err(FileServiceError::UnsupportedFileType {
            mime_type: content_type.to_string(),
        });
    }
    
    Ok(())
}

/// Comprehensive file validation
pub fn validate_file(
    path: &str,
    filename: &str,
    size: u64,
    content_type: &str,
    max_size: Option<u64>,
    allowed_extensions: &[&str],
    allowed_content_types: &[&str],
) -> FileResult<()> {
    validate_path_safety(path)?;
    validate_filename(filename)?;
    validate_file_size(size, max_size)?;
    validate_file_extension(filename, allowed_extensions)?;
    validate_content_type(content_type, allowed_content_types)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_safety() {
        assert!(validate_path_safety("safe/path/file.txt").is_ok());
        assert!(validate_path_safety("../traversal").is_err());
        assert!(validate_path_safety("/absolute/path").is_err());
        assert!(validate_path_safety("").is_err());
    }

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("valid_file.txt").is_ok());
        assert!(validate_filename("file with spaces.txt").is_ok());
        assert!(validate_filename("file<invalid>.txt").is_err());
        assert!(validate_filename("CON.txt").is_err());
        assert!(validate_filename("").is_err());
    }

    #[test]
    fn test_validate_file_size() {
        assert!(validate_file_size(1000, Some(2000)).is_ok());
        assert!(validate_file_size(3000, Some(2000)).is_err());
        assert!(validate_file_size(1000, None).is_ok());
    }

    #[test]
    fn test_validate_file_extension() {
        let allowed = vec!["txt", "pdf", "jpg"];
        assert!(validate_file_extension("file.txt", &allowed).is_ok());
        assert!(validate_file_extension("file.PDF", &allowed).is_ok()); // Case insensitive
        assert!(validate_file_extension("file.exe", &allowed).is_err());
        assert!(validate_file_extension("file.txt", &[]).is_ok()); // No restrictions
    }

    #[test]
    fn test_validate_content_type() {
        let allowed = vec!["text/plain", "application/pdf"];
        assert!(validate_content_type("text/plain", &allowed).is_ok());
        assert!(validate_content_type("image/png", &allowed).is_err());
        assert!(validate_content_type("text/plain", &[]).is_ok()); // No restrictions
    }
}