use crate::db::repositories::claim::ClaimRepository;
use crate::db::DbState;
use crate::domain::claims::ClaimResult;
use crate::errors::AppError;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

#[tauri::command]
pub fn submit_claim(
    game_id: i64,
    scheme_id: i64,
    ticket_code: String,
    state: State<'_, DbState>,
) -> Result<ClaimResult, AppError> {
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    ClaimRepository::submit_claim(&mut conn, game_id, scheme_id, &ticket_code)
        .map_err(AppError::database_error)
}

#[tauri::command]
pub fn list_game_claims(
    game_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<ClaimResult>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    ClaimRepository::list_game_claims(&conn, game_id).map_err(AppError::database_error)
}
