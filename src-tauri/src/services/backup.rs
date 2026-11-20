use crate::db::migrations::get_schema_version;
use crate::errors::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub app_version: String,
    pub schema_version: i32,
    pub created_at: String,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResultDto {
    pub path: String,
    pub size_bytes: u64,
    pub schema_version: i32,
    pub file_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResultDto {
    pub staged: bool,
    pub restart_required: bool,
    pub schema_version: i32,
}

pub fn create_backup_archive(
    target_path: Option<String>,
    app: &AppHandle,
    conn: &Connection,
) -> Result<BackupResultDto, AppError> {
    let app_data_dir: PathBuf = app.path().app_data_dir().map_err(|e: tauri::Error| {
        AppError::internal_error(format!("Failed to get app data dir: {}", e))
    })?;
    create_backup_archive_in_dir(target_path, &app_data_dir, conn)
}

pub fn create_backup_archive_in_dir(
    target_path: Option<String>,
    app_data_dir: &std::path::Path,
    conn: &Connection,
) -> Result<BackupResultDto, AppError> {
    let schema_version: i32 = get_schema_version(conn).map_err(|e: String| {
        AppError::with_detail("DATABASE_ERROR", "Failed to query schema version", e)
    })?;

    // Determine backup destination zip path
    let out_zip_path: PathBuf = match target_path {
        Some(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
        _ => {
            let backups_dir: PathBuf = app_data_dir.join("backups");
            fs::create_dir_all(&backups_dir).map_err(|e: std::io::Error| {
                AppError::internal_error(format!("Failed to create backups dir: {}", e))
            })?;
            let now_str: String = chrono_timestamp_slug();
            backups_dir.join(format!("backup_{}.zip", now_str))
        }
    };

    if let Some(parent) = out_zip_path.parent() {
        fs::create_dir_all(parent).map_err(|e: std::io::Error| {
            AppError::internal_error(format!("Failed to create backup target dir: {}", e))
        })?;
    }

    // 1. Transaction-safe SQLite online backup to a temporary file
    let temp_db: tempfile::NamedTempFile =
        tempfile::NamedTempFile::new().map_err(|e: std::io::Error| {
            AppError::internal_error(format!("Failed to create temp db file: {}", e))
        })?;
    {
        let mut dst_conn: Connection =
            Connection::open(temp_db.path()).map_err(|e: rusqlite::Error| {
                AppError::database_error(format!("Failed to open destination backup DB: {}", e))
            })?;
        let backup =
            rusqlite::backup::Backup::new(conn, &mut dst_conn).map_err(|e: rusqlite::Error| {
                AppError::database_error(format!("Failed to initialize SQLite backup: {}", e))
            })?;
        backup
            .run_to_completion(5, std::time::Duration::from_millis(50), None)
            .map_err(|e: rusqlite::Error| {
                AppError::database_error(format!("Failed to run SQLite backup: {}", e))
            })?;
    }

    // 2. Prepare Zip Writer
    let zip_file: File = File::create(&out_zip_path).map_err(|e: std::io::Error| {
        AppError::internal_error(format!("Failed to create zip file: {}", e))
    })?;
    let mut zip: ZipWriter<File> = ZipWriter::new(zip_file);
    let options: SimpleFileOptions =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut included_files: Vec<String> = Vec::new();

    // 3. Write SQLite DB
    zip.start_file("tambola.sqlite", options).map_err(|e| {
        AppError::internal_error(format!("Failed to add tambola.sqlite to zip: {}", e))
    })?;
    let db_bytes: Vec<u8> = fs::read(temp_db.path()).map_err(|e: std::io::Error| {
        AppError::internal_error(format!("Failed to read backup DB: {}", e))
    })?;
    zip.write_all(&db_bytes).map_err(|e: std::io::Error| {
        AppError::internal_error(format!("Failed to write DB into zip: {}", e))
    })?;
    included_files.push("tambola.sqlite".to_string());

    // 4. Write posters directory
    let posters_dir: PathBuf = app_data_dir.join("posters");
    if posters_dir.exists() && posters_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&posters_dir) {
            for entry in entries.flatten() {
                let path: PathBuf = entry.path();
                if path.is_file() {
                    let file_name: String = entry.file_name().to_string_lossy().to_string();
                    let zip_entry_name: String = format!("posters/{}", file_name);
                    if let Ok(bytes) = fs::read(&path) {
                        if zip.start_file(&zip_entry_name, options).is_ok()
                            && zip.write_all(&bytes).is_ok()
                        {
                            included_files.push(zip_entry_name);
                        }
                    }
                }
            }
        }
    }

    // 5. Write templates directory
    let templates_dir: PathBuf = app_data_dir.join("templates");
    if templates_dir.exists() && templates_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&templates_dir) {
            for entry in entries.flatten() {
                let path: PathBuf = entry.path();
                if path.is_file() {
                    let file_name: String = entry.file_name().to_string_lossy().to_string();
                    let zip_entry_name: String = format!("templates/{}", file_name);
                    if let Ok(bytes) = fs::read(&path) {
                        if zip.start_file(&zip_entry_name, options).is_ok()
                            && zip.write_all(&bytes).is_ok()
                        {
                            included_files.push(zip_entry_name);
                        }
                    }
                }
            }
        }
    }

    // 6. Write manifest.json
    let manifest: BackupManifest = BackupManifest {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        created_at: chrono_timestamp_iso(),
        files: included_files.clone(),
    };
    let manifest_bytes: Vec<u8> = serde_json::to_vec_pretty(&manifest).map_err(|e| {
        AppError::internal_error(format!("Failed to serialize backup manifest: {}", e))
    })?;
    zip.start_file("manifest.json", options).map_err(|e| {
        AppError::internal_error(format!("Failed to add manifest.json to zip: {}", e))
    })?;
    zip.write_all(&manifest_bytes)
        .map_err(|e: std::io::Error| {
            AppError::internal_error(format!("Failed to write manifest into zip: {}", e))
        })?;

    zip.finish()
        .map_err(|e| AppError::internal_error(format!("Failed to finalize zip archive: {}", e)))?;

    let size_bytes: u64 = fs::metadata(&out_zip_path)
        .map(|m: fs::Metadata| m.len())
        .unwrap_or(0);

    Ok(BackupResultDto {
        path: out_zip_path.to_string_lossy().to_string(),
        size_bytes,
        schema_version,
        file_count: included_files.len() + 1,
    })
}

