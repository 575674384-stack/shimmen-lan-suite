use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db(app_dir: &std::path::Path) -> Result<DbPool> {
    let db_path = app_dir.join("shimmen.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;"
    )?;
    schema::create_tables(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

mod schema;
pub mod user;
pub mod task;
pub mod password;
pub mod announcement;
pub mod shared_folder;
pub mod file_version;
