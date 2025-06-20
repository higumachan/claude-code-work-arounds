use std::fs;
use tempfile::TempDir;

#[test]
fn test_sync_with_new_files() -> anyhow::Result<()> {
    // Create temporary directories for testing
    let source_temp = TempDir::new()?;
    let target_temp = TempDir::new()?;

    // Create a mock session directory
    let session_dir = source_temp.path().join("-Users-test-project");
    fs::create_dir_all(&session_dir)?;

    // Create some test files in the session
    let test_file1 = session_dir.join("file1.txt");
    let test_file2 = session_dir.join("subdir").join("file2.txt");
    
    fs::create_dir_all(session_dir.join("subdir"))?;
    fs::write(&test_file1, "Test content 1")?;
    fs::write(&test_file2, "Test content 2")?;

    // Initialize the target repository
    std::fs::create_dir_all(target_temp.path().join(".git"))?;
    std::fs::create_dir_all(target_temp.path().join(".claude/ccss_sessions"))?;
    std::fs::write(target_temp.path().join(".claude/ccss_sessions/.gitkeep"), "")?;
    
    // Run the sync command
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cc-sync-session"))
        .args(&[
            "sync",
            "--source-dir",
            source_temp.path().to_str().unwrap(),
            "--repo-dir",
            target_temp.path().to_str().unwrap(),
        ])
        .output()?;

    // Verify the sync was successful
    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Check that files were copied to the correct location
    let expected_dir = target_temp.path().join(".claude/ccss_sessions/Users/test/project");
    assert!(expected_dir.exists());
    assert!(expected_dir.join("file1.txt").exists());
    assert!(expected_dir.join("subdir/file2.txt").exists());

    // Verify file contents
    let content1 = fs::read_to_string(expected_dir.join("file1.txt"))?;
    assert_eq!(content1, "Test content 1");

    let content2 = fs::read_to_string(expected_dir.join("subdir/file2.txt"))?;
    assert_eq!(content2, "Test content 2");

    Ok(())
}

#[test]
fn test_sync_with_dry_run() -> anyhow::Result<()> {
    // Create temporary directories for testing
    let source_temp = TempDir::new()?;
    let target_temp = TempDir::new()?;

    // Create a mock session directory
    let session_dir = source_temp.path().join("-Users-test-dryrun");
    fs::create_dir_all(&session_dir)?;

    // Create a test file
    let test_file = session_dir.join("test.txt");
    fs::write(&test_file, "Should not be copied")?;

    // Initialize the target repository
    std::fs::create_dir_all(target_temp.path().join(".git"))?;
    std::fs::create_dir_all(target_temp.path().join(".claude/ccss_sessions"))?;
    std::fs::write(target_temp.path().join(".claude/ccss_sessions/.gitkeep"), "")?;
    
    // Run the sync command with --dry-run
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cc-sync-session"))
        .args(&[
            "sync",
            "--source-dir",
            source_temp.path().to_str().unwrap(),
            "--repo-dir",
            target_temp.path().to_str().unwrap(),
            "--dry-run",
        ])
        .output()?;

    assert!(output.status.success());

    // Verify that no files were actually copied
    let expected_dir = target_temp.path().join(".claude/ccss_sessions/Users/test/dryrun");
    assert!(!expected_dir.exists());

    Ok(())
}

#[test]
fn test_sync_updates_newer_files() -> anyhow::Result<()> {
    // Create temporary directories for testing
    let source_temp = TempDir::new()?;
    let target_temp = TempDir::new()?;

    // Create a mock session directory
    let session_dir = source_temp.path().join("-Users-test-update");
    fs::create_dir_all(&session_dir)?;

    // Initialize the target repository
    std::fs::create_dir_all(target_temp.path().join(".git"))?;
    std::fs::create_dir_all(target_temp.path().join(".claude/ccss_sessions"))?;
    std::fs::write(target_temp.path().join(".claude/ccss_sessions/.gitkeep"), "")?;
    
    // Create target directory with an existing file
    let target_dir = target_temp.path().join(".claude/ccss_sessions/Users/test/update");
    fs::create_dir_all(&target_dir)?;

    let source_file = session_dir.join("update.txt");
    let target_file = target_dir.join("update.txt");

    // Create target file first (older)
    fs::write(&target_file, "Old content")?;
    
    // Wait a moment to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Create source file (newer)
    fs::write(&source_file, "New content")?;

    // Run the sync command
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cc-sync-session"))
        .args(&[
            "sync",
            "--source-dir",
            source_temp.path().to_str().unwrap(),
            "--repo-dir",
            target_temp.path().to_str().unwrap(),
        ])
        .output()?;

    assert!(output.status.success());

    // Verify the file was updated
    let content = fs::read_to_string(&target_file)?;
    assert_eq!(content, "New content");

    Ok(())
}

#[test]
fn test_init_command() -> anyhow::Result<()> {
    // Create temporary directory for testing
    let temp_dir = TempDir::new()?;
    
    // Create a git repository
    std::fs::create_dir_all(temp_dir.path().join(".git"))?;
    
    // Run the init command
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cc-sync-session"))
        .args(&[
            "init",
            "--repo-dir",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()?;
    
    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    // Verify the directories and files were created
    let ccss_dir = temp_dir.path().join(".claude/ccss_sessions");
    assert!(ccss_dir.exists());
    assert!(ccss_dir.join(".gitkeep").exists());
    
    Ok(())
}

#[test]
fn test_sync_auto_detect_repo() -> anyhow::Result<()> {
    // Create temporary directories for testing
    let source_temp = TempDir::new()?;
    let repo_temp = TempDir::new()?;
    
    // Create a mock session directory
    let session_dir = source_temp.path().join("-Users-test-autodetect");
    fs::create_dir_all(&session_dir)?;
    fs::write(session_dir.join("test.txt"), "Auto detect test")?;
    
    // Initialize the repository
    std::fs::create_dir_all(repo_temp.path().join(".git"))?;
    std::fs::create_dir_all(repo_temp.path().join(".claude/ccss_sessions"))?;
    std::fs::write(repo_temp.path().join(".claude/ccss_sessions/.gitkeep"), "")?;
    
    // Change to the repo directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&repo_temp)?;
    
    // Run the sync command without --repo-dir
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cc-sync-session"))
        .args(&[
            "sync",
            "--source-dir",
            source_temp.path().to_str().unwrap(),
        ])
        .output()?;
    
    // Restore original directory
    std::env::set_current_dir(original_dir)?;
    
    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    // Verify the file was synced
    let expected_file = repo_temp.path().join(".claude/ccss_sessions/Users/test/autodetect/test.txt");
    assert!(expected_file.exists());
    
    Ok(())
}