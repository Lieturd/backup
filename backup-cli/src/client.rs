use std::io::{Read, Seek, SeekFrom};
use std::fs::File;
use std::sync::Arc;
use std::time::SystemTime;

use futures::Future;
use futures::future::{self, Loop, Either};

use backuplib::grpc::ClientStubExt;
use backuplib::rpc::*;
use backuplib::client::BaacupClient;

use crate::configuration::Configuration;
use crate::file_scanner::FileScanner;

pub struct Client {
    config: Configuration,
    scanner: FileScanner,
    client: Arc<BaacupClient>,
}

impl Client {
    pub fn new(config: Configuration) -> Client {
        let scanner = FileScanner::new(config.backup_paths.clone());

        // Make client
        let _client = BaacupClient::new_plain(&config.server_host, config.server_port, Default::default()).unwrap();

        let _client = Arc::new(_client);

        return Client {
            config,
            scanner,
            client: _client,
        };
    }

    pub fn run(mut self) {
        println!("Checking for files to backup...");

        let reader = self.scanner.get_receiver();
        loop {
            let path = reader.recv().unwrap();
            Client::upload_file(
                &self.client,
                path.as_os_str().to_str().unwrap(),
                &self.config.server_host[..],
                self.config.server_port,
            );
        }
    }

    fn upload_file(_client: &Arc<BaacupClient>, path: &str, server_host: &str, server_port: u16) {
        println!("Uploading {:?} to {}:{}", path, server_host, server_port);

        // Open file
        let file;
        match File::open(&path) {
            Ok(f) => {
                file = f;
            }
            Err(e) => {
                println!("Failed to open file {}: {}", path, e);
                return;
            }
        }

        let metadata = file.metadata().unwrap();
        let file_size = metadata.len();
        let last_modified = metadata.modified().unwrap()
            .duration_since(SystemTime::UNIX_EPOCH).unwrap()
            .as_secs();

        // Get a token
        let file_data = FileMetadata {
            file_name: path.into(),
            // TODO: Make last_modified a u64 instead
            last_modified: last_modified as u32,
            file_size,
        };

        tokio::run(start_upload(_client, file, file_data).map_err(|e| println!("Error uploading file: {}", e)));
    }
}

fn start_upload(_client: &Arc<BaacupClient>, file: File, file_data: FileMetadata) -> impl Future<Item=(), Error=String> + '_ {
    _client.file_is_uploaded(file_data.clone())
        .and_then(move |is_uploaded| {
            if !is_uploaded {
                Either::A(
                    init_upload(_client, file, file_data)
                )
            } else {
                Either::B(future::err("file is up to date".into()))
            }
        })
}

fn init_upload(_client: &Arc<BaacupClient>, file: File, file_data: FileMetadata) -> impl Future<Item=(), Error=String> + '_ {
    _client.init_upload(file_data)
        .and_then(move |token| {
            // Get file head
            _client.get_head(token)
                .and_then(move |offset| {
                    upload_file(_client, file, file_data, token, offset)
                })
        })
}

fn upload_file(_client: &Arc<BaacupClient>, file: File, file_data: FileMetadata, token: u32, offset: u64) -> impl Future<Item=(), Error=String> + '_ {
    future::loop_fn((_client, file, file_data, token, offset), move |(_client, mut file, file_data, token, offset)| {
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
        Either::B(_client.upload_chunk(file_chunk)
            .and_then(move |checksum| {
                if checksum != 0 {
                    panic!("Bad upload response");
                }

                // Check if we've finished uploading.
                if bytes as u64 + offset == file_data.file_size {
                    return Ok(Loop::Break(()));
                }
                Ok(Loop::Continue((_client, file, file_data, token, offset + 1024)))
            }))
    })
}