use crate::db::repositories::ticket::TicketRepository;
use crate::db::DbState;
use crate::domain::format::GameFormat;
use crate::domain::random::OsRngSource;
use crate::domain::tambola::{TicketBatch, TicketDto, TicketPage};
use crate::errors::AppError;
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
    format: Option<GameFormat>,
    song_count: Option<u32>,
    window: tauri::Window,
    app_handle: AppHandle,
) -> Result<TicketBatch, AppError> {
    crate::errors::require_main_window(&window)?;
    if !(1..=10_000).contains(&count) {
        return Err(AppError::new(
            "BATCH_COUNT_INVALID",
            format!(
                "Ticket batch count must be between 1 and 10000, got {}",
                count
            ),
        )
        .with_param("count", count.to_string()));
    }

    let app: AppHandle = app_handle.clone();
    tokio::task::spawn_blocking(move || {
        let state: State<'_, DbState> = app.state::<DbState>();
        let mut conn: MutexGuard<'_, Connection> =
            state.conn.lock().map_err(|_| AppError::lock_error())?;
        if let Some(pin_state) = app.try_state::<crate::services::admin_pin::AdminPinState>() {
            crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
        }

        let mut rng: OsRngSource = OsRngSource::new();
        TicketRepository::generate_batch(
            &mut conn,
            &name,
            count,
            format,
            song_count,
            &mut rng,
            |done: usize, total: usize| {
                let _ = app.emit(
                    "ticket-generation-progress",
                    GenerationProgressPayload { done, total },
                );
            },
        )
        .map_err(AppError::database_error)
    })
    .await
    .map_err(|e: tokio::task::JoinError| {
        AppError::with_detail("UNKNOWN_ERROR", "Task execution failed", e.to_string())
    })?
}

#[tauri::command]
pub fn list_ticket_batches(state: State<'_, DbState>) -> Result<Vec<TicketBatch>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    TicketRepository::list_batches(&conn).map_err(AppError::database_error)
}

#[tauri::command]
pub fn list_batch_tickets(
    batch_id: i64,
    offset: u32,
    limit: u32,
    state: State<'_, DbState>,
) -> Result<TicketPage, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    TicketRepository::list_batch_tickets(&conn, batch_id, offset, limit)
        .map_err(AppError::database_error)
}

#[tauri::command]
pub fn get_ticket(code: String, state: State<'_, DbState>) -> Result<TicketDto, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    TicketRepository::get_ticket(&conn, &code).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "TICKET_NOT_FOUND",
                format!("Ticket was not found: {}", code),
            )
            .with_param("code", code)
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn search_tickets(
    query: String,
    limit: u32,
    state: State<'_, DbState>,
) -> Result<Vec<TicketDto>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    TicketRepository::search_tickets(&conn, &query, limit).map_err(AppError::database_error)
}

#[tauri::command]
pub fn delete_ticket_batch(
    batch_id: i64,
    window: tauri::Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<(), AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;

    TicketRepository::delete_batch(&conn, batch_id).map_err(|e| {
        if e.contains("Cannot delete ticket batch") || e.contains("assigned to") {
            AppError::new(
                "BATCH_IN_USE",
                "Cannot delete ticket batch because it is assigned to an existing game or claim"
                    .to_string(),
            )
            .with_param("batchId", batch_id.to_string())
        } else if e.contains("not found") {
            AppError::new(
                "BATCH_NOT_FOUND",
                format!("Ticket batch was not found: {}", batch_id),
            )
            .with_param("batchId", batch_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })
}
