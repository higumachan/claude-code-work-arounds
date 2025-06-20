use cc_sync_session::{FileSystem, mock::MockFileSystem};
use cc_sync_session::sync::{SessionSyncer, SyncOptions};
use std::path::Path;
use std::time::{Duration, SystemTime};

#[test]
fn test_sync_new_files() {
    let fs = MockFileSystem::new();
    let syncer = SessionSyncer::new(fs.clone());
    
    // Setup source directory structure
    let source_dir = Path::new("/source");
    let target_dir = Path::new("/target");
    
    fs.add_directory(source_dir);
    fs.add_directory(target_dir);
    
    // Add files to source
    let file1 = source_dir.join("-Users-yuta-project").join("file1.txt");
    let file2 = source_dir.join("-Users-yuta-project").join("subdir").join("file2.txt");
    
    fs.add_directory(source_dir.join("-Users-yuta-project"));
    fs.add_directory(source_dir.join("-Users-yuta-project").join("subdir"));
    fs.add_file(&file1, vec![1, 2, 3], SystemTime::now());
    fs.add_file(&file2, vec![4, 5, 6], SystemTime::now());
    
    // Run sync
    let options = SyncOptions::default();
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify results
    assert_eq!(result.files_copied, 2);
    assert_eq!(result.files_skipped, 0);
    assert_eq!(result.directories_created, 2); // Users/yuta/project and Users/yuta/project/subdir
    assert_eq!(result.errors.len(), 0);
    
    // Verify files were copied correctly
    let expected_file1 = target_dir.join("Users/yuta/project").join("file1.txt");
    let expected_file2 = target_dir.join("Users/yuta/project").join("subdir").join("file2.txt");
    
    assert!(fs.exists(&expected_file1).unwrap());
    assert!(fs.exists(&expected_file2).unwrap());
    
    assert_eq!(fs.get_file_content(&expected_file1).unwrap(), vec![1, 2, 3]);
    assert_eq!(fs.get_file_content(&expected_file2).unwrap(), vec![4, 5, 6]);
}

#[test]
fn test_sync_skip_up_to_date_files() {
    let fs = MockFileSystem::new();
    let syncer = SessionSyncer::new(fs.clone());
    
    let source_dir = Path::new("/source");
    let target_dir = Path::new("/target");
    
    fs.add_directory(source_dir);
    fs.add_directory(target_dir);
    
    // Add source file
    let source_file = source_dir.join("-Users-yuta-project").join("file.txt");
    fs.add_directory(source_dir.join("-Users-yuta-project"));
    
    let old_time = SystemTime::now() - Duration::from_secs(3600);
    fs.add_file(&source_file, vec![1, 2, 3], old_time);
    
    // Add target file with newer timestamp
    let target_file = target_dir.join("Users/yuta/project").join("file.txt");
    fs.add_directory(target_dir.join("Users"));
    fs.add_directory(target_dir.join("Users/yuta"));
    fs.add_directory(target_dir.join("Users/yuta/project"));
    fs.add_file(&target_file, vec![1, 2, 3], SystemTime::now());
    
    // Run sync
    let options = SyncOptions::default();
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify file was skipped
    assert_eq!(result.files_copied, 0);
    assert_eq!(result.files_skipped, 1);
}

#[test]
fn test_sync_update_newer_files() {
    let fs = MockFileSystem::new();
    let syncer = SessionSyncer::new(fs.clone());
    
    let source_dir = Path::new("/source");
    let target_dir = Path::new("/target");
    
    fs.add_directory(source_dir);
    fs.add_directory(target_dir);
    
    // Add source file with newer timestamp
    let source_file = source_dir.join("-Users-yuta-project").join("file.txt");
    fs.add_directory(source_dir.join("-Users-yuta-project"));
    fs.add_file(&source_file, vec![7, 8, 9], SystemTime::now());
    
    // Add target file with older timestamp
    let target_file = target_dir.join("Users/yuta/project").join("file.txt");
    fs.add_directory(target_dir.join("Users"));
    fs.add_directory(target_dir.join("Users/yuta"));
    fs.add_directory(target_dir.join("Users/yuta/project"));
    
    let old_time = SystemTime::now() - Duration::from_secs(3600);
    fs.add_file(&target_file, vec![1, 2, 3], old_time);
    
    // Run sync
    let options = SyncOptions::default();
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify file was updated
    assert_eq!(result.files_copied, 1);
    assert_eq!(result.files_skipped, 0);
    assert_eq!(fs.get_file_content(&target_file).unwrap(), vec![7, 8, 9]);
}

