use df_files::{
    models::{
        config::FileServiceConfig,
        file_operation::{BatchOperation, BatchOperationType, DownloadOptions, ListOptions, UploadOptions},
        error::FileServiceError,
    },
    storage::LocalStorageProvider,
    traits::{FileService, StorageProvider},
};
use bytes::Bytes;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio_test;

async fn create_test_provider() -> (LocalStorageProvider, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config = FileServiceConfig::local("test".to_string(), temp_dir.path().to_path_buf());
    let mut provider = LocalStorageProvider::new(config).unwrap();
    provider.initialize().await.unwrap();
    (provider, temp_dir)
}

#[tokio::test]
async fn test_provider_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let config = FileServiceConfig::local("test".to_string(), temp_dir.path().to_path_buf());
    let mut provider = LocalStorageProvider::new(config).unwrap();

    assert!(!provider.is_ready());
    assert!(provider.initialize().await.is_ok());
    assert!(provider.is_ready());
}

#[tokio::test]
async fn test_provider_health_check() {
    let (provider, _temp_dir) = create_test_provider().await;

    let health = StorageProvider::health_check(&provider).await.unwrap();
    assert!(health.healthy);
    assert!(health.response_time_ms >= 0);
}

#[tokio::test]
async fn test_file_upload_success() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("Hello, World!");
    let options = UploadOptions::default();

    let result = provider.upload_bytes("/test.txt", content.clone(), options).await;
    assert!(result.is_ok());

    let file_result = result.unwrap();
    assert!(file_result.success);
    assert_eq!(file_result.operation, "upload");
    assert_eq!(file_result.path, "/test.txt");
    assert_eq!(file_result.bytes_transferred, Some(content.len() as u64));
}

#[tokio::test]
async fn test_file_upload_overwrite_protection() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("Initial content");
    let options = UploadOptions::default();

    // First upload should succeed
    provider.upload_bytes("/test.txt", content.clone(), options.clone()).await.unwrap();

    // Second upload without overwrite should fail
    let result = provider.upload_bytes("/test.txt", content, options).await;
    assert!(matches!(result.unwrap_err(), FileServiceError::FileAlreadyExists { .. }));
}

#[tokio::test]
async fn test_file_upload_with_overwrite() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content1 = Bytes::from("Initial content");
    let content2 = Bytes::from("Updated content");

    // First upload
    provider.upload_bytes("/test.txt", content1, UploadOptions::default()).await.unwrap();

    // Second upload with overwrite
    let options = UploadOptions::default().with_overwrite(true);
    let result = provider.upload_bytes("/test.txt", content2.clone(), options).await;
    assert!(result.is_ok());

    // Verify content was updated
    let (downloaded, _) = provider.download_bytes("/test.txt", DownloadOptions::default()).await.unwrap();
    assert_eq!(downloaded, content2);
}

#[tokio::test]
async fn test_file_download() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("Test file content");
    provider.upload_bytes("/test.txt", content.clone(), UploadOptions::default()).await.unwrap();

    let (downloaded_content, file_info) = provider
        .download_bytes("/test.txt", DownloadOptions::default())
        .await
        .unwrap();

    assert_eq!(downloaded_content, content);
    assert_eq!(file_info.name, "test.txt");
    assert_eq!(file_info.path, "/test.txt");
    assert_eq!(file_info.size, content.len() as u64);
    assert!(!file_info.is_directory);
}

#[tokio::test]
async fn test_file_download_nonexistent() {
    let (provider, _temp_dir) = create_test_provider().await;

    let result = provider.download_bytes("/nonexistent.txt", DownloadOptions::default()).await;
    assert!(matches!(result.unwrap_err(), FileServiceError::FileNotFound { .. }));
}

#[tokio::test]
async fn test_file_download_range() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("0123456789");
    provider.upload_bytes("/test.txt", content, UploadOptions::default()).await.unwrap();

    let options = DownloadOptions::default().with_range(2, Some(5));
    let (downloaded_content, _) = provider.download_bytes("/test.txt", options).await.unwrap();

    assert_eq!(downloaded_content, Bytes::from("2345"));
}

#[tokio::test]
async fn test_file_exists() {
    let (provider, _temp_dir) = create_test_provider().await;

    assert!(!provider.exists("/test.txt").await.unwrap());

    let content = Bytes::from("test");
    provider.upload_bytes("/test.txt", content, UploadOptions::default()).await.unwrap();

    assert!(provider.exists("/test.txt").await.unwrap());
}

