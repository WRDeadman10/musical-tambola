use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct DbState {
    pub conn: Mutex<Connection>,
    pub db_path: PathBuf,
}

pub fn get_app_data_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let base_dir: PathBuf = app_handle
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| format!("Failed to resolve app data directory: {}", e))?;

    let data_dir: PathBuf = base_dir.join("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e: std::io::Error| format!("Failed to create data directory: {}", e))?;
    }

    Ok(data_dir)
}

pub fn initialize_database(db_path: &Path) -> Result<Connection, String> {
    let conn: Connection = Connection::open(db_path)
        .map_err(|e: rusqlite::Error| format!("Failed to open SQLite database: {}", e))?;

    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )
    .map_err(|e: rusqlite::Error| format!("Failed to set database pragmas: {}", e))?;

    super::migrations::run_migrations(&conn)?;

    Ok(conn)
}
