use crate::models::error::FileResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// File metadata information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileInfo {
    /// File path relative to the service root
    pub path: String,
    /// File name (last component of path)
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Content type / MIME type
    pub content_type: String,
    /// File creation timestamp
    pub created_at: DateTime<Utc>,
    /// File last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// File last accessed timestamp (if available)
    pub accessed_at: Option<DateTime<Utc>>,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File permissions (Unix-style octal, e.g., 755)
    pub permissions: Option<u32>,
    /// File owner
    pub owner: Option<String>,
    /// File group
    pub group: Option<String>,
    /// ETag or version identifier
    pub etag: Option<String>,
    /// Checksum/hash of file content
    pub checksum: Option<String>,
    /// Checksum algorithm used (e.g., "md5", "sha256")
    pub checksum_algorithm: Option<String>,
    /// Additional metadata from storage provider
    pub metadata: HashMap<String, String>,
    /// Whether the file is encrypted
    pub encrypted: bool,
    /// Compression type if compressed
    pub compression: Option<String>,
}

impl FileInfo {
    /// Create a new FileInfo instance
    pub fn new(path: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            path,
            name,
            size: 0,
            content_type: "application/octet-stream".to_string(),
            created_at: now,
            modified_at: now,
            accessed_at: None,
            is_directory: false,
            permissions: None,
            owner: None,
            group: None,
            etag: None,
            checksum: None,
            checksum_algorithm: None,
            metadata: HashMap::new(),
            encrypted: false,
            compression: None,
        }
    }

    /// Create FileInfo for a directory
    pub fn directory(path: String, name: String) -> Self {
        let mut info = Self::new(path, name);
        info.is_directory = true;
        info.content_type = "inode/directory".to_string();
        info
    }

    /// Set file size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = content_type;
        self
    }

    /// Set timestamps
    pub fn with_timestamps(
        mut self,
        created: DateTime<Utc>,
        modified: DateTime<Utc>,
    ) -> Self {
        self.created_at = created;
        self.modified_at = modified;
        self
    }

    /// Set permissions
    pub fn with_permissions(mut self, permissions: u32) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Set owner information
    pub fn with_owner(mut self, owner: String, group: Option<String>) -> Self {
        self.owner = Some(owner);
        self.group = group;
        self
    }

    /// Set ETag
    pub fn with_etag(mut self, etag: String) -> Self {
        self.etag = Some(etag);
        self
    }

    /// Set checksum
    pub fn with_checksum(mut self, checksum: String, algorithm: String) -> Self {
        self.checksum = Some(checksum);
        self.checksum_algorithm = Some(algorithm);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set encryption status
    pub fn with_encryption(mut self, encrypted: bool) -> Self {
        self.encrypted = encrypted;
        self
    }

    /// Set compression
    pub fn with_compression(mut self, compression: String) -> Self {
        self.compression = Some(compression);
        self
    }

    /// Get the file extension
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.name)
            .extension()
            .and_then(|ext| ext.to_str())
    }

    /// Get the parent directory path
    pub fn parent_path(&self) -> Option<String> {
        if self.path == "/" || self.path.is_empty() {
            return None;
        }
        
        let path = std::path::Path::new(&self.path);
        path.parent()
            .and_then(|p| p.to_str())
            .map(|p| if p.is_empty() { "/" } else { p }.to_string())
    }

    /// Check if the file is hidden (starts with .)
    pub fn is_hidden(&self) -> bool {
        self.name.starts_with('.')
    }

    /// Get human-readable file size
    pub fn human_readable_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = self.size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", self.size, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Validate file info for consistency
    pub fn validate(&self) -> FileResult<()> {
        use crate::models::error::FileServiceError;

        if self.path.is_empty() {
            return Err(FileServiceError::InvalidPath {
                path: self.path.clone(),
            });
        }

        if self.name.is_empty() {
            return Err(FileServiceError::InvalidPath {
                path: self.path.clone(),
            });
        }

        if self.is_directory && self.size > 0 {
            return Err(FileServiceError::InternalError {
                message: "Directory cannot have non-zero size".to_string(),
            });
        }

        if let (Some(checksum), None) = (&self.checksum, &self.checksum_algorithm) {
            return Err(FileServiceError::InternalError {
                message: "Checksum provided without algorithm".to_string(),
            });
        }

        Ok(())
    }
}

/// Directory listing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    /// Directory path
    pub path: String,
    /// Files and subdirectories
    pub entries: Vec<FileInfo>,
    /// Total number of entries
    pub total_count: usize,
    /// Whether there are more entries (for pagination)
    pub has_more: bool,
    /// Continuation token for pagination
    pub continuation_token: Option<String>,
}

impl DirectoryListing {
    /// Create a new directory listing
    pub fn new(path: String, entries: Vec<FileInfo>) -> Self {
        let total_count = entries.len();
        Self {
            path,
            entries,
            total_count,
            has_more: false,
            continuation_token: None,
        }
    }