pub fn restore_backup_archive(
    zip_path_str: &str,
    app: &AppHandle,
    conn: &Connection,
) -> Result<RestoreResultDto, AppError> {
    let app_data_dir: PathBuf = app.path().app_data_dir().map_err(|e: tauri::Error| {
        AppError::internal_error(format!("Failed to get app data dir: {}", e))
    })?;
    restore_backup_archive_in_dir(zip_path_str, &app_data_dir, conn)
}

pub fn restore_backup_archive_in_dir(
    zip_path_str: &str,
    app_data_dir: &std::path::Path,
    conn: &Connection,
) -> Result<RestoreResultDto, AppError> {
    // 1. Guard: Cannot restore while game is ACTIVE or PAUSED
    let has_active_game: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status IN ('ACTIVE', 'PAUSED');",
            [],
            |r: &rusqlite::Row<'_>| r.get::<_, i64>(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);

    if has_active_game {
        return Err(AppError::new(
            "CANNOT_RESTORE_ACTIVE_GAME",
            "Cannot restore backup while a game session is active or paused",
        ));
    }

    let zip_path: PathBuf = PathBuf::from(zip_path_str.trim());
    if !zip_path.exists() || !zip_path.is_file() {
        return Err(AppError::new(
            "BACKUP_INVALID_FILE",
            format!("Backup file not found: {}", zip_path_str),
        ));
    }

    let file: File = File::open(&zip_path).map_err(|e: std::io::Error| {
        AppError::new(
            "BACKUP_INVALID_FILE",
            format!("Failed to open backup zip: {}", e),
        )
    })?;

    let mut archive: ZipArchive<File> = ZipArchive::new(file)
        .map_err(|e| AppError::new("BACKUP_INVALID_FILE", format!("Invalid zip archive: {}", e)))?;

    // 2. Validate manifest
    let manifest: BackupManifest = {
        let mut manifest_file = archive.by_name("manifest.json").map_err(|_| {
            AppError::new(
                "BACKUP_INVALID_FILE",
                "Backup archive is missing manifest.json",
            )
        })?;
        let mut content: String = String::new();
        manifest_file.read_to_string(&mut content).map_err(|e| {
            AppError::new(
                "BACKUP_INVALID_FILE",
                format!("Failed to read manifest.json: {}", e),
            )
        })?;
        serde_json::from_str(&content).map_err(|e| {
            AppError::new(
                "BACKUP_INVALID_FILE",
                format!("Invalid manifest.json structure: {}", e),
            )
        })?
    };

    let current_schema: i32 = get_schema_version(conn).map_err(|e: String| {
        AppError::with_detail("DATABASE_ERROR", "Failed to query schema version", e)
    })?;

    if manifest.schema_version > current_schema {
        return Err(AppError::new(
            "BACKUP_SCHEMA_INCOMPATIBLE",
            format!(
                "Backup schema version ({}) is newer than current application schema ({})",
                manifest.schema_version, current_schema
            ),
        )
        .with_param("backupSchemaVersion", manifest.schema_version.to_string())
        .with_param("currentSchemaVersion", current_schema.to_string()));
    }

    // 3. Stage extraction into app_data_dir/restore_staging
    let staging_dir: PathBuf = app_data_dir.join("restore_staging");

    if staging_dir.exists() {
        let _ = fs::remove_dir_all(&staging_dir);
    }
    fs::create_dir_all(&staging_dir).map_err(|e: std::io::Error| {
        AppError::internal_error(format!("Failed to create restore staging directory: {}", e))
    })?;

    archive.extract(&staging_dir).map_err(|e| {
        AppError::internal_error(format!(
            "Failed to extract backup archive to staging: {}",
            e
        ))
    })?;

    // Write restore_staged.json marker
    let marker_file: PathBuf = app_data_dir.join("restore_staged.json");
    let marker_content: String = serde_json::to_string_pretty(&manifest).unwrap_or_default();
    let _ = fs::write(&marker_file, marker_content);

    Ok(RestoreResultDto {
        staged: true,
        restart_required: true,
        schema_version: manifest.schema_version,
    })
}

