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
}

impl Baacup for BaacupImpl {
    fn init_upload(&self, _o: RequestOptions, p: FileMetadata) -> SingleResponse<UploadToken> {
        // Make file
        // Possibly isn't needed
        let filename = p.get_filename();
        let full_path = self.base_path.join(filename);
        println!("Got filename {}", filename);
        println!("full path: {:?}", full_path);
        let _file = File::create(&full_path).unwrap();

        // Get a token and increment token counter
        // (Bad for security)
        let mut next_token = self.next_token_mutex.lock().unwrap();
        let token = *next_token;
        *next_token += 1;

        // Insert token into map
        let metadata = Metadata::new(p, full_path);
        let mut token_map = self.token_map_mutex.lock().unwrap();
        token_map.insert(token, metadata);

        // Return token
        let mut upload_token = UploadToken::new();
        upload_token.set_token(token);
        SingleResponse::completed(upload_token)
    }

    fn get_head(&self, _o: RequestOptions, p: UploadToken) -> SingleResponse<FileHead> {
        // Get token
        let token = p.get_token();

        // Get path from map
        let token_map = self.token_map_mutex.lock().unwrap();
        let metadata = token_map.get(&token).unwrap();
        let full_path = &metadata.full_path;

        // Get file length
        let len = File::open(full_path).unwrap().metadata().unwrap().len();

        // Return offset
        let mut file_head = FileHead::new();
        file_head.set_offset(len);
        SingleResponse::completed(file_head)
    }

    fn upload_chunk(&self, _o: RequestOptions, p: FileChunk) -> SingleResponse<UploadFileResponse> {
        // Get data
        let token = p.get_token();
        let offset = p.get_offset();
        let data = p.get_data();

        println!("Got chunk with token {} offset {} data.len() {}", token, offset, data.len());

        // Get file
        let mut token_map = self.token_map_mutex.lock().unwrap();
        let metadata = token_map.get(&token).unwrap();
        let full_path = &metadata.full_path;
        let mut file = OpenOptions::new()
            .write(true)
            .open(full_path).unwrap();

        // Double-check len
        if file.metadata().unwrap().len() != offset {
            panic!("Bad offset");
        }

        // Write data
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.write_all(&data).unwrap();

        // Check if we're done
        println!("{} {}", offset + data.len() as u64, metadata.file_metadata.get_file_size());
        if offset + data.len() as u64 == metadata.file_metadata.get_file_size() {
            println!("File upload finished.");
            token_map.remove(&token);
        }

        // Return checksum
        // TODO: Actually return checksum
        let mut upload_file_response = UploadFileResponse::new();
        upload_file_response.set_checksum(0);
        SingleResponse::completed(upload_file_response)
    }

    fn file_is_uploaded(&self, _o: RequestOptions, p: FileMetadata) -> SingleResponse<FileIsUploadedResponse> {
        unimplemented!()
    }
}