    /// Create paginated directory listing
    pub fn paginated(
        path: String,
        entries: Vec<FileInfo>,
        has_more: bool,
        continuation_token: Option<String>,
    ) -> Self {
        let total_count = entries.len();
        Self {
            path,
            entries,
            total_count,
            has_more,
            continuation_token,
        }
    }

    /// Get only files (non-directories)
    pub fn files(&self) -> Vec<&FileInfo> {
        self.entries.iter().filter(|e| !e.is_directory).collect()
    }

    /// Get only directories
    pub fn directories(&self) -> Vec<&FileInfo> {
        self.entries.iter().filter(|e| e.is_directory).collect()
    }

    /// Sort entries by name
    pub fn sort_by_name(&mut self) {
        self.entries.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Sort entries by modified date
    pub fn sort_by_modified(&mut self) {
        self.entries.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    }

    /// Sort entries by size
    pub fn sort_by_size(&mut self) {
        self.entries.sort_by(|a, b| b.size.cmp(&a.size));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let info = FileInfo::new("test/file.txt".to_string(), "file.txt".to_string());
        assert_eq!(info.path, "test/file.txt");
        assert_eq!(info.name, "file.txt");
        assert_eq!(info.size, 0);
        assert!(!info.is_directory);
    }

    #[test]
    fn test_directory_creation() {
        let info = FileInfo::directory("test/dir".to_string(), "dir".to_string());
        assert!(info.is_directory);
        assert_eq!(info.content_type, "inode/directory");
    }

    #[test]
    fn test_file_info_builder() {
        let info = FileInfo::new("test.txt".to_string(), "test.txt".to_string())
            .with_size(1024)
            .with_content_type("text/plain".to_string())
            .with_permissions(644)
            .with_checksum("abc123".to_string(), "md5".to_string());

        assert_eq!(info.size, 1024);
        assert_eq!(info.content_type, "text/plain");
        assert_eq!(info.permissions, Some(644));
        assert_eq!(info.checksum.as_ref().unwrap(), "abc123");
        assert_eq!(info.checksum_algorithm.as_ref().unwrap(), "md5");
    }

    #[test]
    fn test_file_extension() {
        let info = FileInfo::new("test.txt".to_string(), "test.txt".to_string());
        assert_eq!(info.extension(), Some("txt"));

        let info = FileInfo::new("noext".to_string(), "noext".to_string());
        assert_eq!(info.extension(), None);
    }

    #[test]
    fn test_parent_path() {
        let info = FileInfo::new("dir/subdir/file.txt".to_string(), "file.txt".to_string());
        assert_eq!(info.parent_path(), Some("dir/subdir".to_string()));

        let info = FileInfo::new("file.txt".to_string(), "file.txt".to_string());
        assert_eq!(info.parent_path(), Some("/".to_string()));

        let info = FileInfo::new("/".to_string(), "/".to_string());
        assert_eq!(info.parent_path(), None);
    }

    #[test]
    fn test_is_hidden() {
        let info = FileInfo::new(".hidden".to_string(), ".hidden".to_string());
        assert!(info.is_hidden());

        let info = FileInfo::new("visible.txt".to_string(), "visible.txt".to_string());
        assert!(!info.is_hidden());
    }

    #[test]
    fn test_human_readable_size() {
        let info = FileInfo::new("test".to_string(), "test".to_string()).with_size(1024);
        assert_eq!(info.human_readable_size(), "1.0 KB");

        let info = FileInfo::new("test".to_string(), "test".to_string()).with_size(1048576);
        assert_eq!(info.human_readable_size(), "1.0 MB");

        let info = FileInfo::new("test".to_string(), "test".to_string()).with_size(512);
        assert_eq!(info.human_readable_size(), "512 B");
    }

    #[test]
    fn test_file_info_validation() {
        let info = FileInfo::new("test.txt".to_string(), "test.txt".to_string());
        assert!(info.validate().is_ok());

        let info = FileInfo::new("".to_string(), "test.txt".to_string());
        assert!(info.validate().is_err());

        let mut info = FileInfo::directory("test".to_string(), "test".to_string());
        info.size = 100; // Invalid: directory with size
        assert!(info.validate().is_err());
    }

    #[test]
    fn test_directory_listing() {
        let files = vec![
            FileInfo::new("file1.txt".to_string(), "file1.txt".to_string()),
            FileInfo::directory("dir1".to_string(), "dir1".to_string()),
        ];

        let listing = DirectoryListing::new("/".to_string(), files);
        assert_eq!(listing.files().len(), 1);
        assert_eq!(listing.directories().len(), 1);
        assert_eq!(listing.total_count, 2);
    }

    #[test]
    fn test_directory_listing_sorting() {
        let mut listing = DirectoryListing::new(
            "/".to_string(),
            vec![
                FileInfo::new("z.txt".to_string(), "z.txt".to_string()),
                FileInfo::new("a.txt".to_string(), "a.txt".to_string()),
            ],
        );

        listing.sort_by_name();
        assert_eq!(listing.entries[0].name, "a.txt");
        assert_eq!(listing.entries[1].name, "z.txt");
    }
}