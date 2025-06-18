use crate::models::error::{FileResult, FileServiceError};
use std::path::{Path, PathBuf};

/// Utility functions for path manipulation and validation
pub struct PathUtils;

impl PathUtils {
    /// Normalize a file path by removing dangerous components and ensuring consistency
    pub fn normalize_path(path: &str) -> FileResult<String> {
        if path.is_empty() {
            return Err(FileServiceError::InvalidPath {
                path: path.to_string(),
            });
        }

        // Convert backslashes to forward slashes for consistency
        let normalized = path.replace('\\', "/");

        // Split path and filter out dangerous components
        let components: Vec<&str> = normalized
            .split('/')
            .filter(|component| {
                !component.is_empty() && *component != "." && *component != ".."
            })
            .collect();

        // Rebuild the path
        let result = if components.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", components.join("/"))
        };

        Ok(result)
    }

    /// Join multiple path components safely
    pub fn join_paths(base: &str, parts: &[&str]) -> FileResult<String> {
        let mut result = Self::normalize_path(base)?;
        
        for part in parts {
            let normalized_part = Self::normalize_path(part)?;
            // Remove leading slash from part to avoid double slashes
            let part_clean = normalized_part.strip_prefix('/').unwrap_or(&normalized_part);
            
            if result.ends_with('/') {
                result.push_str(part_clean);
            } else {
                result.push('/');
                result.push_str(part_clean);
            }
        }

        Ok(result)
    }

    /// Get the parent directory of a path
    pub fn get_parent(path: &str) -> Option<String> {
        let normalized = Self::normalize_path(path).ok()?;
        
        if normalized == "/" {
            return None;
        }

        let path_obj = Path::new(&normalized);
        path_obj.parent()?.to_str().map(|s| {
            if s.is_empty() {
                "/".to_string()
            } else {
                s.to_string()
            }
        })
    }

    /// Get the filename from a path
    pub fn get_filename(path: &str) -> Option<String> {
        let normalized = Self::normalize_path(path).ok()?;
        let path_obj = Path::new(&normalized);
        path_obj.file_name()?.to_str().map(|s| s.to_string())
    }

    /// Get the file extension from a path
    pub fn get_extension(path: &str) -> Option<String> {
        let path_obj = Path::new(path);
        path_obj.extension()?.to_str().map(|s| s.to_lowercase())
    }

    /// Get the file stem (filename without extension) from a path
    pub fn get_file_stem(path: &str) -> Option<String> {
        let path_obj = Path::new(path);
        path_obj.file_stem()?.to_str().map(|s| s.to_string())
    }

    /// Check if a path is absolute
    pub fn is_absolute(path: &str) -> bool {
        Path::new(path).is_absolute()
    }

    /// Check if a path is relative
    pub fn is_relative(path: &str) -> bool {
        !Self::is_absolute(path)
    }

    /// Convert a relative path to absolute using a base path
    pub fn to_absolute(path: &str, base: &str) -> FileResult<String> {
        if Self::is_absolute(path) {
            Self::normalize_path(path)
        } else {
            Self::join_paths(base, &[path])
        }
    }

    /// Check if a path is a child of another path
    pub fn is_child_of(child_path: &str, parent_path: &str) -> FileResult<bool> {
        let normalized_child = Self::normalize_path(child_path)?;
        let normalized_parent = Self::normalize_path(parent_path)?;

        // Ensure parent path ends with / for proper prefix checking
        let parent_with_slash = if normalized_parent.ends_with('/') {
            normalized_parent
        } else {
            format!("{}/", normalized_parent)
        };

        Ok(normalized_child.starts_with(&parent_with_slash) || normalized_child == normalized_parent.trim_end_matches('/'))
    }

    /// Calculate the relative path from one path to another
    pub fn relative_path(from: &str, to: &str) -> FileResult<String> {
        let from_normalized = Self::normalize_path(from)?;
        let to_normalized = Self::normalize_path(to)?;

        let from_components: Vec<&str> = from_normalized.split('/').filter(|s| !s.is_empty()).collect();
        let to_components: Vec<&str> = to_normalized.split('/').filter(|s| !s.is_empty()).collect();

        // Find common prefix
        let mut common_length = 0;
        for (i, (from_part, to_part)) in from_components.iter().zip(to_components.iter()).enumerate() {
            if from_part == to_part {
                common_length = i + 1;
            } else {
                break;
            }
        }

        // Build relative path
        let mut result_components = Vec::new();

        // Add .. for each remaining component in from_path
        for _ in common_length..from_components.len() {
            result_components.push("..");
        }

        // Add remaining components from to_path
        for component in &to_components[common_length..] {
            result_components.push(component);
        }

        if result_components.is_empty() {
            Ok(".".to_string())
        } else {
            Ok(result_components.join("/"))
        }
    }

    /// Check if a filename is hidden (starts with .)
    pub fn is_hidden(filename: &str) -> bool {
        filename.starts_with('.')
    }

    /// Validate a path for security issues
    pub fn validate_path_security(path: &str) -> FileResult<()> {
        // Check for null bytes
        if path.contains('\0') {
            return Err(FileServiceError::InvalidPath {
                path: path.to_string(),
            });
        }

        // Check for path traversal attempts
        if path.contains("..") {
            return Err(FileServiceError::InvalidPath {
                path: path.to_string(),
            });
        }

        // Check for absolute paths starting with /
        if path.starts_with('/') && path.len() > 1 && !path.chars().nth(1).unwrap().is_alphanumeric() {
            return Err(FileServiceError::InvalidPath {
                path: path.to_string(),
            });
        }

        // Check for reserved characters (Windows)
        let reserved_chars = ['<', '>', ':', '"', '|', '?', '*'];
        if path.chars().any(|c| reserved_chars.contains(&c)) {
            return Err(FileServiceError::InvalidPath {
                path: path.to_string(),
            });
        }

        // Check for reserved names (Windows)
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];

        if let Some(filename) = Self::get_filename(path) {
            let filename_upper = filename.to_uppercase();
            let stem = Self::get_file_stem(&filename_upper).unwrap_or(filename_upper);
            
            if reserved_names.contains(&stem.as_str()) {
                return Err(FileServiceError::InvalidPath {
                    path: path.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Generate a unique filename by appending a number if the file already exists
    pub fn generate_unique_filename(base_path: &str, filename: &str) -> String {
        let path_obj = Path::new(filename);
        let stem = path_obj.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let extension = path_obj.extension().and_then(|s| s.to_str()).unwrap_or("");

        let mut counter = 1;
        loop {
            let new_filename = if extension.is_empty() {
                if counter == 1 {
                    filename.to_string()
                } else {
                    format!("{}_{}", stem, counter)
                }
            } else {
                if counter == 1 {
                    filename.to_string()
                } else {
                    format!("{}_{}.{}", stem, counter, extension)
                }
            };

            let full_path = Self::join_paths(base_path, &[&new_filename]);
            if full_path.is_err() {
                // If we can't join paths, just return the new filename
                return new_filename;
            }

            // In a real implementation, you would check if the file exists
            // For now, we'll just return the first generated name
            return new_filename;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(PathUtils::normalize_path("/test/file.txt").unwrap(), "/test/file.txt");
        assert_eq!(PathUtils::normalize_path("test/file.txt").unwrap(), "/test/file.txt");
        assert_eq!(PathUtils::normalize_path("/test/../file.txt").unwrap(), "/file.txt");
        assert_eq!(PathUtils::normalize_path("/test/./file.txt").unwrap(), "/test/file.txt");
        assert_eq!(PathUtils::normalize_path("\\test\\file.txt").unwrap(), "/test/file.txt");
        assert_eq!(PathUtils::normalize_path("").unwrap_err().to_string().contains("Invalid"), true);
    }

    #[test]
    fn test_join_paths() {
        assert_eq!(
            PathUtils::join_paths("/base", &["sub", "file.txt"]).unwrap(),
            "/base/sub/file.txt"
        );
        assert_eq!(
            PathUtils::join_paths("/base/", &["sub", "file.txt"]).unwrap(),
            "/base/sub/file.txt"
        );
        assert_eq!(
            PathUtils::join_paths("/", &["test"]).unwrap(),
            "/test"
        );
    }

    #[test]
    fn test_get_parent() {
        assert_eq!(PathUtils::get_parent("/test/file.txt"), Some("/test".to_string()));
        assert_eq!(PathUtils::get_parent("/test"), Some("/".to_string()));
        assert_eq!(PathUtils::get_parent("/"), None);
        assert_eq!(PathUtils::get_parent("file.txt"), Some("/".to_string()));
    }

    #[test]
    fn test_get_filename() {
        assert_eq!(PathUtils::get_filename("/test/file.txt"), Some("file.txt".to_string()));
        assert_eq!(PathUtils::get_filename("/test/"), None);
        assert_eq!(PathUtils::get_filename("file.txt"), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(PathUtils::get_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(PathUtils::get_extension("file.TXT"), Some("txt".to_string()));
        assert_eq!(PathUtils::get_extension("file"), None);
        assert_eq!(PathUtils::get_extension(".hidden"), None);
        assert_eq!(PathUtils::get_extension("file.tar.gz"), Some("gz".to_string()));
    }

    #[test]
    fn test_get_file_stem() {
        assert_eq!(PathUtils::get_file_stem("file.txt"), Some("file".to_string()));
        assert_eq!(PathUtils::get_file_stem("file"), Some("file".to_string()));
        assert_eq!(PathUtils::get_file_stem(".hidden"), Some(".hidden".to_string()));
    }

    #[test]
    fn test_is_absolute() {
        assert!(PathUtils::is_absolute("/test/file.txt"));
        assert!(!PathUtils::is_absolute("test/file.txt"));
        
        // Windows paths
        #[cfg(windows)]
        {
            assert!(PathUtils::is_absolute("C:\\test\\file.txt"));
            assert!(!PathUtils::is_absolute("test\\file.txt"));
        }
    }

    #[test]
    fn test_is_child_of() {
        assert!(PathUtils::is_child_of("/parent/child/file.txt", "/parent").unwrap());
        assert!(PathUtils::is_child_of("/parent/child", "/parent").unwrap());
        assert!(PathUtils::is_child_of("/parent", "/parent").unwrap());
        assert!(!PathUtils::is_child_of("/other/file.txt", "/parent").unwrap());
        assert!(!PathUtils::is_child_of("/parentx/file.txt", "/parent").unwrap());
    }

    #[test]
    fn test_relative_path() {
        assert_eq!(
            PathUtils::relative_path("/base/dir1", "/base/dir2/file.txt").unwrap(),
            "../dir2/file.txt"
        );
        assert_eq!(
            PathUtils::relative_path("/base/dir1", "/base/dir1/file.txt").unwrap(),
            "file.txt"
        );
        assert_eq!(
            PathUtils::relative_path("/base/dir1", "/base/dir1").unwrap(),
            "."
        );
    }

    #[test]
    fn test_is_hidden() {
        assert!(PathUtils::is_hidden(".hidden"));
        assert!(PathUtils::is_hidden(".bashrc"));
        assert!(!PathUtils::is_hidden("visible.txt"));
        assert!(!PathUtils::is_hidden("not.hidden"));
    }

    #[test]
    fn test_validate_path_security() {
        assert!(PathUtils::validate_path_security("/normal/path.txt").is_ok());
        assert!(PathUtils::validate_path_security("normal/path.txt").is_ok());
        
        // Path traversal attempts
        assert!(PathUtils::validate_path_security("../etc/passwd").is_err());
        assert!(PathUtils::validate_path_security("/normal/../etc/passwd").is_err());
        
        // Null bytes
        assert!(PathUtils::validate_path_security("/path\0/file.txt").is_err());
        
        // Reserved characters
        assert!(PathUtils::validate_path_security("/path/file<.txt").is_err());
        assert!(PathUtils::validate_path_security("/path/file>.txt").is_err());
        assert!(PathUtils::validate_path_security("/path/file|.txt").is_err());
        
        // Reserved names (Windows)
        assert!(PathUtils::validate_path_security("/path/CON.txt").is_err());
        assert!(PathUtils::validate_path_security("/path/con").is_err());
        assert!(PathUtils::validate_path_security("/path/COM1.log").is_err());
    }

    #[test]
    fn test_generate_unique_filename() {
        let unique = PathUtils::generate_unique_filename("/base", "test.txt");
        assert_eq!(unique, "test.txt");

        let unique = PathUtils::generate_unique_filename("/base", "test");
        assert_eq!(unique, "test");
    }
}