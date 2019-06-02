use std::cmp;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write, Seek, SeekFrom, Result as IoResult};
use std::sync::{Arc, Mutex};

use backupd::storage::{StorageManager, FileLen};
use backupd::server::BaacupImpl;
use futures::future::{self, Future, Loop, Either};

use backuplib::rpc::{Baacup, FileMetadata, FileChunk};

#[derive(Debug, Clone)]
pub struct InMemoryStorage {
    map_mutex: Arc<Mutex<HashMap<String, Arc<Mutex<Vec<u8>>>>>>,
}

impl InMemoryStorage {
    pub fn new() -> InMemoryStorage {
        InMemoryStorage {
            map_mutex: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<'a> StorageManager<'a> for InMemoryStorage {
    fn create(&self, metadata: &FileMetadata) -> Result<(), String> {
        let data = Vec::new();
        let mut map = self.map_mutex.lock().unwrap();
        map.insert(metadata.file_name.clone(), Arc::new(Mutex::new(data)));
        Ok(())
    }

    fn append(&'a self, filename: &str, data: &[u8]) -> Result<(), String> {
        let map = self.map_mutex.lock()
            .map_err(|e| e.to_string())?;
        let file_mutex = map.get_mut(filename)
            .ok_or("bad filename".to_string())?;
        let file = file_mutex.lock()
            .map_err(|e| e.to_string())?;
        file.extend_from_slice(data);
        Ok(())
    }

    fn storage_outdated(&'a self, metadata: &FileMetadata) -> Result<bool, String> {
        Ok(false)
    }

    fn get_head(&'a self, filename: &str) -> Result<u64, String> {
        let map = self.map_mutex.lock()
            .map_err(|e| e.to_string())?;
        let file_mutex = map.get_mut(filename)
            .ok_or("bad filename".to_string())?;
        let file = file_mutex.lock()
            .map_err(|e| e.to_string())?;
        Ok(file.len() as u64)
    }
}

#[test]
fn test_unique_tokens() {
    // Make manager
    let storage_manager = InMemoryStorage::new();

    // Make a new server from the manager
    let server = BaacupImpl::new_from_storage(storage_manager.clone());

    let token_set = HashSet::new();
    let fut = future::loop_fn((server, token_set, 0), move |(server, mut token_set, count)| {
        if count == 10000 {
            Either::A(future::ok(Loop::Break(())))
        }
        else {
            // Upload generated file to server
            let metadata = FileMetadata {
                file_name: format!("file_{}", count),
                last_modified: 0,
                file_size: 1,
            };
            Either::B(server.init_upload(metadata)
                .and_then(move |token| {
                    assert!(!token_set.contains(&token));
                    token_set.insert(token);
                    Ok(Loop::Continue((server, token_set, count + 1)))
                }))
        }
    });
    tokio::run(fut.map_err(|err| panic!("Error: {}", err)));
}

#[test]
fn test_file_upload() {
    // Make manager
    let storage_manager = InMemoryStorage::new();

    // Make a new server from the manager
    let server = BaacupImpl::new_from_storage(storage_manager.clone());

    // Upload generated file to server
    let metadata = FileMetadata {
        file_name: "test_file".into(),
        last_modified: 0,
        file_size: 2048,
    };
    let fut = server.init_upload(metadata).and_then(move |token| {
        server.get_head(token).and_then(move |offset| {
            assert_eq!(offset, 0);
            let chunk = FileChunk {
                token: token,
                offset: offset,
                data: (0..1024).map(|n| (n % 256) as u8).collect(),
            };
            server.upload_chunk(chunk).and_then(move |_checksum| {
                server.get_head(token).and_then(move |offset| {
                    assert_eq!(offset, 1024);
                    let chunk = FileChunk {
                        token: token,
                        offset: offset,
                        data: (0..1024).map(|n| (n % 256) as u8).collect(),
                    };
                    server.upload_chunk(chunk).and_then(move |_checksum| {
                        // Get file from storage manager
                        let mut file = storage_manager.open_storage("test_file".into()).unwrap();
                        let mut buf = Vec::new();

                        // Read file
                        file.read_to_end(&mut buf).unwrap();

                        // Was it the right length?
                        assert_eq!(buf.len(), 2048);

                        // Does it have the right contents?
                        for (idx, byte) in buf.iter().enumerate() {
                            assert_eq!(*byte, (idx % 256) as u8);
                        }
                        Ok(())
                    })
                })
            })
        })
    });
    tokio::run(fut.map_err(|err| panic!("Error: {}", err)));
}
