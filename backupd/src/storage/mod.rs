pub mod sqlite_db;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek};
use std::path::PathBuf;

use backuplib::rpc::FileMetadata;

pub trait FileLen {
    fn len(&self) -> Result<u64, String>;
}

impl FileLen for File {
    fn len(&self) -> Result<u64, String> {
        self.metadata()
            .map_err(|e| e.to_string())
            .map(|m| m.len())
    }
}

pub trait StorageManager<'a> {
    fn create(&'a self, metadata: &FileMetadata) -> Result<(), String>;
    fn append(&'a self, filename: &str, data: &[u8]) -> Result<(), String>;
    fn storage_outdated(&'a self, metadata: &FileMetadata) -> Result<bool, String>;
    fn get_head(&'a self, filename: &str) -> Result<u64, String>;
}

#[derive(Debug, Clone)]
pub struct FileSystem {
    base_path: PathBuf,
}

impl FileSystem {
    pub fn new<P>(base_path: P) -> FileSystem
        where P: Into<PathBuf>,
    {
        FileSystem {
            base_path: base_path.into(),
        }
    }
}

impl<'a> StorageManager<'a> for FileSystem {
    fn create(&self, metadata: &FileMetadata) -> Result<(), String> {
        let full_path = self.base_path.join(&metadata.file_name);
        File::create(full_path).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn append(&'a self, filename: &str, data: &[u8]) -> Result<(), String> {
        let full_path = self.base_path.join(&filename);
        let mut file = OpenOptions::new()
            .write(true)
            .open(full_path)
            .map_err(|e| e.to_string())?;
        file.write_all(data)
            .map_err(|e| e.to_string())
    }

    fn storage_outdated(&'a self, metadata: &FileMetadata) -> Result<bool, String> {
        // Dummy implementation
        Ok(true)
    }

    fn get_head(&'a self, filename: &str) -> Result<u64, String> {
        unimplemented!()
    }
}