#[tokio::test]
async fn test_get_file_info() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("Test content");
    provider.upload_bytes("/test.txt", content.clone(), UploadOptions::default()).await.unwrap();

    let file_info = provider.get_file_info("/test.txt").await.unwrap();
    assert_eq!(file_info.name, "test.txt");
    assert_eq!(file_info.path, "/test.txt");
    assert_eq!(file_info.size, content.len() as u64);
    assert!(!file_info.is_directory);
    assert!(file_info.content_type.starts_with("text/"));
}

#[tokio::test]
async fn test_file_deletion() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("test");
    provider.upload_bytes("/test.txt", content, UploadOptions::default()).await.unwrap();

    assert!(provider.exists("/test.txt").await.unwrap());

    let result = provider.delete_file("/test.txt").await.unwrap();
    assert!(result.success);
    assert_eq!(result.operation, "delete");

    assert!(!provider.exists("/test.txt").await.unwrap());
}

#[tokio::test]
async fn test_delete_nonexistent_file() {
    let (provider, _temp_dir) = create_test_provider().await;

    let result = provider.delete_file("/nonexistent.txt").await;
    assert!(matches!(result.unwrap_err(), FileServiceError::FileNotFound { .. }));
}

#[tokio::test]
async fn test_directory_creation() {
    let (provider, _temp_dir) = create_test_provider().await;

    let result = provider.create_directory("/testdir").await.unwrap();
    assert!(result.success);
    assert_eq!(result.operation, "create_directory");

    let file_info = provider.get_file_info("/testdir").await.unwrap();
    assert!(file_info.is_directory);
    assert_eq!(file_info.name, "testdir");
}

#[tokio::test]
async fn test_directory_listing_empty() {
    let (provider, _temp_dir) = create_test_provider().await;

    provider.create_directory("/testdir").await.unwrap();

    let listing = provider.list_directory("/testdir", ListOptions::default()).await.unwrap();
    assert_eq!(listing.path, "/testdir");
    assert_eq!(listing.entries.len(), 0);
    assert_eq!(listing.total_count, 0);
}

#[tokio::test]
async fn test_directory_listing_with_files() {
    let (provider, _temp_dir) = create_test_provider().await;

    provider.create_directory("/testdir").await.unwrap();

    let content = Bytes::from("test");
    provider.upload_bytes("/testdir/file1.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/testdir/file2.txt", content, UploadOptions::default()).await.unwrap();

    let listing = provider.list_directory("/testdir", ListOptions::default()).await.unwrap();
    assert_eq!(listing.entries.len(), 2);

    let names: Vec<_> = listing.entries.iter().map(|e| &e.name).collect();
    assert!(names.contains(&&"file1.txt".to_string()));
    assert!(names.contains(&&"file2.txt".to_string()));
}

#[tokio::test]
async fn test_directory_listing_filtering() {
    let (provider, _temp_dir) = create_test_provider().await;

    provider.create_directory("/testdir").await.unwrap();

    let content = Bytes::from("test");
    provider.upload_bytes("/testdir/file1.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/testdir/file2.log", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/testdir/.hidden", content, UploadOptions::default()).await.unwrap();

    // Test pattern filtering
    let mut options = ListOptions::default();
    options.pattern = Some("*.txt".to_string());
    let listing = provider.list_directory("/testdir", options).await.unwrap();
    assert_eq!(listing.entries.len(), 1);
    assert_eq!(listing.entries[0].name, "file1.txt");

    // Test hidden file filtering
    let mut options = ListOptions::default();
    options.include_hidden = false;
    let listing = provider.list_directory("/testdir", options).await.unwrap();
    assert_eq!(listing.entries.len(), 2); // Should not include .hidden

    // Test including hidden files
    let mut options = ListOptions::default();
    options.include_hidden = true;
    let listing = provider.list_directory("/testdir", options).await.unwrap();
    assert_eq!(listing.entries.len(), 3); // Should include .hidden
}

#[tokio::test]
async fn test_directory_deletion() {
    let (provider, _temp_dir) = create_test_provider().await;

    provider.create_directory("/testdir").await.unwrap();

    // Test empty directory deletion
    let result = provider.delete_directory("/testdir", false).await.unwrap();
    assert!(result.success);

    assert!(!provider.exists("/testdir").await.unwrap());
}

#[tokio::test]
async fn test_directory_deletion_recursive() {
    let (provider, _temp_dir) = create_test_provider().await;

    provider.create_directory("/testdir").await.unwrap();
    provider.create_directory("/testdir/subdir").await.unwrap();

    let content = Bytes::from("test");
    provider.upload_bytes("/testdir/file.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/testdir/subdir/file.txt", content, UploadOptions::default()).await.unwrap();

    // Test recursive deletion
    let result = provider.delete_directory("/testdir", true).await.unwrap();
    assert!(result.success);

    assert!(!provider.exists("/testdir").await.unwrap());
}

#[tokio::test]
async fn test_file_copy() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("Test content for copying");
    provider.upload_bytes("/source.txt", content.clone(), UploadOptions::default()).await.unwrap();

    let result = provider.copy_file("/source.txt", "/copy.txt", Default::default()).await.unwrap();
    assert!(result.success);
    assert_eq!(result.operation, "copy");

    // Verify both files exist
    assert!(provider.exists("/source.txt").await.unwrap());
    assert!(provider.exists("/copy.txt").await.unwrap());

    // Verify content is the same
    let (copied_content, _) = provider.download_bytes("/copy.txt", DownloadOptions::default()).await.unwrap();
    assert_eq!(copied_content, content);
}

