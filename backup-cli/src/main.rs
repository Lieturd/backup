use std::env;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

use backuplib::grpc::ClientStubExt;
use backuplib::rpc::*;
use backuplib::client::BaacupClient;

mod configuration;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    backuplib::print_hello();
    println!("backup-cli v{} using backuplib v{}", VERSION, backuplib::VERSION);

    let filename = env::args().skip(1).next().unwrap();

    // Open file
    let mut file = File::open(&filename).unwrap();
    let file_size = file.metadata().unwrap().len();

    // Make client
    let client = BaacupClient::new_plain("127.0.0.1", 8000, Default::default()).unwrap();

    // Get a token
    let file_data = FileMetadata {
        file_name: filename.into(),
        last_modified: 0,
        file_size: file_size,
    };
    let token = client.init_upload(file_data).unwrap();

    loop {
        // Get file head
        let offset = match client.get_head(token) {
            Ok(offset) => offset,
            Err(error) => {
                println!("Error: {}", error);
                break;
            }
        };

        // Read from file
        let mut buffer = [0; 1024];
        file.seek(SeekFrom::Start(offset)).unwrap();
        let bytes = file.read(&mut buffer).unwrap();
        if bytes == 0 {
            break;
        }
        let buffer_vec = buffer[..bytes].to_vec();

        // Upload data
        let file_chunk = FileChunk {
            token: token,
            offset: offset,
            data: buffer_vec,
        };
        let checksum = client.upload_chunk(file_chunk).unwrap();
        if checksum != 0 {
            panic!("Bad upload_resp");
        }

        // Check if we've finished uploading.
        if bytes as u64 + offset == file_size {
            break;
        }
    }
}
