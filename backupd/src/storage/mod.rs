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
    type File: Read + Write + Seek + FileLen + 'a;
    fn create_storage(&'a self, metadata: &FileMetadata) -> Result<Self::File, String>;
    fn open_storage(&'a self, path: String) -> Result<Self::File, String>;
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
    type File = File;

    fn create_storage(&self, metadata: &FileMetadata) -> Result<Self::File, String> {
        let full_path = self.base_path.join(&metadata.file_name);
        File::create(full_path).map_err(|e| e.to_string())
    }

    fn open_storage(&self, path: String) -> Result<Self::File, String> {
        let full_path = self.base_path.join(path);
        OpenOptions::new()
            .write(true)
            .open(full_path)
            .map_err(|e| e.to_string())
    }
}
