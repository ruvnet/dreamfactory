use df_files::{
    models::{
        config::FileServiceConfig,
        file_operation::{DownloadOptions, UploadOptions},
    },
    services::FileServiceFactory,
    traits::FileService,
};
use bytes::Bytes;
use std::sync::Arc;
use tempfile::TempDir;
use tokio_test;

async fn create_local_service() -> (Arc<dyn FileService>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config = FileServiceConfig::local("integration_test".to_string(), temp_dir.path().to_path_buf());
    let service = FileServiceFactory::create(config).await.unwrap();
    (service, temp_dir)
}

#[tokio::test]
async fn test_complete_file_workflow() {
    let (service, _temp_dir) = create_local_service().await;

    // Test service info
    let info = service.service_info();
    assert_eq!(info.name, "integration_test");
    assert_eq!(info.provider, "local");

    // Test health check
    assert!(service.health_check().await.is_ok());

    // Test file upload
    let content = Bytes::from("Hello, Integration Test!");
    let upload_result = service
        .upload_bytes("/test/integration.txt", content.clone(), UploadOptions::default())
        .await
        .unwrap();
    
    assert!(upload_result.success);
    assert_eq!(upload_result.path, "/test/integration.txt");
    assert_eq!(upload_result.bytes_transferred, Some(content.len() as u64));

    // Test file exists
    assert!(service.exists("/test/integration.txt").await.unwrap());
    assert!(!service.exists("/test/nonexistent.txt").await.unwrap());

    // Test file info
    let file_info = service.get_file_info("/test/integration.txt").await.unwrap();
    assert_eq!(file_info.name, "integration.txt");
    assert_eq!(file_info.size, content.len() as u64);
    assert!(!file_info.is_directory);

    // Test file download
    let (downloaded_content, download_info) = service
        .download_bytes("/test/integration.txt", DownloadOptions::default())
        .await
        .unwrap();
    
    assert_eq!(downloaded_content, content);
    assert_eq!(download_info.name, "integration.txt");

    // Test file copy
    let copy_result = service
        .copy_file("/test/integration.txt", "/test/integration_copy.txt", Default::default())
        .await
        .unwrap();
    
    assert!(copy_result.success);
    assert!(service.exists("/test/integration_copy.txt").await.unwrap());

    // Test file move
    let move_result = service
        .move_file("/test/integration_copy.txt", "/test/integration_moved.txt", Default::default())
        .await
        .unwrap();
    
    assert!(move_result.success);
    assert!(!service.exists("/test/integration_copy.txt").await.unwrap());
    assert!(service.exists("/test/integration_moved.txt").await.unwrap());

    // Test directory creation
    let dir_result = service.create_directory("/test/subdir").await.unwrap();
    assert!(dir_result.success);

    // Test directory listing
    let listing = service
        .list_directory("/test", Default::default())
        .await
        .unwrap();
    
    assert_eq!(listing.path, "/test");
    assert!(listing.entries.len() >= 3); // integration.txt, integration_moved.txt, subdir

    let filenames: Vec<_> = listing.entries.iter().map(|e| &e.name).collect();
    assert!(filenames.contains(&&"integration.txt".to_string()));
    assert!(filenames.contains(&&"integration_moved.txt".to_string()));
    assert!(filenames.contains(&&"subdir".to_string()));

    // Test file deletion
    let delete_result = service.delete_file("/test/integration_moved.txt").await.unwrap();
    assert!(delete_result.success);
    assert!(!service.exists("/test/integration_moved.txt").await.unwrap());

    // Test directory deletion
    let dir_delete_result = service.delete_directory("/test/subdir", false).await.unwrap();
    assert!(dir_delete_result.success);
    assert!(!service.exists("/test/subdir").await.unwrap());

    // Test recursive directory deletion
    let recursive_delete_result = service.delete_directory("/test", true).await.unwrap();
    assert!(recursive_delete_result.success);
    assert!(!service.exists("/test").await.unwrap());
}