#[tokio::test]
async fn test_file_move() {
    let (provider, _temp_dir) = create_test_provider().await;

    let content = Bytes::from("Test content for moving");
    provider.upload_bytes("/source.txt", content.clone(), UploadOptions::default()).await.unwrap();

    let result = provider.move_file("/source.txt", "/moved.txt", Default::default()).await.unwrap();
    assert!(result.success);
    assert_eq!(result.operation, "move");

    // Verify source no longer exists and destination does
    assert!(!provider.exists("/source.txt").await.unwrap());
    assert!(provider.exists("/moved.txt").await.unwrap());

    // Verify content is preserved
    let (moved_content, _) = provider.download_bytes("/moved.txt", DownloadOptions::default()).await.unwrap();
    assert_eq!(moved_content, content);
}

#[tokio::test]
async fn test_multipart_upload() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Initiate multipart upload
    let upload = provider
        .initiate_multipart_upload("/large_file.txt", UploadOptions::default())
        .await
        .unwrap();

    assert!(!upload.upload_id.is_empty());
    assert_eq!(upload.path, "/large_file.txt");

    // Upload parts
    let part1 = provider
        .upload_part(&upload.upload_id, 1, Bytes::from("Part 1 content"))
        .await
        .unwrap();

    let part2 = provider
        .upload_part(&upload.upload_id, 2, Bytes::from("Part 2 content"))
        .await
        .unwrap();

    assert_eq!(part1.part_number, 1);
    assert_eq!(part2.part_number, 2);

    // Complete upload
    let result = provider
        .complete_multipart_upload(&upload.upload_id, vec![part1, part2])
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.operation, "multipart_upload");

    // Verify file exists and has correct content
    assert!(provider.exists("/large_file.txt").await.unwrap());
    let (content, _) = provider
        .download_bytes("/large_file.txt", DownloadOptions::default())
        .await
        .unwrap();
    assert_eq!(content, Bytes::from("Part 1 contentPart 2 content"));
}

#[tokio::test]
async fn test_multipart_upload_abort() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Initiate multipart upload
    let upload = provider
        .initiate_multipart_upload("/aborted_file.txt", UploadOptions::default())
        .await
        .unwrap();

    // Upload a part
    provider
        .upload_part(&upload.upload_id, 1, Bytes::from("Part 1"))
        .await
        .unwrap();

    // Abort upload
    let result = provider.abort_multipart_upload(&upload.upload_id).await.unwrap();
    assert!(result.success);

    // Verify file doesn't exist
    assert!(!provider.exists("/aborted_file.txt").await.unwrap());
}

#[tokio::test]
async fn test_batch_delete_operation() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Create test files
    let content = Bytes::from("test content");
    provider.upload_bytes("/file1.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/file2.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/file3.txt", content, UploadOptions::default()).await.unwrap();

    // Batch delete
    let batch_op = BatchOperation {
        operation: BatchOperationType::Delete,
        sources: vec![
            "/file1.txt".to_string(),
            "/file2.txt".to_string(),
            "/file3.txt".to_string(),
        ],
        destination: None,
        options: HashMap::new(),
    };

    let result = provider.batch_operation(batch_op).await.unwrap();
    assert_eq!(result.total, 3);
    assert_eq!(result.successful, 3);
    assert_eq!(result.failed, 0);

    // Verify all files are deleted
    assert!(!provider.exists("/file1.txt").await.unwrap());
    assert!(!provider.exists("/file2.txt").await.unwrap());
    assert!(!provider.exists("/file3.txt").await.unwrap());
}

#[tokio::test]
async fn test_storage_usage() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Upload some files
    let content1 = Bytes::from("content1"); // 8 bytes
    let content2 = Bytes::from("content2"); // 8 bytes
    provider.upload_bytes("/file1.txt", content1, UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/file2.txt", content2, UploadOptions::default()).await.unwrap();

    let usage = provider.get_storage_usage(None).await.unwrap();
    assert_eq!(usage.file_count, 2);
    assert_eq!(usage.used_bytes, 16);
}

