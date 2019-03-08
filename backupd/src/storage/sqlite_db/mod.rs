mod model;
mod schema;

use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, RunQueryDsl};

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

    fn create_storage(&'a self, path: String) -> Result<Self::File, String> {
        let local_filename = "test.bin";

        let new_file = DbFile {
            real_filename: path,
            local_filename: local_filename.into(),
            last_updated: 0,
        };

        let file = File::create(local_filename).unwrap();

        let connection = self.connection.lock().unwrap();
        diesel::insert_into(files::table)
            .values(&new_file)
            .execute(&*connection)
            .unwrap();

        Ok(file)
    }

    fn open_storage(&'a self, path: String) -> Result<Self::File, String> {
        println!("Opening storage");
        let connection = self.connection.lock().unwrap();
        let file_row = files::table.find(path)
            .first::<DbFile>(&*connection)
            .unwrap();
        OpenOptions::new()
            .write(true)
            .open(file_row.local_filename)
            .map_err(|e| e.to_string())
    }
}
