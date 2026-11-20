use crate::db::DbState;
use crate::errors::AppError;
use crate::services::admin_pin::AdminPinState;
use crate::services::title_import::{
    apply_song_title_import_impl, preview_song_title_import_impl, ApplyTitleImportItemDto,
    TitleImportPreviewDto,
};
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::{State, Window};

#[tauri::command]
pub fn preview_song_title_import(
    bytes: Vec<u8>,
    filename: String,
    window: Window,
    state: State<'_, DbState>,
) -> Result<TitleImportPreviewDto, AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;
    preview_song_title_import_impl(&bytes, &filename, &conn)
}

#[tauri::command]
pub fn apply_song_title_import(
    items: Vec<ApplyTitleImportItemDto>,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<u32, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    apply_song_title_import_impl(&items, &mut conn)
}
