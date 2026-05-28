use rusqlite::Result;

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

pub fn init_db(app_dir: &std::path::Path) -> Result<DbPool> {
    let db_path = app_dir.join("shimmen.db");
    let manager = r2d2_sqlite::SqliteConnectionManager::file(&db_path);
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Null, Box::new(e)))?;
    let conn = pool.get()
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Null, Box::new(e)))?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;"
    )?;
    // 启动时执行一次 WAL checkpoint，防止 -wal 文件无限增长
    let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    schema::create_tables(&conn)?;
    Ok(pool)
}

mod schema;
pub mod user;
pub mod task;
pub mod password;
pub mod announcement;
pub mod shared_folder;
pub mod file_version;
