use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io::{Seek, Write, SeekFrom};
use std::collections::HashMap;

use crate::rpc::*;
pub use crate::proto::baacup_grpc::BaacupServer;

struct Context {
    file_metadata: FileMetadata,
    full_path: PathBuf,
}

impl Context {
    fn new(file_metadata: FileMetadata, full_path: PathBuf) -> Context {
        Context {
            file_metadata: file_metadata,
            full_path: full_path,
        }
    }
}

pub struct BaacupImpl {
    base_path: PathBuf,
    next_token_mutex: Arc<Mutex<u32>>,
    token_map_mutex: Arc<Mutex<HashMap<u32, Context>>>,
}

impl BaacupImpl {
    pub fn new<P>(path: P) -> BaacupImpl
        where P: Into<PathBuf>,
    {
        BaacupImpl {
            base_path: path.into(),
            next_token_mutex: Arc::new(Mutex::new(0)),
            token_map_mutex: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Baacup for BaacupImpl {
    fn init_upload(&self, metadata: FileMetadata) -> Result<u32, String> {
        let full_path = self.base_path.join(&metadata.file_name);
        println!("Got filename {:?}", metadata.file_name);
        println!("full path: {:?}", full_path);
        let _file = File::create(&full_path)
            .map_err(|e| e.to_string())?;

        // Get a token and increment token counter
        // (Bad for security)
        let mut next_token = self.next_token_mutex.lock().unwrap();
        let token = *next_token;
        *next_token += 1;

        // Insert token into map
        let context = Context::new(metadata, full_path);
        let mut token_map = self.token_map_mutex.lock().unwrap();
        token_map.insert(token, context);

        Ok(token)
    }

    fn get_head(&self, token: u32) -> Result<u64, String> {
        // Get path from map
        let token_map = self.token_map_mutex.lock().unwrap();
        let context = token_map.get(&token)
            .ok_or("Invalid token".to_string())?;
        let full_path = &context.full_path;

        // Get file length
        let len = File::open(full_path).unwrap().metadata().unwrap().len();
        Ok(len)
    }

    fn upload_chunk(&self, chunk: FileChunk) -> Result<u32, String> {
        println!("Got chunk with token {} offset {} data.len() {}", chunk.token, chunk.offset, chunk.data.len());

        // Get file
        let mut token_map = self.token_map_mutex.lock().unwrap();
        let metadata = token_map.get(&chunk.token)
            .ok_or("Invalid token".to_string())?;
        let full_path = &metadata.full_path;
        let mut file = OpenOptions::new()
            .write(true)
            .open(full_path)
            .map_err(|e| e.to_string())?;

        // Double-check len
        let file_len = file.metadata()
            .map_err(|e| e.to_string())?
            .len();
        if file_len != chunk.offset {
            return Err("Bad offset".to_string());
        }

        // Write data
        file.seek(SeekFrom::Start(chunk.offset))
            .map_err(|e| e.to_string())?;
        file.write_all(&chunk.data)
            .map_err(|e| e.to_string())?;

        // Check if we're done
        println!("{} {}", chunk.offset + chunk.data.len() as u64, metadata.file_metadata.file_size);
        if chunk.offset + chunk.data.len() as u64 == metadata.file_metadata.file_size {
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
