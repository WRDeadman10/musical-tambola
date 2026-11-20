use crate::db::repositories::history::HistoryRepository;
use crate::domain::history::{ClaimHistoryRow, DashboardStats, HistoryFilter, Page, WinnerRow};
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

use crate::db::connection::DbState;

#[tauri::command]
pub fn list_claim_history(
    filter: HistoryFilter,
    state: State<'_, DbState>,
) -> Result<Page<ClaimHistoryRow>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    HistoryRepository::list_claim_history(&conn, &filter)
}

#[tauri::command]
pub fn list_winners(
    filter: HistoryFilter,
    state: State<'_, DbState>,
) -> Result<Page<WinnerRow>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    HistoryRepository::list_winners(&conn, &filter)
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, DbState>) -> Result<DashboardStats, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    HistoryRepository::get_dashboard_stats(&conn)
}
