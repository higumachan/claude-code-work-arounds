mod real;

pub use real::RealFileSystem;

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Path error: {0}")]
    PathError(String),
    
    #[error("Not found: {0}")]
    NotFound(PathBuf),
}

pub type Result<T> = std::result::Result<T, FileSystemError>;

#[derive(Debug, Clone)]
pub struct EntryMetadata {
    pub path: PathBuf,
    pub modified: SystemTime,
    pub is_directory: bool,
}

pub trait FileSystem: Send + Sync {
    fn list_directory(&self, path: &Path) -> Result<Vec<EntryMetadata>>;
    
    fn get_metadata(&self, path: &Path) -> Result<EntryMetadata>;
    
    fn copy_file(&self, from: &Path, to: &Path) -> Result<()>;
    
    fn create_directory(&self, path: &Path) -> Result<()>;
    
    fn exists(&self, path: &Path) -> Result<bool>;
    
    fn set_modified_time(&self, path: &Path, time: SystemTime) -> Result<()>;
}