#[tokio::test]
async fn test_multipart_upload_workflow() {
    let (service, _temp_dir) = create_local_service().await;

    // Initiate multipart upload
    let upload = service
        .initiate_multipart_upload("/large_file.bin", UploadOptions::default())
        .await
        .unwrap();
    
    assert!(!upload.upload_id.is_empty());
    assert_eq!(upload.path, "/large_file.bin");

    // Upload parts
    let part1_content = Bytes::from("Part 1 of the large file");
    let part1 = service
        .upload_part(&upload.upload_id, 1, part1_content.clone())
        .await
        .unwrap();
    
    assert_eq!(part1.part_number, 1);
    assert_eq!(part1.size, part1_content.len() as u64);

    let part2_content = Bytes::from("Part 2 of the large file");
    let part2 = service
        .upload_part(&upload.upload_id, 2, part2_content.clone())
        .await
        .unwrap();
    
    assert_eq!(part2.part_number, 2);
    assert_eq!(part2.size, part2_content.len() as u64);

    // Complete multipart upload
    let complete_result = service
        .complete_multipart_upload(&upload.upload_id, vec![part1, part2])
        .await
        .unwrap();
    
    assert!(complete_result.success);
    assert_eq!(complete_result.path, "/large_file.bin");

    // Verify the file was created correctly
    assert!(service.exists("/large_file.bin").await.unwrap());

    let (content, _) = service
        .download_bytes("/large_file.bin", DownloadOptions::default())
        .await
        .unwrap();
    
    let expected_content = format!("{}{}", 
        String::from_utf8(part1_content.to_vec()).unwrap(),
        String::from_utf8(part2_content.to_vec()).unwrap()
    );
    assert_eq!(String::from_utf8(content.to_vec()).unwrap(), expected_content);
}

#[tokio::test]
async fn test_multipart_upload_abort() {
    let (service, _temp_dir) = create_local_service().await;

    // Initiate multipart upload
    let upload = service
        .initiate_multipart_upload("/aborted_file.bin", UploadOptions::default())
        .await
        .unwrap();

    // Upload a part
    let part_content = Bytes::from("This upload will be aborted");
    service
        .upload_part(&upload.upload_id, 1, part_content)
        .await
        .unwrap();

    // Abort the upload
    let abort_result = service
        .abort_multipart_upload(&upload.upload_id)
        .await
        .unwrap();
    
    assert!(abort_result.success);

    // Verify the file was not created
    assert!(!service.exists("/aborted_file.bin").await.unwrap());
}

#[tokio::test]
async fn test_streaming_upload_download() {
    let (service, _temp_dir) = create_local_service().await;

    // Create a stream of data
    let data_chunks = vec![
        Bytes::from("Chunk 1\n"),
        Bytes::from("Chunk 2\n"),
        Bytes::from("Chunk 3\n"),
    ];
    
    let stream = futures::stream::iter(
        data_chunks.clone().into_iter().map(Ok::<Bytes, std::io::Error>)
    );
    let file_stream = Box::pin(stream);

    // Upload stream
    let upload_result = service
        .upload_stream("/streamed_file.txt", file_stream, None, UploadOptions::default())
        .await
        .unwrap();
    
    assert!(upload_result.success);

    // Download as stream
    let (mut download_stream, file_info) = service
        .download_stream("/streamed_file.txt", DownloadOptions::default())
        .await
        .unwrap();

    // Collect stream data
    use futures::StreamExt;
    let mut downloaded_chunks = Vec::new();
    while let Some(chunk_result) = download_stream.next().await {
        let chunk = chunk_result.unwrap();
        downloaded_chunks.push(chunk);
    }

    // Verify content
    let downloaded_content = downloaded_chunks
        .into_iter()
        .fold(Bytes::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            acc
        });

    let expected_content = data_chunks
        .into_iter()
        .fold(Bytes::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            acc
        });

    assert_eq!(downloaded_content, expected_content);
    assert_eq!(file_info.name, "streamed_file.txt");
}

