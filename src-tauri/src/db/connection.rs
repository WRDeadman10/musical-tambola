use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct DbState {
    pub conn: Mutex<Connection>,
    pub db_path: PathBuf,
}

pub fn apply_staged_restore_if_present(base_dir: &Path) -> Result<(), String> {
    let staging_dir: PathBuf = base_dir.join("restore_staging");
    let marker_file: PathBuf = base_dir.join("restore_staged.json");

    if !staging_dir.exists() || !staging_dir.is_dir() {
        return Ok(());
    }

    let staged_db: PathBuf = staging_dir.join("tambola.sqlite");
    if staged_db.exists() && staged_db.is_file() {
        let data_dir: PathBuf = base_dir.join("data");
        let _ = fs::create_dir_all(&data_dir);
        let target_db: PathBuf = data_dir.join("tambola.sqlite");

        // Remove old WAL and SHM files if present
        let _ = fs::remove_file(data_dir.join("tambola.sqlite-wal"));
        let _ = fs::remove_file(data_dir.join("tambola.sqlite-shm"));

        fs::copy(&staged_db, &target_db)
            .map_err(|e: std::io::Error| format!("Failed to copy restored database: {}", e))?;
    }

    let staged_posters: PathBuf = staging_dir.join("posters");
    if staged_posters.exists() && staged_posters.is_dir() {
        let target_posters: PathBuf = base_dir.join("posters");
        let _ = fs::create_dir_all(&target_posters);
        if let Ok(entries) = fs::read_dir(&staged_posters) {
            for entry in entries.flatten() {
                let path: PathBuf = entry.path();
                if path.is_file() {
                    let dest: PathBuf = target_posters.join(entry.file_name());
                    let _ = fs::copy(&path, &dest);
                }
            }
        }
    }

    let staged_templates: PathBuf = staging_dir.join("templates");
    if staged_templates.exists() && staged_templates.is_dir() {
        let target_templates: PathBuf = base_dir.join("templates");
        let _ = fs::create_dir_all(&target_templates);
        if let Ok(entries) = fs::read_dir(&staged_templates) {
            for entry in entries.flatten() {
                let path: PathBuf = entry.path();
                if path.is_file() {
                    let dest: PathBuf = target_templates.join(entry.file_name());
                    let _ = fs::copy(&path, &dest);
                }
            }
        }
    }

    let _ = fs::remove_dir_all(&staging_dir);
    let _ = fs::remove_file(&marker_file);

    log::info!("Applied staged database and asset restore successfully.");
    Ok(())
}

pub fn get_app_data_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let base_dir: PathBuf = app_handle
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| format!("Failed to resolve app data directory: {}", e))?;

    let _ = apply_staged_restore_if_present(&base_dir);

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
