use cc_sync_session::{RealFileSystem, SessionSyncer, SyncOptions};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

fn create_test_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn set_file_time(path: &Path, time: SystemTime) {
    let file_time = filetime::FileTime::from(time);
    filetime::set_file_mtime(path, file_time).unwrap();
}

#[test]
fn test_real_filesystem_sync() {
    let source_temp = TempDir::new().unwrap();
    let target_temp = TempDir::new().unwrap();
    
    let source_dir = source_temp.path();
    let target_dir = target_temp.path();
    
    // Create source directory structure mimicking Claude Code sessions
    let session_dir = source_dir.join("-Users-test-project");
    let file1 = session_dir.join("session.json");
    let file2 = session_dir.join("conversations").join("conv1.json");
    
    create_test_file(&file1, b"{'session': 'data'}");
    create_test_file(&file2, b"{'conversation': 'data'}");
    
    // Run sync
    let filesystem = RealFileSystem::new();
    let syncer = SessionSyncer::new(filesystem);
    let options = SyncOptions::default();
    
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify results
    assert_eq!(result.files_copied, 2);
    assert_eq!(result.files_skipped, 0);
    assert_eq!(result.errors.len(), 0);
    
    // Verify files exist at correct locations
    let expected_file1 = target_dir.join("Users/test/project").join("session.json");
    let expected_file2 = target_dir.join("Users/test/project").join("conversations").join("conv1.json");
    
    assert!(expected_file1.exists());
    assert!(expected_file2.exists());
    
    // Verify content
    assert_eq!(fs::read(&expected_file1).unwrap(), b"{'session': 'data'}");
    assert_eq!(fs::read(&expected_file2).unwrap(), b"{'conversation': 'data'}");
}

#[test]
fn test_incremental_sync() {
    let source_temp = TempDir::new().unwrap();
    let target_temp = TempDir::new().unwrap();
    
    let source_dir = source_temp.path();
    let target_dir = target_temp.path();
    
    // Create initial files
    let session_dir = source_dir.join("-Users-test-incremental");
    let file1 = session_dir.join("file1.txt");
    let file2 = session_dir.join("file2.txt");
    
    create_test_file(&file1, b"content1");
    create_test_file(&file2, b"content2");
    
    // First sync
    let filesystem = RealFileSystem::new();
    let syncer = SessionSyncer::new(filesystem);
    let options = SyncOptions::default();
    
    let result1 = syncer.sync(source_dir, target_dir, &options).unwrap();
    assert_eq!(result1.files_copied, 2);
    
    // Run sync again without changes
    let result2 = syncer.sync(source_dir, target_dir, &options).unwrap();
    assert_eq!(result2.files_copied, 0);
    assert_eq!(result2.files_skipped, 2);
    
    // Update one file
    let old_time = SystemTime::now() - Duration::from_secs(3600);
    set_file_time(&target_dir.join("Users/test/incremental").join("file1.txt"), old_time);
    
    // Sync again
    let result3 = syncer.sync(source_dir, target_dir, &options).unwrap();
    assert_eq!(result3.files_copied, 1);
    assert_eq!(result3.files_skipped, 1);
}

#[test]
fn test_dry_run_mode() {
    let source_temp = TempDir::new().unwrap();
    let target_temp = TempDir::new().unwrap();
    
    let source_dir = source_temp.path();
    let target_dir = target_temp.path();
    
    // Create source files
    let session_dir = source_dir.join("-Users-test-dryrun");
    let file = session_dir.join("test.txt");
    create_test_file(&file, b"test content");
    
    // Run sync in dry-run mode
    let filesystem = RealFileSystem::new();
    let syncer = SessionSyncer::new(filesystem);
    let options = SyncOptions {
        dry_run: true,
        verbose: false,
    };
    
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify counts
    assert_eq!(result.files_copied, 1);
    assert_eq!(result.directories_created, 1);
    
    // Verify no actual files were created
    let expected_file = target_dir.join("Users/test/dryrun").join("test.txt");
    assert!(!expected_file.exists());
}

#[test]
fn test_github_com_path_conversion() {
    let source_temp = TempDir::new().unwrap();
    let target_temp = TempDir::new().unwrap();
    
    let source_dir = source_temp.path();
    let target_dir = target_temp.path();
    
    // Create source with github.com in path
    let session_dir = source_dir.join("-Users-dev-github.com-myrepo");
    let file = session_dir.join("README.md");
    create_test_file(&file, b"# My Repo");
    
    // Run sync
    let filesystem = RealFileSystem::new();
    let syncer = SessionSyncer::new(filesystem);
    let options = SyncOptions::default();
    
    let result = syncer.sync(source_dir, target_dir, &options).unwrap();
    
    // Verify correct path conversion
    let expected_file = target_dir.join("Users/dev/github.com/myrepo").join("README.md");
    assert!(expected_file.exists());
    assert_eq!(result.files_copied, 1);
}