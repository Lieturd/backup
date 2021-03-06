use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::io::{Seek, Write, SeekFrom};
use std::collections::HashMap;

use backuplib::rpc::*;

use crate::storage::{StorageManager, FileLen, FileSystem};
use crate::storage::sqlite_db::SqliteStorageManager;

macro_rules! try_future {
    ($x:expr) => {
        match $x {
            Ok(val) => val,
            Err(err) => return BaacupFuture::new(Err(err)),
        }
    };
}

struct Context {
    file_metadata: FileMetadata,
}

impl Context {
    fn new(file_metadata: FileMetadata) -> Context {
        Context {
            file_metadata: file_metadata,
        }
    }
}

pub struct BaacupImpl<S> {
    next_token_mutex: Arc<Mutex<u32>>,
    token_map_mutex: Arc<Mutex<HashMap<u32, Context>>>,
    storage: S,
}

impl<S> BaacupImpl<S> {
    pub fn new_from_storage(storage_manager: S) -> BaacupImpl<S> {
        BaacupImpl {
            next_token_mutex: Arc::new(Mutex::new(0)),
            token_map_mutex: Arc::new(Mutex::new(HashMap::new())),
            storage: storage_manager,
        }
    }
}

impl BaacupImpl<FileSystem> {
    pub fn new_from_path<P>(path: P) -> BaacupImpl<FileSystem>
        where P: Into<PathBuf>,
    {
        let fs = FileSystem::new(path);
        Self::new_from_storage(fs)
    }
}

impl BaacupImpl<SqliteStorageManager> {
    pub fn new_from_db_path(path: &str) -> BaacupImpl<SqliteStorageManager> {
        let fs = SqliteStorageManager::new(path);
        Self::new_from_storage(fs)
    }
}

impl<S> Baacup for BaacupImpl<S>
    where for<'a> S: StorageManager<'a>,
{
    fn init_upload(&self, metadata: FileMetadata) -> BaacupFuture<u32> {
        try_future!(self.storage.create(&metadata)
            .map_err(|e| e.to_string()));

        // Get a token and increment token counter
        // (Bad for security)
        let mut next_token = self.next_token_mutex.lock().unwrap();
        let token = *next_token;
        *next_token += 1;

        // Insert token into map
        let context = Context::new(metadata);
        let mut token_map = self.token_map_mutex.lock().unwrap();
        token_map.insert(token, context);

        BaacupFuture::new(Ok(token))
    }

    fn get_head(&self, token: u32) -> BaacupFuture<u64> {
        // Get path from map
        let token_map = self.token_map_mutex.lock().unwrap();
        let context = try_future!(token_map.get(&token)
            .ok_or("Invalid token".to_string()));

        // Get file length
        BaacupFuture::new(self.storage
            .get_head(&context.file_metadata.file_name)
            .map_err(|e| e.to_string()))
    }

    fn upload_chunk(&self, chunk: FileChunk) -> BaacupFuture<u32> {
        println!("Got chunk with token {} offset {} data.len() {}", chunk.token, chunk.offset, chunk.data.len());

        // Get metadata
        let mut token_map = self.token_map_mutex.lock().unwrap();
        let context = try_future!(token_map.get(&chunk.token)
            .ok_or("Invalid token".to_string()));

        // Double-check len
        let file_len = try_future!(self.storage
            .get_head(&context.file_metadata.file_name)
            .map_err(|e| e.to_string()));
        if file_len != chunk.offset {
            return BaacupFuture::new(Err("Bad offset".to_string()));
        }

        // Write data
        try_future!(self.storage
            .append(&context.file_metadata.file_name, &chunk.data)
            .map_err(|e| e.to_string()));

        // Check if we're done
        println!("{} {}", chunk.offset + chunk.data.len() as u64, context.file_metadata.file_size);
        if chunk.offset + chunk.data.len() as u64 == context.file_metadata.file_size {
            println!("File upload finished.");
            token_map.remove(&chunk.token);
        }

        // Return checksum
        // TODO: Actually return checksum
        BaacupFuture::new(Ok(0))
    }

    fn file_is_uploaded(&self, metadata: FileMetadata) -> BaacupFuture<bool> {
        BaacupFuture::new(self.storage.storage_outdated(&metadata).map(|b| !b))
    }
}
