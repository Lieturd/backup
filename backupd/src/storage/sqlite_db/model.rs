use std::time::Instant;

use crate::storage::sqlite_db::schema::files;

#[derive(Queryable, Insertable)]
#[table_name="files"]
struct DbFile {
    real_filename: String,
    local_filename: String,
    last_updated: i32,
}