#[tokio::test]
async fn test_range_download() {
    let (service, _temp_dir) = create_local_service().await;

    // Upload a file
    let content = Bytes::from("0123456789abcdefghij");
    service
        .upload_bytes("/range_test.txt", content.clone(), UploadOptions::default())
        .await
        .unwrap();

    // Test range download
    let range_options = DownloadOptions::default().with_range(5, Some(9));
    let (range_content, _) = service
        .download_bytes("/range_test.txt", range_options)
        .await
        .unwrap();

    assert_eq!(range_content, Bytes::from("56789"));

    // Test open-ended range
    let range_options = DownloadOptions::default().with_range(15, None);
    let (range_content, _) = service
        .download_bytes("/range_test.txt", range_options)
        .await
        .unwrap();

    assert_eq!(range_content, Bytes::from("fghij"));
}

#[tokio::test]
async fn test_storage_usage() {
    let (service, _temp_dir) = create_local_service().await;

    // Upload some files
    let content1 = Bytes::from("File 1 content");
    let content2 = Bytes::from("File 2 content with more data");

    service
        .upload_bytes("/usage_test1.txt", content1.clone(), UploadOptions::default())
        .await
        .unwrap();

    service
        .upload_bytes("/usage_test2.txt", content2.clone(), UploadOptions::default())
        .await
        .unwrap();

    // Get storage usage
    let usage = service.get_storage_usage(None).await.unwrap();
    
    assert_eq!(usage.file_count, 2);
    assert_eq!(usage.used_bytes, (content1.len() + content2.len()) as u64);
}

#[tokio::test]
async fn test_search_functionality() {
    let (service, _temp_dir) = create_local_service().await;

    // Upload test files
    let content = Bytes::from("test content");
    service.upload_bytes("/search/doc1.txt", content.clone(), UploadOptions::default()).await.unwrap();
    service.upload_bytes("/search/doc2.txt", content.clone(), UploadOptions::default()).await.unwrap();
    service.upload_bytes("/search/image.jpg", content.clone(), UploadOptions::default()).await.unwrap();
    service.upload_bytes("/search/data.log", content, UploadOptions::default()).await.unwrap();

    // Search for .txt files
    let criteria = crate::traits::file_service::SearchCriteria {
        path: "/search".to_string(),
        recursive: false,
        name_pattern: Some("*.txt".to_string()),
        content_type_pattern: None,
        min_size: None,
        max_size: None,
        modified_after: None,
        modified_before: None,
        metadata_filters: std::collections::HashMap::new(),
        limit: None,
    };

    let results = service.search_files(criteria).await.unwrap();
    assert_eq!(results.len(), 2);

    let names: Vec<_> = results.iter().map(|f| &f.name).collect();
    assert!(names.contains(&&"doc1.txt".to_string()));
    assert!(names.contains(&&"doc2.txt".to_string()));
}

#[tokio::test]
async fn test_error_handling() {
    let (service, _temp_dir) = create_local_service().await;

    // Test file not found
    let result = service.get_file_info("/nonexistent.txt").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), df_files::models::error::FileServiceError::FileNotFound { .. }));

    // Test directory not found
    let result = service.list_directory("/nonexistent_dir", Default::default()).await;
    assert!(result.is_err());

    // Test invalid path
    let result = service.upload_bytes("", Bytes::from("test"), UploadOptions::default()).await;
    assert!(result.is_err());

    // Test overwrite protection
    let content = Bytes::from("original content");
    service.upload_bytes("/overwrite_test.txt", content.clone(), UploadOptions::default()).await.unwrap();

    let result = service.upload_bytes("/overwrite_test.txt", content, UploadOptions::default()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), df_files::models::error::FileServiceError::FileAlreadyExists { .. }));
}

#[tokio::test]
async fn test_concurrent_operations() {
    let (service, _temp_dir) = create_local_service().await;

    // Spawn multiple concurrent upload operations
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let content = Bytes::from(format!("Content for file {}", i));
            let path = format!("/concurrent/file_{}.txt", i);
            service_clone.upload_bytes(&path, content, UploadOptions::default()).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all files were created
    let listing = service.list_directory("/concurrent", Default::default()).await.unwrap();
    assert_eq!(listing.entries.len(), 10);

    // Test concurrent reads
    let mut read_handles = Vec::new();
    
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let path = format!("/concurrent/file_{}.txt", i);
            service_clone.download_bytes(&path, DownloadOptions::default()).await
        });
        read_handles.push(handle);
    }

    // Wait for all read operations to complete
    for handle in read_handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}