#[tokio::test]
async fn test_file_search() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Create test files
    let content = Bytes::from("test content");
    provider.upload_bytes("/document1.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/document2.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/image.jpg", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/data.log", content, UploadOptions::default()).await.unwrap();

    // Search for .txt files
    let criteria = df_files::traits::file_service::SearchCriteria {
        path: "/".to_string(),
        recursive: false,
        name_pattern: Some("*.txt".to_string()),
        content_type_pattern: None,
        min_size: None,
        max_size: None,
        modified_after: None,
        modified_before: None,
        metadata_filters: HashMap::new(),
        limit: None,
    };

    let results = provider.search_files(criteria).await.unwrap();
    assert_eq!(results.len(), 2);

    let names: Vec<_> = results.iter().map(|f| &f.name).collect();
    assert!(names.contains(&&"document1.txt".to_string()));
    assert!(names.contains(&&"document2.txt".to_string()));
}

#[tokio::test]
async fn test_service_info() {
    let (provider, _temp_dir) = create_test_provider().await;

    let info = provider.service_info();
    assert_eq!(info.name, "test");
    assert_eq!(info.provider, "local");
    assert!(!info.features.is_empty());
}

#[tokio::test]
async fn test_path_validation() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Valid paths should work
    let content = Bytes::from("test");
    assert!(provider.upload_bytes("/valid.txt", content.clone(), UploadOptions::default()).await.is_ok());
    assert!(provider.upload_bytes("valid2.txt", content.clone(), UploadOptions::default()).await.is_ok());
    assert!(provider.upload_bytes("/dir/valid3.txt", content, UploadOptions::default()).await.is_ok());

    // Invalid empty path should fail
    let result = provider.exists("").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let (provider, _temp_dir) = create_test_provider().await;

    let provider = std::sync::Arc::new(provider);
    let mut handles = Vec::new();

    // Spawn multiple concurrent upload operations
    for i in 0..10 {
        let provider = provider.clone();
        let handle = tokio::spawn(async move {
            let content = Bytes::from(format!("Content for file {}", i));
            let path = format!("/concurrent_file_{}.txt", i);
            provider.upload_bytes(&path, content, UploadOptions::default()).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all files exist
    for i in 0..10 {
        let path = format!("/concurrent_file_{}.txt", i);
        assert!(provider.exists(&path).await.unwrap());
    }
}

#[tokio::test]
async fn test_large_file_handling() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Create a moderately large file (1MB)
    let large_content = Bytes::from(vec![b'A'; 1024 * 1024]);
    
    let result = provider.upload_bytes("/large_file.txt", large_content.clone(), UploadOptions::default()).await;
    assert!(result.is_ok());

    let (downloaded_content, file_info) = provider
        .download_bytes("/large_file.txt", DownloadOptions::default())
        .await
        .unwrap();

    assert_eq!(downloaded_content.len(), large_content.len());
    assert_eq!(file_info.size, large_content.len() as u64);
}

#[tokio::test]
async fn test_nested_directory_operations() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Create nested directory structure
    let content = Bytes::from("nested content");
    let path = "/level1/level2/level3/nested_file.txt";
    
    let options = UploadOptions::default().with_overwrite(true);
    let result = provider.upload_bytes(path, content.clone(), options).await;
    assert!(result.is_ok());

    // Verify file exists
    assert!(provider.exists(path).await.unwrap());

    // Verify directory structure was created
    assert!(provider.exists("/level1").await.unwrap());
    assert!(provider.exists("/level1/level2").await.unwrap());
    assert!(provider.exists("/level1/level2/level3").await.unwrap());

    // Test recursive directory listing
    let mut options = ListOptions::default();
    options.recursive = true;
    let listing = provider.list_directory("/level1", options).await.unwrap();
    
    // Should contain the nested directories and file
    assert!(listing.entries.len() >= 4); // 3 directories + 1 file
}

#[tokio::test]
async fn test_statistics_tracking() {
    let (provider, _temp_dir) = create_test_provider().await;

    // Perform some operations
    let content = Bytes::from("test content");
    provider.upload_bytes("/test1.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.upload_bytes("/test2.txt", content.clone(), UploadOptions::default()).await.unwrap();
    provider.download_bytes("/test1.txt", DownloadOptions::default()).await.unwrap();

    let stats = provider.get_statistics().await.unwrap();
    assert!(stats.total_operations > 0);
    assert!(stats.successful_operations > 0);
    assert_eq!(stats.failed_operations, 0);
    assert!(stats.last_operation_at.is_some());
}