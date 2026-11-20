use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct DbState {
    pub conn: Mutex<Connection>,
    pub db_path: PathBuf,
}

fn move_or_copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    let _ = fs::create_dir_all(dst);
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let target: PathBuf = dst.join(entry.file_name());
            if path.is_dir() {
                move_or_copy_dir(&path, &target)?;
            } else if path.is_file() {
                fs::copy(&path, &target).map_err(|e: std::io::Error| {
                    format!("Failed to copy file {:?}: {}", path, e)
                })?;
            }
        }
    }
    let _ = fs::remove_dir_all(src);
    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    let _ = fs::create_dir_all(dst);
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let target: PathBuf = dst.join(entry.file_name());
            if path.is_dir() {
                copy_dir_contents(&path, &target)?;
            } else if path.is_file() {
                fs::copy(&path, &target).map_err(|e: std::io::Error| {
                    format!("Failed to copy {:?} to {:?}: {}", path, target, e)
                })?;
            }
        }
    }
    Ok(())
}

fn prune_old_pre_restores(base_dir: &Path, keep_count: usize) {
    if let Ok(entries) = fs::read_dir(base_dir) {
        let mut pre_restores: Vec<PathBuf> = Vec::new();
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("pre_restore_") {
                        pre_restores.push(path);
                    }
                }
            }
        }
        pre_restores.sort();
        if pre_restores.len() > keep_count {
            let to_remove: usize = pre_restores.len() - keep_count;
            for dir in pre_restores.iter().take(to_remove) {
                let _ = fs::remove_dir_all(dir);
            }
        }
    }
}

