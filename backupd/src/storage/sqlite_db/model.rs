use crate::storage::sqlite_db::schema::files;

#[derive(Queryable, Insertable, Identifiable)]
#[table_name="files"]
#[primary_key(id)]
pub struct DbFile {
    pub id: String,
    pub filename: String,
    pub last_modified: i64,
}
