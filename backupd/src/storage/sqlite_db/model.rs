use crate::storage::sqlite_db::schema::files;

#[derive(Queryable, Insertable, Identifiable)]
#[table_name="files"]
#[primary_key(real_filename)]
pub struct DbFile {
    pub real_filename: String,
    pub local_filename: String,
    pub last_updated: i64,
}
