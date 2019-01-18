use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io::{Seek, Write, SeekFrom};
use std::collections::HashMap;

use grpc::{RequestOptions, SingleResponse};

use crate::proto::baacup_grpc::*;
use crate::proto::baacup::*;

struct Metadata {
    file_metadata: FileMetadata,
    full_path: PathBuf,
}

impl Metadata {
    fn new(file_metadata: FileMetadata, full_path: PathBuf) -> Metadata {
        Metadata {
            file_metadata: file_metadata,
            full_path: full_path,
        }
    }
}

pub struct BaacupImpl {
    base_path: PathBuf,
    next_token_mutex: Arc<Mutex<u32>>,
    token_map_mutex: Arc<Mutex<HashMap<u32, Metadata>>>,
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

    fn init_upload_internal(&self, p: FileMetadata) -> Result<u32, String> {
        // Make file
        // Possibly isn't needed
        let filename = p.get_filename();
        let full_path = self.base_path.join(filename);
        println!("Got filename {}", filename);
        println!("full path: {:?}", full_path);
        let _file = File::create(&full_path)
            .map_err(|e| e.to_string())?;

        // Get a token and increment token counter
        // (Bad for security)
        let mut next_token = self.next_token_mutex.lock().unwrap();
        let token = *next_token;
        *next_token += 1;

        // Insert token into map
        let metadata = Metadata::new(p, full_path);
        let mut token_map = self.token_map_mutex.lock().unwrap();
        token_map.insert(token, metadata);

        Ok(token)
    }

    fn get_head_internal(&self, p: UploadToken) -> Result<u64, String> {
        // Get token
        let token = p.get_token();

        // Get path from map
        let token_map = self.token_map_mutex.lock().unwrap();
        let metadata = token_map.get(&token)
            .ok_or("Invalid token".to_string())?;
        let full_path = &metadata.full_path;

        // Get file length
        let len = File::open(full_path).unwrap().metadata().unwrap().len();
        Ok(len)
    }

    fn upload_chunk_internal(&self, p: FileChunk) -> Result<u32, String> {
        // Get data
        let token = p.get_token();
        let offset = p.get_offset();
        let data = p.get_data();

        println!("Got chunk with token {} offset {} data.len() {}", token, offset, data.len());

        // Get file
        let mut token_map = self.token_map_mutex.lock().unwrap();
        let metadata = token_map.get(&token)
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
        if file_len != offset {
            return Err("Bad offset".to_string());
        }

        // Write data
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        file.write_all(&data)
            .map_err(|e| e.to_string())?;

        // Check if we're done
        println!("{} {}", offset + data.len() as u64, metadata.file_metadata.get_file_size());
        if offset + data.len() as u64 == metadata.file_metadata.get_file_size() {
            println!("File upload finished.");
            token_map.remove(&token);
        }

        // Return checksum
        // TODO: Actually return checksum
        Ok(0)
    }
}

impl Baacup for BaacupImpl {
    fn init_upload(&self, _o: RequestOptions, p: FileMetadata) -> SingleResponse<InitUploadResponse> {
        match self.init_upload_internal(p) {
            Ok(token) => {
                let mut init_upload_response = InitUploadResponse::new();
                init_upload_response.set_status(Status::SUCCESS);
                init_upload_response.mut_token().set_token(token);
                SingleResponse::completed(init_upload_response)
            }
            Err(error) => {
                println!("Error: {}", error);
                let mut init_upload_response = InitUploadResponse::new();
                init_upload_response.set_status(Status::ERROR);
                init_upload_response.set_error_message(error);
                SingleResponse::completed(init_upload_response)
            }
        }
    }

    fn get_head(&self, _o: RequestOptions, p: UploadToken) -> SingleResponse<FileHead> {
        match self.get_head_internal(p) {
            Ok(offset) => {
                let mut file_head = FileHead::new();
                file_head.set_status(Status::SUCCESS);
                file_head.set_offset(offset);
                SingleResponse::completed(file_head)
            }
            Err(error) => {
                println!("Error: {}", error);
                let mut file_head = FileHead::new();
                file_head.set_status(Status::ERROR);
                file_head.set_error_message(error);
                SingleResponse::completed(file_head)
            }
        }
    }

    fn upload_chunk(&self, _o: RequestOptions, p: FileChunk) -> SingleResponse<UploadFileResponse> {
        match self.upload_chunk_internal(p) {
            Ok(checksum) => {
                let mut upload_file_response = UploadFileResponse::new();
                upload_file_response.set_status(Status::SUCCESS);
                upload_file_response.set_checksum(checksum);
                SingleResponse::completed(upload_file_response)
            }
            Err(error) => {
                println!("Error: {}", error);
                let mut upload_file_response = UploadFileResponse::new();
                upload_file_response.set_status(Status::ERROR);
                upload_file_response.set_error_message(error);
                SingleResponse::completed(upload_file_response)
            }
        }
    }

    fn file_is_uploaded(&self, _o: RequestOptions, _p: FileMetadata) -> SingleResponse<FileIsUploadedResponse> {
        unimplemented!()
    }
}
