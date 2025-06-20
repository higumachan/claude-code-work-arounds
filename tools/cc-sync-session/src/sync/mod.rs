use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::collections::VecDeque;
use log::{info, warn};

use crate::filesystem::{FileSystem, FileSystemError, Result};

#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub verbose: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            verbose: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct SyncResult {
    pub files_copied: usize,
    pub files_skipped: usize,
    pub directories_created: usize,
    pub errors: Vec<String>,
}

pub struct SessionSyncer<FS: FileSystem> {
    filesystem: FS,
}

impl<FS: FileSystem> SessionSyncer<FS> {
    pub fn new(filesystem: FS) -> Self {
        Self { filesystem }
    }
    
    pub fn sync(
        &self,
        source_root_dir: &Path,
        source_prefix: &str,
        target_dir: &Path,
        options: &SyncOptions,
    ) -> Result<SyncResult> {
        let mut result = SyncResult::default();
        
        // Ensure target directory exists
        if !self.filesystem.exists(target_dir)? {
            if !options.dry_run {
                self.filesystem.create_directory(target_dir)?;
            }
            result.directories_created += 1;
            if options.verbose {
                info!("Created directory: {}", target_dir.display());
            }
        }
        
        // Find directories that match the source prefix
        let mut dirs_to_process = VecDeque::new();
        
        for current_dir in self.filesystem.list_directory(source_root_dir)? {
            log::debug!("Processing directory: {}", current_dir.path.display());

            if !current_dir.path.file_name().unwrap().to_string_lossy().starts_with(source_prefix){
                // Skip directories that don't match the prefix
                log::debug!("Skipping directory {}: does not match prefix {}", current_dir.path.display(), source_prefix);
                continue;
            }

            if !current_dir.is_directory {
                log::warn!("Skipping non-directory entry: {}", current_dir.path.display());
                continue;
            }
            
            // Add this directory to be processed
            dirs_to_process.push_back(current_dir.path.clone());
        }
        
        // Process all directories recursively
        while let Some(current_dir) = dirs_to_process.pop_front() {
            log::debug!("Processing directory contents: {}", current_dir.display());
            
            // List directory contents
            let entries = match self.filesystem.list_directory(&current_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    if options.verbose {
                        warn!("Failed to list directory {}: {}", current_dir.display(), e);
                    }
                    continue;
                }
            };

            for entry in entries {
                log::debug!("Processing entry: {:?}", entry.path);
                let source_path = &entry.path;
                let relative_path = source_path.strip_prefix(source_root_dir)
                    .map_err(|e| FileSystemError::PathError(e.to_string()))?;
                
                // Convert directory name format
                let converted_path = self.convert_directory_name(relative_path)?;
                log::debug!("Converted path: {}", converted_path.display());
                let target_path = target_dir.join(&converted_path);
                
                if entry.is_directory {
                    // Handle directory
                    if !self.filesystem.exists(&target_path)? {
                        if !options.dry_run {
                            self.filesystem.create_directory(&target_path)?;
                        }
                        result.directories_created += 1;
                        if options.verbose {
                            info!("Created directory: {}", target_path.display());
                        }
                    }
                    // Add subdirectory to process queue
                    dirs_to_process.push_back(entry.path.clone());
                } else {
                    // Handle file
                    match self.should_copy_file(source_path, &target_path) {
                        Ok(true) => {
                            // Check if parent directory needs to be created
                            if let Some(parent) = target_path.parent() {
                                if !self.filesystem.exists(parent)? {
                                    result.directories_created += 1;
                                    if !options.dry_run {
                                        self.filesystem.create_directory(parent)?;
                                    }
                                }
                            }
                            
                            if !options.dry_run {
                                self.filesystem.copy_file(source_path, &target_path)?;
                                
                                // Update timestamp to mark as synced
                                let now = SystemTime::now();
                                if let Err(e) = self.filesystem.set_modified_time(&target_path, now) {
                                    if options.verbose {
                                        warn!("Failed to update timestamp for {}: {}", target_path.display(), e);
                                    }
                                }
                            }
                            result.files_copied += 1;
                            info!("Copied: {} -> {}", source_path.display(), target_path.display());
                        }
                        Ok(false) => {
                            result.files_skipped += 1;
                            info!("Skipped (up to date): {}", source_path.display());
                        }
                        Err(e) => {
                            result.errors.push(format!("Error checking file {}: {}", source_path.display(), e));
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    fn should_copy_file(&self, source: &Path, target: &Path) -> Result<bool> {
        if !self.filesystem.exists(target)? {
            return Ok(true);
        }
        
        let source_metadata = self.filesystem.get_metadata(source)?;
        let target_metadata = self.filesystem.get_metadata(target)?;
        
        // Copy if source is newer than target
        Ok(source_metadata.modified > target_metadata.modified)
    }
    
    fn convert_directory_name(&self, path: &Path) -> Result<PathBuf> {
        let mut result_components = Vec::new();
        let mut first_component = true;
        
        for component in path.components() {
            if let std::path::Component::Normal(os_str) = component {
                if let Some(s) = os_str.to_str() {
                    if first_component && s.starts_with('-') {
                        // This is a Claude Code style directory, convert it
                        let without_prefix = s.strip_prefix('-').unwrap();
                        let converted = without_prefix.replace('-', "/");
                        // Split the converted path and add as separate components
                        for part in converted.split('/') {
                            result_components.push(part.to_string());
                        }
                        first_component = false;
                    } else {
                        // Regular component, keep as is
                        result_components.push(s.to_string());
                    }
                } else {
                    return Err(FileSystemError::PathError(
                        "Invalid UTF-8 in path component".to_string()
                    ));
                }
            } else {
                return Err(FileSystemError::PathError(
                    "Unexpected path component type".to_string()
                ));
            }
        }
        
        Ok(result_components.iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockFileSystem;
    use std::time::Duration;
    

    
    #[test]
    fn test_should_copy_file() {
        let fs = MockFileSystem::new();
        let syncer = SessionSyncer::new(fs.clone());
        
        let source_path = Path::new("/source/file.txt");
        let target_path = Path::new("/target/file.txt");
        
        // Test when target doesn't exist
        fs.add_file(source_path, vec![1, 2, 3], SystemTime::now());
        assert!(syncer.should_copy_file(source_path, target_path).unwrap());
        
        // Test when source is newer
        let old_time = SystemTime::now() - Duration::from_secs(3600);
        fs.add_file(target_path, vec![1, 2, 3], old_time);
        assert!(syncer.should_copy_file(source_path, target_path).unwrap());
        
        // Test when target is newer
        let new_time = SystemTime::now() + Duration::from_secs(3600);
        fs.set_modified_time(target_path, new_time).unwrap();
        assert!(!syncer.should_copy_file(source_path, target_path).unwrap());
    }
}