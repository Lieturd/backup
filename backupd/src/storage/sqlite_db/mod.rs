mod model;
mod schema;

use std::fs::File;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, RunQueryDsl};

use crate::storage::StorageManager;
use crate::storage::sqlite_db::model::DbFile;
use crate::storage::sqlite_db::schema::files;

pub struct SqliteStorageManager {
    connection: SqliteConnection,
}

impl SqliteStorageManager {
    pub fn new(filename: &str) -> SqliteStorageManager {
        let connection = SqliteConnection::establish(filename).unwrap();
        SqliteStorageManager {
            connection: connection,
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

        diesel::insert_into(files::table)
            .values(&new_file)
            .execute(&self.connection)
            .unwrap();

        Ok(file)
    }

    fn open_storage(&'a self, path: String) -> Result<Self::File, String> {
        let file_row = files::table.find(path)
            .first::<DbFile>(&self.connection)
            .unwrap();
        let file = File::open(file_row.local_filename).unwrap();
        Ok(file)
    }
}
