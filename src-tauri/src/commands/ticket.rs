use crate::db::repositories::ticket::TicketRepository;
use crate::db::DbState;
use crate::domain::random::OsRngSource;
use crate::domain::tambola::{TicketBatch, TicketDto, TicketPage};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::MutexGuard;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationProgressPayload {
    done: usize,
    total: usize,
}

#[tauri::command]
pub async fn generate_ticket_batch(
    name: String,
    count: u32,
    app_handle: AppHandle,
) -> Result<TicketBatch, String> {
    let app: AppHandle = app_handle.clone();
    tokio::task::spawn_blocking(move || {
        let state: State<'_, DbState> = app.state::<DbState>();
        let mut conn: MutexGuard<'_, Connection> = state
            .conn
            .lock()
            .map_err(|_| "Failed to acquire database lock".to_string())?;

        let mut rng: OsRngSource = OsRngSource::new();
        TicketRepository::generate_batch(
            &mut conn,
            &name,
            count,
            &mut rng,
            |done: usize, total: usize| {
                let _ = app.emit(
                    "ticket-generation-progress",
                    GenerationProgressPayload { done, total },
                );
            },
        )
    })
    .await
    .map_err(|e: tokio::task::JoinError| format!("Task join failed: {}", e))?
}

#[tauri::command]
pub fn list_ticket_batches(state: State<'_, DbState>) -> Result<Vec<TicketBatch>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    TicketRepository::list_batches(&conn)
}

#[tauri::command]
pub fn list_batch_tickets(
    batch_id: i64,
    offset: u32,
    limit: u32,
    state: State<'_, DbState>,
) -> Result<TicketPage, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    TicketRepository::list_batch_tickets(&conn, batch_id, offset, limit)
}

#[tauri::command]
pub fn get_ticket(code: String, state: State<'_, DbState>) -> Result<TicketDto, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    TicketRepository::get_ticket(&conn, &code)
}

#[tauri::command]
pub fn search_tickets(
    query: String,
    limit: u32,
    state: State<'_, DbState>,
) -> Result<Vec<TicketDto>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    TicketRepository::search_tickets(&conn, &query, limit)
}

#[tauri::command]
pub fn delete_ticket_batch(batch_id: i64, state: State<'_, DbState>) -> Result<(), String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    TicketRepository::delete_batch(&conn, batch_id)
}
