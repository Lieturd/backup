mod model;
mod schema;

use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};
use std::io::Write;

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
    fn create(&'a self, metadata: &FileMetadata) -> Result<(), String> {
        let id = Uuid::new_v4().to_simple().to_string();

        let connection = self.connection.lock().unwrap();
        let file_row_result = files::table
            .filter(files::filename.eq(&metadata.file_name))
            .first::<DbFile>(&*connection);

        match file_row_result {
            Ok(file_row) => {
                diesel::update(&file_row)
                    .set(files::last_modified.eq(metadata.last_modified as i64))
                    .execute(&*connection)
                    .map_err(|e| e.to_string())?;

                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(file_row.id)
                    .map_err(|e| e.to_string())?;

                Ok(())
            }
            Err(_) => {
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&id)
                    .map_err(|e| e.to_string())?;

                let new_file = DbFile {
                    id: id,
                    filename: metadata.file_name.clone(),
                    last_modified: metadata.last_modified as i64,
                };

                diesel::insert_into(files::table)
                    .values(&new_file)
                    .execute(&*connection)
                    .map_err(|e| e.to_string())?;

                Ok(())
            }
        }
    }

    fn append(&'a self, filename: &str, data: &[u8]) -> Result<(), String> {
        let connection = self.connection.lock().unwrap();

        let file_row = files::table
            .filter(files::filename.eq(&filename))
            .first::<DbFile>(&*connection)
            .unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .open(file_row.id)
            .map_err(|e| e.to_string())?;

        file.write_all(data)
            .map_err(|e| e.to_string())
    }

    fn storage_outdated(&'a self, metadata: &FileMetadata) -> Result<bool, String> {
        let connection = self.connection.lock().unwrap();

        let file_is_updated = files::table
            .filter(files::filename.eq(&metadata.file_name))
            .filter(files::last_modified.eq(metadata.last_modified as i64))
            .first::<DbFile>(&*connection)
            .is_ok();

        Ok(!file_is_updated)
    }

    fn get_head(&'a self, filename: &str) -> Result<u64, String> {
        let connection = self.connection.lock().unwrap();

        let file_row = files::table
            .filter(files::filename.eq(&filename))
            .first::<DbFile>(&*connection)
            .unwrap();

        let mut file = OpenOptions::new()
            .read(true)
            .open(file_row.id)
            .map_err(|e| e.to_string())?;

        file.metadata()
            .map_err(|e| e.to_string())
            .map(|m| m.len())
    }
}
