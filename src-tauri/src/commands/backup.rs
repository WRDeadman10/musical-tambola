use crate::db::DbState;
use crate::errors::AppError;
use crate::services::admin_pin::AdminPinState;
use crate::services::backup::{
    create_backup_archive, create_backup_archive_in_dir, restore_backup_archive, BackupResultDto,
    RestoreResultDto,
};
use tauri::{AppHandle, Manager, State, Window};

#[tauri::command]
pub fn create_backup(
    target_path: Option<String>,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<BackupResultDto, AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;
    create_backup_archive(target_path, &app, &conn)
}

#[tauri::command]
pub fn restore_backup(
    path: String,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<RestoreResultDto, AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    restore_backup_archive(&path, &app, &conn)
}

#[tauri::command]
pub fn reset_all_data(
    confirm: String,
    pin: Option<String>,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<BackupResultDto, AppError> {
    crate::errors::require_main_window(&window)?;

    if confirm.trim() != "RESET_ALL_DATA" {
        return Err(AppError::new(
            "RESET_DATA_CONFIRM_REQUIRED",
            "Confirmation text 'RESET_ALL_DATA' is required to perform factory reset",
        ));
    }

    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    // Check PIN requirement: if admin PIN is configured, verify the passed PIN or existing unlocked session
    let pin_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'admin_pin_hash';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .ok()
        .filter(|s: &String| !s.trim().is_empty());

    if let Some(ref stored_hash) = pin_hash {
        let is_session_unlocked: bool = pin_state.is_session_valid();
        if !is_session_unlocked {
            pin_state.check_rate_limit()?;
            let provided_pin: &str = pin.as_deref().unwrap_or("");
            if !crate::services::admin_pin::verify_pin_hash(provided_pin, stored_hash)? {
                pin_state.record_failure();
                return Err(AppError::new(
                    "ADMIN_PIN_INVALID",
                    "Admin PIN is incorrect for factory reset",
                ));
            }
        }
    }

    // 1. Create an automatic pre-reset backup in app_data_dir/backups
    let app_data_dir: std::path::PathBuf =
        app.path().app_data_dir().map_err(|e: tauri::Error| {
            AppError::internal_error(format!("Failed to get app data dir: {}", e))
        })?;

    let backup_dto: BackupResultDto = create_backup_archive_in_dir(None, &app_data_dir, &conn)?;

    // 2. Wipe all tables safely in a single transaction with foreign keys off
    wipe_all_tables(&conn).map_err(AppError::database_error)?;

    // 3. Delete poster files on disk (data/posters)
    let posters_dir: std::path::PathBuf = app_data_dir.join("data").join("posters");
    if posters_dir.exists() {
        let _ = std::fs::remove_dir_all(&posters_dir);
    }

    // 4. Re-run migrations fresh to seed default schemes and initial baseline schema
    crate::db::migrations::run_migrations(&conn).map_err(|e: String| {
        AppError::internal_error(format!("Failed to re-run migrations: {}", e))
    })?;

    // Clear unlocked PIN session
    pin_state.clear_session();

    Ok(backup_dto)
}

/// Settings rows that are not preferences: deleting them would corrupt data or weaken security.
/// `ticket_sequence_counter` numbers new tickets (a reset would reuse numbers of existing tickets),
/// the format keys match the stored batches and media, and the PIN hash protects the app.
pub const PRESERVED_SETTING_KEYS: [&str; 5] = [
    "ticket_sequence_counter",
    "active_game_format",
    "active_song_count",
    "admin_pin_hash",
    "__smoke_sentinel__",
];

/// Wipes all application tables transactionally with foreign keys disabled.
pub fn wipe_all_tables(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
    conn.execute_batch("BEGIN IMMEDIATE;")?;
    let res = conn.execute_batch(
        "DELETE FROM claims;
         DELETE FROM lucky_draw_winners;
         DELETE FROM lucky_draws;
         DELETE FROM game_media_snapshots;
         DELETE FROM game_schemes;
         DELETE FROM games;
         DELETE FROM ticket_cells;
         DELETE FROM tickets;
         DELETE FROM ticket_batches;
         DELETE FROM ticket_templates;
         DELETE FROM media_assets;
         DELETE FROM prize_schemes;
         DELETE FROM app_settings;
         DELETE FROM schema_migrations;",
    );
    match res {
        Ok(_) => {
            conn.execute_batch("COMMIT;")?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            Ok(())
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK;");
            let _ = conn.execute_batch("PRAGMA foreign_keys = ON;");
            Err(err)
        }
    }
}

/// Puts every preference back to its default by deleting it; the rest of `app_settings` stays.
pub fn reset_preference_rows(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    let placeholders: String = PRESERVED_SETTING_KEYS
        .iter()
        .map(|_| "?")
        .collect::<Vec<&str>>()
        .join(",");
    let sql: String = format!(
        "DELETE FROM app_settings WHERE key NOT IN ({});",
        placeholders
    );
    conn.execute_batch("BEGIN IMMEDIATE;")?;
    match conn.execute(
        &sql,
        rusqlite::params_from_iter(PRESERVED_SETTING_KEYS.iter()),
    ) {
        Ok(_) => conn.execute_batch("COMMIT;"),
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

#[tauri::command]
pub fn reset_app_settings(
    confirm: String,
    pin: Option<String>,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<BackupResultDto, AppError> {
    crate::errors::require_main_window(&window)?;

    if confirm.trim() != "RESET_APP_SETTINGS" {
        return Err(AppError::new(
            "RESET_SETTINGS_CONFIRM_REQUIRED",
            "Confirmation text 'RESET_APP_SETTINGS' is required to perform settings reset",
        ));
    }

    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    // Check PIN requirement: if admin PIN is configured, verify the passed PIN or existing unlocked session
    let pin_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'admin_pin_hash';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .ok()
        .filter(|s: &String| !s.trim().is_empty());

    if let Some(ref stored_hash) = pin_hash {
        let is_session_unlocked: bool = pin_state.is_session_valid();
        if !is_session_unlocked {
            pin_state.check_rate_limit()?;
            let provided_pin: &str = pin.as_deref().unwrap_or("");
            if !crate::services::admin_pin::verify_pin_hash(provided_pin, stored_hash)? {
                pin_state.record_failure();
                return Err(AppError::new(
                    "ADMIN_PIN_INVALID",
                    "Admin PIN is incorrect for settings reset",
                ));
            }
        }
    }

    // 1. Create an automatic pre-reset backup in app_data_dir/backups
    let app_data_dir: std::path::PathBuf =
        app.path().app_data_dir().map_err(|e: tauri::Error| {
            AppError::internal_error(format!("Failed to get app data dir: {}", e))
        })?;

    let backup_dto: BackupResultDto = create_backup_archive_in_dir(None, &app_data_dir, &conn)?;

    // 2. Reset the preference rows inside a transaction; internal state is kept.
    reset_preference_rows(&conn).map_err(AppError::database_error)?;

    // Clear unlocked PIN session
    pin_state.clear_session();

    Ok(backup_dto)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection_with_settings() -> rusqlite::Connection {
        let conn: rusqlite::Connection = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO app_settings VALUES ('language', 'en');
             INSERT INTO app_settings VALUES ('theme', 'light');
             INSERT INTO app_settings VALUES ('ticket_sequence_counter', '50');
             INSERT INTO app_settings VALUES ('admin_pin_hash', 'hash');
             INSERT INTO app_settings VALUES ('active_game_format', 'SONG_GRID');
             INSERT INTO app_settings VALUES ('active_song_count', '67');",
        )
        .unwrap();
        conn
    }

    fn keys(conn: &rusqlite::Connection) -> Vec<String> {
        let mut statement = conn
            .prepare("SELECT key FROM app_settings ORDER BY key;")
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|row| row.unwrap())
            .collect()
    }

    #[test]
    fn settings_reset_removes_preferences_but_keeps_counters_format_and_pin() {
        let conn = connection_with_settings();
        reset_preference_rows(&conn).unwrap();
        assert_eq!(
            keys(&conn),
            vec![
                "active_game_format",
                "active_song_count",
                "admin_pin_hash",
                "ticket_sequence_counter"
            ]
        );
    }

    #[test]
    fn settings_reset_rolls_back_when_it_fails() {
        let conn = connection_with_settings();
        conn.execute_batch(
            "CREATE TRIGGER block_delete BEFORE DELETE ON app_settings
             WHEN old.key = 'theme' BEGIN SELECT RAISE(ABORT, 'blocked'); END;",
        )
        .unwrap();
        assert!(reset_preference_rows(&conn).is_err());
        assert_eq!(keys(&conn).len(), 6);
    }
}
