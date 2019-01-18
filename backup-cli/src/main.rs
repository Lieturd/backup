use std::env;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

use backuplib::grpc::ClientStubExt;
use backuplib::grpc::RequestOptions;
use backuplib::proto::baacup_grpc::*;
use backuplib::proto::baacup::*;

mod configuration;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    backuplib::print_hello();
    println!("backup-cli v{} using backuplib v{}", VERSION, backuplib::VERSION);

    let filename = env::args().skip(1).next().unwrap();

    // Open file
    let mut file = File::open(&filename).unwrap();

    // Make client
    let client = BaacupClient::new_plain("127.0.0.1", 8000, Default::default()).unwrap();

    // Get a token
    let mut file_data = FileMetadata::new();
    file_data.set_filename(filename);
    let file_size = file.metadata().unwrap().len();
    file_data.set_file_size(file_size);

    let token_resp = client.init_upload(RequestOptions::new(), file_data);
    let token = token_resp.wait_drop_metadata().unwrap();

    loop {
        // Get file head
        let head_resp = client.get_head(RequestOptions::new(), token.get_token().clone());
        let head = head_resp.wait_drop_metadata().unwrap();
        if head.get_status() == Status::ERROR {
            println!("Error: {}", head.get_error_message());
            break;
        }

        // Read from file
        let mut buffer = [0; 1024];
        file.seek(SeekFrom::Start(head.get_offset())).unwrap();
        let bytes = file.read(&mut buffer).unwrap();
        if bytes == 0 {
            break;
        }
        let buffer_vec = buffer[..bytes].to_vec();

        // Upload data
        let mut file_chunk = FileChunk::new();
        file_chunk.set_token(token.get_token().get_token());
        file_chunk.set_offset(head.get_offset());
        file_chunk.set_data(buffer_vec);
        let upload_resp = client.upload_chunk(RequestOptions::new(), file_chunk);
        if upload_resp.wait_drop_metadata().unwrap().get_checksum() != 0 {
            panic!("Bad upload_resp");
        }
    }
}