#[test]
fn test_sync_dry_run() {
    let fs = MockFileSystem::new();
    let syncer = SessionSyncer::new(fs.clone());
    
    let source_dir = Path::new("/source");
    let target_dir = Path::new("/target");
    
    fs.add_directory(source_dir);
    fs.add_directory(target_dir);
    
    // Add source file
    let source_file = source_dir.join("-Users-yuta-project").join("file.txt");
    fs.add_directory(source_dir.join("-Users-yuta-project"));
    fs.add_file(&source_file, vec![1, 2, 3], SystemTime::now());
    
    // Run sync with dry run
    let options = SyncOptions {
        dry_run: true,
        verbose: false,
    };
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify counts but no actual copies
    assert_eq!(result.files_copied, 1);
    assert_eq!(result.directories_created, 1);
    
    // Verify file was NOT actually copied
    let expected_file = target_dir.join("Users/yuta/project").join("file.txt");
    assert!(!fs.exists(&expected_file).unwrap());
}

#[test]
fn test_sync_with_github_domain() {
    let fs = MockFileSystem::new();
    let syncer = SessionSyncer::new(fs.clone());
    
    let source_dir = Path::new("/source");
    let target_dir = Path::new("/target");
    
    fs.add_directory(source_dir);
    fs.add_directory(target_dir);
    
    // Add file with github.com in path
    let source_file = source_dir.join("-Users-yuta-github.com-project").join("file.txt");
    fs.add_directory(source_dir.join("-Users-yuta-github.com-project"));
    fs.add_file(&source_file, vec![1, 2, 3], SystemTime::now());
    
    // Run sync
    let options = SyncOptions::default();
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify correct path conversion
    let expected_file = target_dir.join("Users/yuta/github.com/project").join("file.txt");
    assert!(fs.exists(&expected_file).unwrap());
    assert_eq!(result.files_copied, 1);
}

#[test]
fn test_sync_nested_directories() {
    let fs = MockFileSystem::new();
    let syncer = SessionSyncer::new(fs.clone());
    
    let source_dir = Path::new("/source");
    let target_dir = Path::new("/target");
    
    fs.add_directory(source_dir);
    fs.add_directory(target_dir);
    
    // Create nested directory structure
    let base = source_dir.join("-Users-yuta-project");
    fs.add_directory(&base);
    fs.add_directory(base.join("src"));
    fs.add_directory(base.join("src").join("lib"));
    fs.add_directory(base.join("tests"));
    
    // Add files at different levels
    fs.add_file(base.join("README.md"), vec![1], SystemTime::now());
    fs.add_file(base.join("src").join("main.rs"), vec![2], SystemTime::now());
    fs.add_file(base.join("src").join("lib").join("mod.rs"), vec![3], SystemTime::now());
    fs.add_file(base.join("tests").join("test.rs"), vec![4], SystemTime::now());
    
    // Run sync
    let options = SyncOptions::default();
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify all files and directories were created
    assert_eq!(result.files_copied, 4);
    assert_eq!(result.directories_created, 4); // Users/yuta/project, src, src/lib, tests
    
    // Verify structure
    let target_base = target_dir.join("Users/yuta/project");
    assert!(fs.exists(&target_base.join("README.md")).unwrap());
    assert!(fs.exists(&target_base.join("src").join("main.rs")).unwrap());
    assert!(fs.exists(&target_base.join("src").join("lib").join("mod.rs")).unwrap());
    assert!(fs.exists(&target_base.join("tests").join("test.rs")).unwrap());
}