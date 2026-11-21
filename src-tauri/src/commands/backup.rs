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

    // 2. Wipe all tables safely in a single transaction
    conn.execute_batch(
        "PRAGMA foreign_keys = OFF;
         DELETE FROM claims;
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
         DELETE FROM schemes;
         DELETE FROM app_settings;
         DELETE FROM schema_migrations;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(AppError::database_error)?;

    // 3. Re-run migrations fresh to seed default schemes and initial baseline schema
    crate::db::migrations::run_migrations(&conn).map_err(|e: String| {
        AppError::internal_error(format!("Failed to re-run migrations: {}", e))
    })?;

    // Clear unlocked PIN session
    pin_state.clear_session();

    Ok(backup_dto)
}