fn chrono_timestamp_slug() -> String {
    let now: std::time::SystemTime = std::time::SystemTime::now();
    let duration: std::time::Duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

fn chrono_timestamp_iso() -> String {
    let now: std::time::SystemTime = std::time::SystemTime::now();
    let duration: std::time::Duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{}Z", duration.as_secs(), duration.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn test_create_and_restore_backup_lifecycle() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        // Create sample poster and template files
        let posters_dir: PathBuf = app_data_dir.join("posters");
        let templates_dir: PathBuf = app_data_dir.join("templates");
        fs::create_dir_all(&posters_dir).unwrap();
        fs::create_dir_all(&templates_dir).unwrap();
        fs::write(posters_dir.join("poster_1.jpg"), b"fake poster").unwrap();
        fs::write(templates_dir.join("template_1.png"), b"fake template").unwrap();

        // Create DB and run migrations
        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 1. Create backup
        let backup_result: BackupResultDto =
            create_backup_archive_in_dir(None, &app_data_dir, &conn).unwrap();
        assert_eq!(backup_result.schema_version, 10);
        assert!(PathBuf::from(&backup_result.path).exists());
        assert!(backup_result.file_count >= 3); // sqlite, manifest, poster, template

        // Verify zip contents
        let zip_file: File = File::open(&backup_result.path).unwrap();
        let mut archive: ZipArchive<File> = ZipArchive::new(zip_file).unwrap();
        assert!(archive.by_name("tambola.sqlite").is_ok());
        assert!(archive.by_name("manifest.json").is_ok());
        assert!(archive.by_name("posters/poster_1.jpg").is_ok());
        assert!(archive.by_name("templates/template_1.png").is_ok());

        // 2. Restore backup into a fresh target dir
        let restore_temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let restore_app_data_dir: PathBuf = restore_temp_dir.path().to_path_buf();

        let restore_result: RestoreResultDto =
            restore_backup_archive_in_dir(&backup_result.path, &restore_app_data_dir, &conn)
                .unwrap();
        assert!(restore_result.staged);
        assert!(restore_result.restart_required);
        assert_eq!(restore_result.schema_version, 10);

        // Verify staging directory and marker file
        assert!(restore_app_data_dir.join("restore_staged.json").exists());
        assert!(restore_app_data_dir
            .join("restore_staging")
            .join("tambola.sqlite")
            .exists());
        assert!(restore_app_data_dir
            .join("restore_staging")
            .join("posters")
            .join("poster_1.jpg")
            .exists());
        assert!(restore_app_data_dir
            .join("restore_staging")
            .join("templates")
            .join("template_1.png")
            .exists());

        // 3. Test apply_staged_restore_if_present
        crate::db::connection::apply_staged_restore_if_present(&restore_app_data_dir).unwrap();
        assert!(!restore_app_data_dir.join("restore_staged.json").exists());
        assert!(!restore_app_data_dir.join("restore_staging").exists());
        assert!(restore_app_data_dir
            .join("data")
            .join("tambola.sqlite")
            .exists());
        assert!(restore_app_data_dir
            .join("posters")
            .join("poster_1.jpg")
            .exists());
        assert!(restore_app_data_dir
            .join("templates")
            .join("template_1.png")
            .exists());
    }

    #[test]
    fn test_restore_rejects_active_game() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let backup_result: BackupResultDto =
            create_backup_archive_in_dir(None, &app_data_dir, &conn).unwrap();

        conn.execute(
            "INSERT INTO ticket_batches (id, batch_code, name, ticket_count) VALUES (1, 'B1', 'Test', 10);",
            [],
        )
        .unwrap();

        // Put game in ACTIVE state
        conn.execute(
            "INSERT INTO games (game_code, status, ticket_batch_id, draw_sequence_json)
             VALUES ('G-001', 'ACTIVE', 1, '[]');",
            [],
        )
        .unwrap();

        let err: AppError =
            restore_backup_archive_in_dir(&backup_result.path, &app_data_dir, &conn).unwrap_err();
        assert_eq!(err.code, "CANNOT_RESTORE_ACTIVE_GAME");

        // Put game in PAUSED state
        conn.execute(
            "UPDATE games SET status = 'PAUSED' WHERE status = 'ACTIVE';",
            [],
        )
        .unwrap();
        let err2: AppError =
            restore_backup_archive_in_dir(&backup_result.path, &app_data_dir, &conn).unwrap_err();
        assert_eq!(err2.code, "CANNOT_RESTORE_ACTIVE_GAME");
    }

    #[test]
    fn test_restore_rejects_incompatible_schema() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Create a zip with future schema version (e.g. 99)
        let future_zip: PathBuf = app_data_dir.join("future_backup.zip");
        let zip_file: File = File::create(&future_zip).unwrap();
        let mut zip: ZipWriter<File> = ZipWriter::new(zip_file);
        let options: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file("manifest.json", options).unwrap();
        let manifest: BackupManifest = BackupManifest {
            app_version: "99.0.0".to_string(),
            schema_version: 99,
            created_at: "2099-01-01T00:00:00Z".to_string(),
            files: vec!["tambola.sqlite".to_string()],
        };
        let manifest_bytes: Vec<u8> = serde_json::to_vec(&manifest).unwrap();
        zip.write_all(&manifest_bytes).unwrap();
        zip.finish().unwrap();

        let err: AppError =
            restore_backup_archive_in_dir(&future_zip.to_string_lossy(), &app_data_dir, &conn)
                .unwrap_err();
        assert_eq!(err.code, "BACKUP_SCHEMA_INCOMPATIBLE");
    }

    #[test]
    fn test_restore_rejects_missing_and_corrupt_files() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Missing file
        let err: AppError =
            restore_backup_archive_in_dir("nonexistent.zip", &app_data_dir, &conn).unwrap_err();
        assert_eq!(err.code, "BACKUP_INVALID_FILE");

        // Corrupt file (not a zip)
        let corrupt_file: PathBuf = app_data_dir.join("corrupt.zip");
        fs::write(&corrupt_file, b"not a real zip file content").unwrap();
        let err2: AppError =
            restore_backup_archive_in_dir(&corrupt_file.to_string_lossy(), &app_data_dir, &conn)
                .unwrap_err();
        assert_eq!(err2.code, "BACKUP_INVALID_FILE");
    }
}
