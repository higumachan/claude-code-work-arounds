use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::filesystem::{EntryMetadata, FileSystem, FileSystemError, Result};

#[derive(Debug, Clone)]
struct MockFile {
    content: Vec<u8>,
    modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct MockFileSystem {
    files: Arc<Mutex<HashMap<PathBuf, MockFile>>>,
    directories: Arc<Mutex<Vec<PathBuf>>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            directories: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn add_file(&self, path: impl Into<PathBuf>, content: Vec<u8>, modified: SystemTime) {
        let path = path.into();
        let mut files = self.files.lock().unwrap();
        files.insert(path, MockFile { content, modified });
    }
    
    pub fn add_directory(&self, path: impl Into<PathBuf>) {
        let path = path.into();
        let mut directories = self.directories.lock().unwrap();
        if !directories.contains(&path) {
            directories.push(path);
        }
    }
    
    pub fn get_file_content(&self, path: &Path) -> Option<Vec<u8>> {
        let files = self.files.lock().unwrap();
        files.get(path).map(|f| f.content.clone())
    }
    
    pub fn list_all_files(&self) -> Vec<PathBuf> {
        let files = self.files.lock().unwrap();
        files.keys().cloned().collect()
    }
}

impl FileSystem for MockFileSystem {
    fn list_directory(&self, path: &Path) -> Result<Vec<EntryMetadata>> {
        let files = self.files.lock().unwrap();
        let directories = self.directories.lock().unwrap();
        
        let mut results = Vec::new();
        
        // Check if the directory exists
        if path != Path::new("") && !directories.contains(&path.to_path_buf()) {
            return Err(FileSystemError::NotFound(path.to_path_buf()));
        }
        
        // List files in the directory
        for (file_path, file) in files.iter() {
            if let Some(parent) = file_path.parent() {
                if parent == path {
                    results.push(EntryMetadata {
                        path: file_path.clone(),
                        modified: file.modified,
                        is_directory: false,
                    });
                }
            }
        }
        
        // List subdirectories
        for dir_path in directories.iter() {
            if let Some(parent) = dir_path.parent() {
                if parent == path && dir_path != path {
                    results.push(EntryMetadata {
                        path: dir_path.clone(),
                        modified: SystemTime::now(),
                        is_directory: true,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    fn get_metadata(&self, path: &Path) -> Result<EntryMetadata> {
        let files = self.files.lock().unwrap();
        let directories = self.directories.lock().unwrap();
        
        if let Some(file) = files.get(path) {
            Ok(EntryMetadata {
                path: path.to_path_buf(),
                modified: file.modified,
                is_directory: false,
            })
        } else if directories.contains(&path.to_path_buf()) {
            Ok(EntryMetadata {
                path: path.to_path_buf(),
                modified: SystemTime::now(),
                is_directory: true,
            })
        } else {
            Err(FileSystemError::NotFound(path.to_path_buf()))
        }
    }
    
    fn copy_file(&self, from: &Path, to: &Path) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        
        let source_file = files.get(from)
            .ok_or_else(|| FileSystemError::NotFound(from.to_path_buf()))?
            .clone();
        
        files.insert(to.to_path_buf(), source_file);
        Ok(())
    }
    
    fn create_directory(&self, path: &Path) -> Result<()> {
        self.add_directory(path);
        Ok(())
    }
    
    fn exists(&self, path: &Path) -> Result<bool> {
        let files = self.files.lock().unwrap();
        let directories = self.directories.lock().unwrap();
        
        Ok(files.contains_key(path) || directories.contains(&path.to_path_buf()))
    }
    
    fn set_modified_time(&self, path: &Path, time: SystemTime) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        
        if let Some(file) = files.get_mut(path) {
            file.modified = time;
            Ok(())
        } else {
            Err(FileSystemError::NotFound(path.to_path_buf()))
        }
    }
}