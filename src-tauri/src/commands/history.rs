use crate::db::repositories::history::HistoryRepository;
use crate::domain::history::{ClaimHistoryRow, DashboardStats, HistoryFilter, Page, WinnerRow};
use crate::errors::AppError;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

use crate::db::connection::DbState;

#[tauri::command]
pub fn list_claim_history(
    filter: HistoryFilter,
    state: State<'_, DbState>,
) -> Result<Page<ClaimHistoryRow>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    HistoryRepository::list_claim_history(&conn, &filter).map_err(AppError::database_error)
}

#[tauri::command]
pub fn list_winners(
    filter: HistoryFilter,
    state: State<'_, DbState>,
) -> Result<Page<WinnerRow>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    HistoryRepository::list_winners(&conn, &filter).map_err(AppError::database_error)
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, DbState>) -> Result<DashboardStats, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    HistoryRepository::get_dashboard_stats(&conn).map_err(AppError::database_error)
}
