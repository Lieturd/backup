mod model;
mod schema;

use std::fs::File;

use diesel::sqlite::SqliteConnection;
use diesel::Connection;

use crate::storage::StorageManager;

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
        unimplemented!()
    }

    fn open_storage(&'a self, path: String) -> Result<Self::File, String> {
        unimplemented!()
    }
}
