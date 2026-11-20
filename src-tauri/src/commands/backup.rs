use crate::db::DbState;
use crate::errors::AppError;
use crate::services::admin_pin::AdminPinState;
use crate::services::backup::{
    create_backup_archive, restore_backup_archive, BackupResultDto, RestoreResultDto,
};
use tauri::{AppHandle, State, Window};

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
