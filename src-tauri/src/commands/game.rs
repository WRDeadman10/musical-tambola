use crate::commands::media::validate_all_media;
use crate::db::repositories::game::GameRepository;
use crate::db::DbState;
use crate::domain::game::{GameState, GameSummary};
use crate::domain::media::MediaValidationReport;
use crate::domain::random::OsRngSource;
use crate::errors::AppError;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::{AppHandle, State, Window};

#[tauri::command]
pub async fn preflight_media(
    app_handle: AppHandle,
    state: State<'_, DbState>,
) -> Result<MediaValidationReport, AppError> {
    validate_all_media(app_handle, state).await
}

#[tauri::command]
pub fn create_game(batch_id: i64, state: State<'_, DbState>) -> Result<GameState, AppError> {
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    let mut rng: OsRngSource = OsRngSource;
    GameRepository::create_game(&mut conn, batch_id, &mut rng).map_err(|e| {
        if e.contains("Cannot create a new game while another is ACTIVE or PAUSED") {
            AppError::new(
                "GAME_ALREADY_ACTIVE",
                "Another game session is already active or paused",
            )
        } else if e.contains("Cannot start game: only") {
            // parse valid count
            let parts: Vec<&str> = e.split_whitespace().collect();
            let count_str = parts.get(4).unwrap_or(&"0");
            AppError::new(
                "GAME_MEDIA_INVALID",
                format!(
                    "Cannot start game: fewer than 90 media items are validated (only {} valid)",
                    count_str
                ),
            )
            .with_param("validCount", count_str.to_string())
        } else if e.contains("Cannot start game without at least one active prize scheme") {
            AppError::new(
                "GAME_NO_ACTIVE_SCHEMES",
                "Cannot start game: no active prize schemes configured",
            )
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

#[tauri::command]
pub fn get_active_game(state: State<'_, DbState>) -> Result<Option<GameState>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::get_active_game(&conn).map_err(AppError::database_error)
}

#[tauri::command]
pub fn get_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::get_game(&conn, game_id).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn draw_next(
    game_id: i64,
    expected_position: u32,
    state: State<'_, DbState>,
) -> Result<GameState, AppError> {
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::draw_next(&mut conn, game_id, expected_position).map_err(|e| {
        if e.contains("All 90 numbers have already been drawn") {
            AppError::new(
                "GAME_ALREADY_COMPLETED",
                "All 90 numbers have already been drawn for this game session",
            )
        } else if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else if e.contains("not active") {
            AppError::new(
                "GAME_NOT_ACTIVE",
                "There is no active game session to perform this action",
            )
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn pause_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, AppError> {
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::pause_game(&mut conn, game_id).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn resume_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, AppError> {
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::resume_game(&mut conn, game_id).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn end_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, AppError> {
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::end_game(&mut conn, game_id).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn list_games(state: State<'_, DbState>) -> Result<Vec<GameSummary>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::list_games(&conn, 50).map_err(AppError::database_error)
}

#[tauri::command]
pub fn set_fullscreen(fullscreen: bool, window: Window) -> Result<(), AppError> {
    window.set_fullscreen(fullscreen).map_err(|e| {
        AppError::with_detail(
            "UNKNOWN_ERROR",
            "Failed to change fullscreen mode",
            e.to_string(),
        )
    })
}
