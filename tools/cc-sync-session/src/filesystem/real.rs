use std::fs;
use std::path::Path;
use std::time::SystemTime;
use filetime::{set_file_mtime, FileTime};

use super::{EntryMetadata, FileSystem, Result};

#[derive(Debug, Clone)]
pub struct RealFileSystem;

impl RealFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for RealFileSystem {
    fn list_directory(&self, path: &Path) -> Result<Vec<EntryMetadata>> {
        let mut results = Vec::new();
        
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => return Err(e.into()),
        };
        
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip entries we can't read
            };
            
            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue, // Skip entries we can't get metadata for
            };
            
            let modified = match metadata.modified() {
                Ok(m) => m,
                Err(_) => SystemTime::now(), // Use current time as fallback
            };
            
            results.push(EntryMetadata {
                path,
                modified,
                is_directory: metadata.is_dir(),
            });
        }
        
        Ok(results)
    }
    
    fn get_metadata(&self, path: &Path) -> Result<EntryMetadata> {
        let metadata = fs::metadata(path)?;
        
        Ok(EntryMetadata {
            path: path.to_path_buf(),
            modified: metadata.modified()?,
            is_directory: metadata.is_dir(),
        })
    }
    
    fn copy_file(&self, from: &Path, to: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::copy(from, to)?;
        Ok(())
    }
    
    fn create_directory(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }
    
    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }
    
    fn set_modified_time(&self, path: &Path, time: SystemTime) -> Result<()> {
        let file_time = FileTime::from(time);
        set_file_mtime(path, file_time)?;
        Ok(())
    }
}