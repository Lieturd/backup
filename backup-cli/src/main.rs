use std::env;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;
use std::sync::Arc;
use std::time::SystemTime;

use backuplib::grpc::ClientStubExt;
use backuplib::rpc::*;
use backuplib::client::BaacupClient;
use futures::Future;
use futures::future::{self, Loop, Either};

mod configuration;
mod file_scanner;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    backuplib::print_hello();
    println!("backup-cli v{} using backuplib v{}", VERSION, backuplib::VERSION);

    let filename = env::args().skip(1).next().unwrap();

    tokio::run(upload_file(filename)
        .map_err(|err| println!("Error: {}", err)));
}

fn upload_file(filename: String) -> impl Future<Item = (), Error = String> {
    // Open file
    let file = File::open(&filename).unwrap();
    let metadata = file.metadata().unwrap();
    let file_size = metadata.len();
    let modified = metadata.modified().unwrap()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap()
        .as_secs();

    // Make client
    let client = BaacupClient::new_plain("127.0.0.1", 8000, Default::default()).unwrap();
    let client = Arc::new(client);

    // Get a token
    let file_data = FileMetadata {
        file_name: filename.into(),
        // TODO: Make last_modified a u64 instead
        last_modified: modified as u32,
        file_size: file_size,
    };
    client.init_upload(file_data)
        .and_then(move |token| {
            future::loop_fn((file, client), move |(mut file, client)| {
                // Get file head
                client.get_head(token)
                    .and_then(move |offset| {
                        // Read from file
                        let mut buffer = [0; 1024];
                        file.seek(SeekFrom::Start(offset)).unwrap();
                        let bytes = file.read(&mut buffer).unwrap();
                        if bytes == 0 {
                            return Either::A(future::ok(Loop::Break(())));
                        }
                        let buffer_vec = buffer[..bytes].to_vec();

                        // Upload data
                        let file_chunk = FileChunk {
                            token: token,
                            offset: offset,
                            data: buffer_vec,
                        };
                        Either::B(client.upload_chunk(file_chunk)
                            .and_then(move |checksum| {
                                if checksum != 0 {
                                    panic!("Bad upload_resp");
                                }

                                // Check if we've finished uploading.
                                if bytes as u64 + offset == file_size {
                                    return Ok(Loop::Break(()));
                                }
                                Ok(Loop::Continue((file, client)))
                            }))
                    })
            })
        })
}
