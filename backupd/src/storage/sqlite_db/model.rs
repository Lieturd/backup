use crate::storage::sqlite_db::schema::files;

#[derive(Queryable, Insertable)]
#[table_name="files"]
pub struct DbFile {
    pub real_filename: String,
    pub local_filename: String,
    pub last_updated: i64,
}
