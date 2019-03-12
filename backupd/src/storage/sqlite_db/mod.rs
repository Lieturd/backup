mod model;
mod schema;

use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, RunQueryDsl};
use uuid::Uuid;
use backuplib::rpc::FileMetadata;

use crate::storage::StorageManager;
use crate::storage::sqlite_db::model::DbFile;
use crate::storage::sqlite_db::schema::files;

pub struct SqliteStorageManager {
    connection: Arc<Mutex<SqliteConnection>>,
}

impl SqliteStorageManager {
    pub fn new(filename: &str) -> SqliteStorageManager {
        let connection = SqliteConnection::establish(filename).unwrap();
        SqliteStorageManager {
            connection: Arc::new(Mutex::new(connection)),
        }
    }
}

impl<'a> StorageManager<'a> for SqliteStorageManager {
    type File = File;

    fn create_storage(&'a self, metadata: &FileMetadata) -> Result<Self::File, String> {
        let local_filename = Uuid::new_v4().to_simple().to_string();

        let connection = self.connection.lock().unwrap();
        let file_row_result = files::table.find(&metadata.file_name)
            .first::<DbFile>(&*connection);

        match file_row_result {
            Ok(file_row) => {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(file_row.local_filename)
                    .map_err(|e| e.to_string())
            }
            Err(_) => {
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&local_filename)
                    .map_err(|e| e.to_string())?;

                let new_file = DbFile {
                    real_filename: metadata.file_name.clone(),
                    local_filename: local_filename,
                    last_updated: metadata.last_modified as i64,
                };

                diesel::insert_into(files::table)
                    .values(&new_file)
                    .execute(&*connection)
                    .map_err(|e| e.to_string())?;

                Ok(file)
            }
        }
    }

    fn open_storage(&'a self, path: String) -> Result<Self::File, String> {
        let connection = self.connection.lock().unwrap();

        let file_row = files::table.find(path)
            .first::<DbFile>(&*connection)
            .unwrap();

        OpenOptions::new()
            .write(true)
            .open(file_row.local_filename)
            .map_err(|e| e.to_string())
    }

    fn storage_outdated(&'a self, metadata: &FileMetadata) -> Result<bool, String> {
        let connection = self.connection.lock().unwrap();

        let file_is_updated = files::table.find(&metadata.file_name)
            .filter(files::last_updated.eq(metadata.last_modified as i64))
            .first::<DbFile>(&*connection)
            .is_ok();

        Ok(!file_is_updated)
    }
}
