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

pub const DEFAULT_MAX_ZIP_ENTRIES: usize = 20_000;
pub const DEFAULT_MAX_UNCOMPRESSED_SIZE: u64 = 2 * 1024 * 1024 * 1024; // 2 GB

pub fn restore_backup_archive_in_dir(
    zip_path_str: &str,
    app_data_dir: &std::path::Path,
    conn: &Connection,
) -> Result<RestoreResultDto, AppError> {
    restore_backup_archive_with_limits(
        zip_path_str,
        app_data_dir,
        conn,
        DEFAULT_MAX_ZIP_ENTRIES,
        DEFAULT_MAX_UNCOMPRESSED_SIZE,
    )
}

pub fn restore_backup_archive_with_limits(
    zip_path_str: &str,
    app_data_dir: &std::path::Path,
    conn: &Connection,
    max_zip_entries: usize,
    max_uncompressed_size: u64,
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

    // 2. Validate zip archive limits and path allowlist
    if archive.len() > max_zip_entries {
        return Err(AppError::new(
            "BACKUP_INVALID_FILE",
            format!(
                "Backup archive contains too many entries ({} > {})",
                archive.len(),
                max_zip_entries
            ),
        ));
    }

    let mut total_uncompressed_size: u64 = 0;
    let mut has_sqlite_db: bool = false;
    let mut has_manifest: bool = false;

    for i in 0..archive.len() {
        let entry: zip::read::ZipFile<'_, File> = archive.by_index(i).map_err(|e| {
            AppError::new(
                "BACKUP_INVALID_FILE",
                format!("Failed to read archive entry #{}: {}", i, e),
            )
        })?;

        total_uncompressed_size = total_uncompressed_size.saturating_add(entry.size());
        if total_uncompressed_size > max_uncompressed_size {
            return Err(AppError::new(
                "BACKUP_INVALID_FILE",
                format!(
                    "Backup archive exceeds uncompressed size limit of {} bytes",
                    max_uncompressed_size
                ),
            ));
        }

        let name: &str = entry.name();
        let norm_name: String = name.replace('\\', "/");
        if norm_name.contains("..") || norm_name.starts_with('/') {
            return Err(AppError::new(
                "BACKUP_INVALID_FILE",
                format!("Backup archive contains illegal path: {}", name),
            ));
        }

        if norm_name == "tambola.sqlite" {
            has_sqlite_db = true;
        } else if norm_name == "manifest.json" {
            has_manifest = true;
        } else if norm_name == "posters/"
            || norm_name.starts_with("posters/")
            || norm_name == "templates/"
            || norm_name.starts_with("templates/")
        {
            // Allowed asset paths
        } else {
            return Err(AppError::new(
                "BACKUP_INVALID_FILE",
                format!("Backup archive contains unexpected file: {}", name),
            ));
        }
    }

    if !has_sqlite_db {
        return Err(AppError::new(
            "BACKUP_INVALID_FILE",
            "Backup archive is missing required database tambola.sqlite",
        ));
    }

    if !has_manifest {
        return Err(AppError::new(
            "BACKUP_INVALID_FILE",
            "Backup archive is missing manifest.json",
        ));
    }

    // 3. Validate manifest
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

    // 4. Stage extraction into app_data_dir/restore_staging
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

    // Write restore_staged.json marker (propagate errors)
    let marker_file: PathBuf = app_data_dir.join("restore_staged.json");
    let marker_content: String = serde_json::to_string_pretty(&manifest).map_err(|e| {
        AppError::internal_error(format!("Failed to serialize restore manifest: {}", e))
    })?;
    fs::write(&marker_file, marker_content).map_err(|e: std::io::Error| {
        AppError::internal_error(format!("Failed to write restore_staged.json marker: {}", e))
    })?;

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
        assert_eq!(
            backup_result.schema_version,
            crate::db::migrations::LATEST_SCHEMA_VERSION
        );
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
        assert_eq!(
            restore_result.schema_version,
            crate::db::migrations::LATEST_SCHEMA_VERSION
        );

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
        zip.start_file("tambola.sqlite", options).unwrap();
        zip.write_all(b"sqlite dummy").unwrap();
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

    #[test]
    fn test_zip_safety_rejects_disallowed_paths_and_missing_db() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 1. Disallowed path `evil.exe`
        let evil_zip: PathBuf = app_data_dir.join("evil.zip");
        let zip_file: File = File::create(&evil_zip).unwrap();
        let mut zip: ZipWriter<File> = ZipWriter::new(zip_file);
        let options: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(b"{}").unwrap();
        zip.start_file("tambola.sqlite", options).unwrap();
        zip.write_all(b"db").unwrap();
        zip.start_file("evil.exe", options).unwrap();
        zip.write_all(b"bad").unwrap();
        zip.finish().unwrap();

        let err: AppError =
            restore_backup_archive_in_dir(&evil_zip.to_string_lossy(), &app_data_dir, &conn)
                .unwrap_err();
        assert_eq!(err.code, "BACKUP_INVALID_FILE");

        // 2. Path traversal
        let slip_zip: PathBuf = app_data_dir.join("slip.zip");
        let zip_file2: File = File::create(&slip_zip).unwrap();
        let mut zip2: ZipWriter<File> = ZipWriter::new(zip_file2);
        zip2.start_file("manifest.json", options).unwrap();
        zip2.write_all(b"{}").unwrap();
        zip2.start_file("tambola.sqlite", options).unwrap();
        zip2.write_all(b"db").unwrap();
        zip2.start_file("../outside.txt", options).unwrap();
        zip2.write_all(b"slip").unwrap();
        zip2.finish().unwrap();

        let err2: AppError =
            restore_backup_archive_in_dir(&slip_zip.to_string_lossy(), &app_data_dir, &conn)
                .unwrap_err();
        assert_eq!(err2.code, "BACKUP_INVALID_FILE");

        // 3. Missing tambola.sqlite
        let no_db_zip: PathBuf = app_data_dir.join("nodb.zip");
        let zip_file3: File = File::create(&no_db_zip).unwrap();
        let mut zip3: ZipWriter<File> = ZipWriter::new(zip_file3);
        zip3.start_file("manifest.json", options).unwrap();
        zip3.write_all(b"{}").unwrap();
        zip3.finish().unwrap();

        let err3: AppError =
            restore_backup_archive_in_dir(&no_db_zip.to_string_lossy(), &app_data_dir, &conn)
                .unwrap_err();
        assert_eq!(err3.code, "BACKUP_INVALID_FILE");
    }

    #[test]
    fn test_apply_staged_restore_integrity_check_failure_aborts() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        // Create original DB in data/
        let data_dir: PathBuf = app_data_dir.join("data");
        fs::create_dir_all(&data_dir).unwrap();
        let original_db_path: PathBuf = data_dir.join("tambola.sqlite");
        let orig_conn: Connection = Connection::open(&original_db_path).unwrap();
        run_migrations(&orig_conn).unwrap();
        orig_conn
            .execute("CREATE TABLE sentinel_test (id INT);", [])
            .unwrap();
        drop(orig_conn);

        // Stage a corrupt database
        let staging_dir: PathBuf = app_data_dir.join("restore_staging");
        fs::create_dir_all(&staging_dir).unwrap();
        fs::write(
            staging_dir.join("tambola.sqlite"),
            b"corrupt non-sqlite file content",
        )
        .unwrap();
        fs::write(
            app_data_dir.join("restore_staged.json"),
            b"{\"schemaVersion\":10}",
        )
        .unwrap();

        // Apply staged restore -> should fail and abort
        let res: Result<(), String> =
            crate::db::connection::apply_staged_restore_if_present(&app_data_dir);
        assert!(res.is_err());

        // Staging removed
        assert!(!staging_dir.exists());
        assert!(!app_data_dir.join("restore_staged.json").exists());

        // Original DB remains intact and operational
        let check_conn: Connection = Connection::open(&original_db_path).unwrap();
        let exists: bool = check_conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sentinel_test';",
                [],
                |r: &rusqlite::Row<'_>| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap();
        assert!(exists);
    }

    #[test]
    fn test_simulated_copy_failure_rolls_back() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        // 1. Original database with sentinel table
        let data_dir: PathBuf = app_data_dir.join("data");
        fs::create_dir_all(&data_dir).unwrap();
        let original_db_path: PathBuf = data_dir.join("tambola.sqlite");
        let orig_conn: Connection = Connection::open(&original_db_path).unwrap();
        run_migrations(&orig_conn).unwrap();
        orig_conn
            .execute("CREATE TABLE rollback_sentinel (value INT);", [])
            .unwrap();
        orig_conn
            .execute("INSERT INTO rollback_sentinel VALUES (123);", [])
            .unwrap();
        drop(orig_conn);

        // 2. Staged valid database
        let staging_dir: PathBuf = app_data_dir.join("restore_staging");
        fs::create_dir_all(&staging_dir).unwrap();
        let staged_conn: Connection = Connection::open(staging_dir.join("tambola.sqlite")).unwrap();
        run_migrations(&staged_conn).unwrap();
        staged_conn
            .execute("CREATE TABLE staged_marker (val INT);", [])
            .unwrap();
        drop(staged_conn);

        fs::write(
            app_data_dir.join("restore_staged.json"),
            b"{\"schemaVersion\":10}",
        )
        .unwrap();

        // 3. Apply with simulated copy failure
        let res: Result<(), String> =
            crate::db::connection::apply_staged_restore_internal(&app_data_dir, true);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Simulated disk copy failure"));

        // 4. Verify original database content was preserved by rollback
        let check_conn: Connection = Connection::open(&original_db_path).unwrap();
        let val: i64 = check_conn
            .query_row(
                "SELECT value FROM rollback_sentinel;",
                [],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .unwrap();
        assert_eq!(val, 123);

        // Staged marker and directory are cleaned up
        assert!(!staging_dir.exists());
        assert!(!app_data_dir.join("restore_staged.json").exists());
    }

    #[test]
    fn test_successful_restore_keeps_pre_restore_copy() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        // 1. Original database with distinct content
        let data_dir: PathBuf = app_data_dir.join("data");
        fs::create_dir_all(&data_dir).unwrap();
        let original_db_path: PathBuf = data_dir.join("tambola.sqlite");
        let orig_conn: Connection = Connection::open(&original_db_path).unwrap();
        run_migrations(&orig_conn).unwrap();
        orig_conn
            .execute("CREATE TABLE old_state (id INT);", [])
            .unwrap();
        orig_conn
            .execute("INSERT INTO old_state VALUES (42);", [])
            .unwrap();
        drop(orig_conn);

        // 2. Staged database with new state
        let staging_dir: PathBuf = app_data_dir.join("restore_staging");
        fs::create_dir_all(&staging_dir).unwrap();
        let staged_conn: Connection = Connection::open(staging_dir.join("tambola.sqlite")).unwrap();
        run_migrations(&staged_conn).unwrap();
        staged_conn
            .execute("CREATE TABLE new_state (id INT);", [])
            .unwrap();
        staged_conn
            .execute("INSERT INTO new_state VALUES (99);", [])
            .unwrap();
        drop(staged_conn);

        fs::write(
            app_data_dir.join("restore_staged.json"),
            b"{\"schemaVersion\":10}",
        )
        .unwrap();

        // 3. Apply restore
        crate::db::connection::apply_staged_restore_if_present(&app_data_dir).unwrap();

        // 4. Verify current DB has new state
        let current_conn: Connection = Connection::open(&original_db_path).unwrap();
        let new_val: i64 = current_conn
            .query_row("SELECT id FROM new_state;", [], |r: &rusqlite::Row<'_>| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(new_val, 99);
        drop(current_conn);

        // 5. Verify pre_restore folder contains OLD state
        let entries = fs::read_dir(&app_data_dir).unwrap();
        let mut found_pre_restore: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let p: PathBuf = entry.path();
            if p.is_dir() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("pre_restore_") {
                        found_pre_restore = Some(p);
                        break;
                    }
                }
            }
        }

        assert!(found_pre_restore.is_some());
        let pre_restore_dir: PathBuf = found_pre_restore.unwrap();
        let pre_restore_db: PathBuf = pre_restore_dir.join("data").join("tambola.sqlite");
        assert!(pre_restore_db.exists());

        let pre_conn: Connection = Connection::open(&pre_restore_db).unwrap();
        let old_val: i64 = pre_conn
            .query_row("SELECT id FROM old_state;", [], |r: &rusqlite::Row<'_>| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(old_val, 42);
    }

    #[test]
    fn test_zip_over_size_or_entry_cap_is_rejected() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let app_data_dir: PathBuf = temp_dir.path().to_path_buf();

        let conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Create a backup archive with 3 files
        let test_zip: PathBuf = app_data_dir.join("test_limits.zip");
        let zip_file: File = File::create(&test_zip).unwrap();
        let mut zip: ZipWriter<File> = ZipWriter::new(zip_file);
        let options: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(b"{\"schemaVersion\":10}").unwrap();
        zip.start_file("tambola.sqlite", options).unwrap();
        zip.write_all(b"sample sqlite payload content exceeding tiny limit")
            .unwrap();
        zip.start_file("posters/p1.jpg", options).unwrap();
        zip.write_all(b"poster byte content").unwrap();
        zip.finish().unwrap();

        // 1. Test entry cap: max_entries = 2 (archive has 3)
        let err1: AppError = restore_backup_archive_with_limits(
            &test_zip.to_string_lossy(),
            &app_data_dir,
            &conn,
            2,
            DEFAULT_MAX_UNCOMPRESSED_SIZE,
        )
        .unwrap_err();
        assert_eq!(err1.code, "BACKUP_INVALID_FILE");
        assert!(err1.message.contains("too many entries"));

        // 2. Test size cap: max_uncompressed_size = 10 bytes (archive uncompressed size is ~90 bytes)
        let err2: AppError = restore_backup_archive_with_limits(
            &test_zip.to_string_lossy(),
            &app_data_dir,
            &conn,
            DEFAULT_MAX_ZIP_ENTRIES,
            10,
        )
        .unwrap_err();
        assert_eq!(err2.code, "BACKUP_INVALID_FILE");
        assert!(err2.message.contains("exceeds uncompressed size limit"));
    }
}