pub fn apply_staged_restore_if_present(base_dir: &Path) -> Result<(), String> {
    let staging_dir: PathBuf = base_dir.join("restore_staging");
    let marker_file: PathBuf = base_dir.join("restore_staged.json");

    if !staging_dir.exists() || !staging_dir.is_dir() || !marker_file.exists() {
        return Ok(());
    }

    log::info!(
        "Staged restore detected at {:?}; validating before apply...",
        staging_dir
    );

    // (a) Verify staged database integrity & schema version
    let staged_db: PathBuf = staging_dir.join("tambola.sqlite");
    if !staged_db.exists() || !staged_db.is_file() {
        log::error!("Staged restore aborted: tambola.sqlite missing in staging dir.");
        let _ = fs::remove_dir_all(&staging_dir);
        let _ = fs::remove_file(&marker_file);
        return Err("Staged restore aborted: missing database".to_string());
    }

    let staged_check_result: Result<(), String> = (|| -> Result<(), String> {
        let staged_conn: Connection =
            Connection::open_with_flags(&staged_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e: rusqlite::Error| format!("Staged database cannot be opened: {}", e))?;

        let integrity: String = staged_conn
            .query_row("PRAGMA integrity_check;", [], |r: &rusqlite::Row<'_>| {
                r.get(0)
            })
            .map_err(|e: rusqlite::Error| {
                format!(
                    "Failed to execute integrity_check on staged database: {}",
                    e
                )
            })?;

        if integrity != "ok" {
            log::error!(
                "Staged database failed integrity check: {}; aborting restore.",
                integrity
            );
            return Err(format!(
                "Staged database corrupted: integrity check returned {}",
                integrity
            ));
        }

        let staged_schema: i32 =
            crate::db::migrations::get_schema_version(&staged_conn).map_err(|e: String| {
                format!("Failed to query staged database schema version: {}", e)
            })?;

        if staged_schema > crate::db::migrations::LATEST_SCHEMA_VERSION {
            log::error!(
                "Staged database schema version {} exceeds supported {}; aborting restore.",
                staged_schema,
                crate::db::migrations::LATEST_SCHEMA_VERSION
            );
            return Err("Staged database schema is newer than application schema".to_string());
        }

        Ok(())
    })();

    if let Err(err) = staged_check_result {
        log::error!(
            "Staged database validation failed: {}; aborting restore.",
            err
        );
        let _ = fs::remove_dir_all(&staging_dir);
        let _ = fs::remove_file(&marker_file);
        return Err(err);
    }

    // (b) Checkpoint current DB and move current assets to pre_restore_<timestamp>/
    let data_dir: PathBuf = base_dir.join("data");
    let current_db: PathBuf = data_dir.join("tambola.sqlite");

    if current_db.exists() {
        log::info!("Checkpointing current database WAL before moving...");
        if let Ok(curr_conn) = Connection::open(&current_db) {
            let _ = curr_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
    }

    let now_ts: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let pre_restore_dir: PathBuf = base_dir.join(format!("pre_restore_{}", now_ts));
    let pre_restore_data: PathBuf = pre_restore_dir.join("data");
    fs::create_dir_all(&pre_restore_data)
        .map_err(|e: std::io::Error| format!("Failed to create pre_restore directory: {}", e))?;

    log::info!(
        "Backing up current runtime state into {:?}",
        pre_restore_dir
    );

    // Copy/move current DB files
    if current_db.exists() {
        let _ = fs::copy(&current_db, pre_restore_data.join("tambola.sqlite"));
        let wal: PathBuf = data_dir.join("tambola.sqlite-wal");
        if wal.exists() {
            let _ = fs::copy(&wal, pre_restore_data.join("tambola.sqlite-wal"));
            let _ = fs::remove_file(&wal);
        }
        let shm: PathBuf = data_dir.join("tambola.sqlite-shm");
        if shm.exists() {
            let _ = fs::copy(&shm, pre_restore_data.join("tambola.sqlite-shm"));
            let _ = fs::remove_file(&shm);
        }
    }

    // Move current posters
    let posters_dir: PathBuf = base_dir.join("posters");
    if posters_dir.exists() {
        let _ = move_or_copy_dir(&posters_dir, &pre_restore_dir.join("posters"));
    }

    // Move current templates
    let templates_dir: PathBuf = base_dir.join("templates");
    if templates_dir.exists() {
        let _ = move_or_copy_dir(&templates_dir, &pre_restore_dir.join("templates"));
    }

    prune_old_pre_restores(base_dir, 3);

    // (c) Copy staged files in, with (d) rollback on any error
    let apply_result: Result<(), String> = (|| -> Result<(), String> {
        let _ = fs::create_dir_all(&data_dir);
        fs::copy(&staged_db, &current_db)
            .map_err(|e: std::io::Error| format!("Failed to copy restored database: {}", e))?;

        let staged_posters: PathBuf = staging_dir.join("posters");
        if staged_posters.exists() && staged_posters.is_dir() {
            copy_dir_contents(&staged_posters, &posters_dir)?;
        }

        let staged_templates: PathBuf = staging_dir.join("templates");
        if staged_templates.exists() && staged_templates.is_dir() {
            copy_dir_contents(&staged_templates, &templates_dir)?;
        }

        Ok(())
    })();

    if let Err(err) = apply_result {
        log::error!(
            "Error copying staged restore files: {}; rolling back to pre_restore...",
            err
        );
        // Rollback
        if pre_restore_data.join("tambola.sqlite").exists() {
            let _ = fs::copy(pre_restore_data.join("tambola.sqlite"), &current_db);
            let wal: PathBuf = pre_restore_data.join("tambola.sqlite-wal");
            if wal.exists() {
                let _ = fs::copy(&wal, data_dir.join("tambola.sqlite-wal"));
            }
            let shm: PathBuf = pre_restore_data.join("tambola.sqlite-shm");
            if shm.exists() {
                let _ = fs::copy(&shm, data_dir.join("tambola.sqlite-shm"));
            }
        }
        if pre_restore_dir.join("posters").exists() {
            let _ = move_or_copy_dir(&pre_restore_dir.join("posters"), &posters_dir);
        }
        if pre_restore_dir.join("templates").exists() {
            let _ = move_or_copy_dir(&pre_restore_dir.join("templates"), &templates_dir);
        }
        let _ = fs::remove_dir_all(&staging_dir);
        let _ = fs::remove_file(&marker_file);
        return Err(format!(
            "Staged restore failed and was rolled back: {}",
            err
        ));
    }

    // (e) Success: cleanup staging dir and marker
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
