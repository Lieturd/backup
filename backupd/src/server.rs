use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::io::{Seek, Write, SeekFrom};
use std::collections::HashMap;

use backuplib::rpc::*;
use backuplib::storage::{StorageManager, FileLen, FileSystem};

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

impl BaacupImpl<FileSystem> {
    pub fn new<P>(path: P) -> BaacupImpl<FileSystem>
        where P: Into<PathBuf>,
    {
        BaacupImpl {
            next_token_mutex: Arc::new(Mutex::new(0)),
            token_map_mutex: Arc::new(Mutex::new(HashMap::new())),
            storage: FileSystem::new(path),
        }
    }
}

impl<'a, S> Baacup for BaacupImpl<S>
    where S: StorageManager<'a>
{
    fn init_upload(&self, metadata: FileMetadata) -> Result<u32, String> {
        let _file = self.storage.create_storage(metadata.file_name.clone())
            .map_err(|e| e.to_string())?;

        // Get a token and increment token counter
        // (Bad for security)
        let mut next_token = self.next_token_mutex.lock().unwrap();
        let token = *next_token;
        *next_token += 1;

        // Insert token into map
        let context = Context::new(metadata);
        let mut token_map = self.token_map_mutex.lock().unwrap();
        token_map.insert(token, context);

        Ok(token)
    }

    fn get_head(&self, token: u32) -> Result<u64, String> {
        // Get path from map
        let token_map = self.token_map_mutex.lock().unwrap();
        let context = token_map.get(&token)
            .ok_or("Invalid token".to_string())?;

        // Get file length
        let len = self.storage.open_storage(context.file_metadata.file_name.clone())?.len()?;
        Ok(len)
    }

    fn upload_chunk(&self, chunk: FileChunk) -> Result<u32, String> {
        println!("Got chunk with token {} offset {} data.len() {}", chunk.token, chunk.offset, chunk.data.len());

        // Get file
        let mut token_map = self.token_map_mutex.lock().unwrap();
        let context = token_map.get(&chunk.token)
            .ok_or("Invalid token".to_string())?;
        let mut file = self.storage.open_storage(context.file_metadata.file_name.clone())?;

        // Double-check len
        let file_len = file.len()?;
        if file_len != chunk.offset {
            return Err("Bad offset".to_string());
        }

        // Write data
        file.seek(SeekFrom::Start(chunk.offset))
            .map_err(|e| e.to_string())?;
        file.write_all(&chunk.data)
            .map_err(|e| e.to_string())?;

        // Check if we're done
        println!("{} {}", chunk.offset + chunk.data.len() as u64, context.file_metadata.file_size);
        if chunk.offset + chunk.data.len() as u64 == context.file_metadata.file_size {
            println!("File upload finished.");
            token_map.remove(&chunk.token);
        }

        // Return checksum
        // TODO: Actually return checksum
        Ok(0)
    }

    fn file_is_uploaded(&self, _metadata: FileMetadata) -> Result<bool, String> {
        unimplemented!()
    }
}
