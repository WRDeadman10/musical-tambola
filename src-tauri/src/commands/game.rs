use crate::commands::media::validate_all_media;
use crate::db::repositories::game::GameRepository;
use crate::db::DbState;
use crate::domain::game::{GameState, GameSummary};
use crate::domain::media::MediaValidationReport;
use crate::domain::random::OsRngSource;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::{AppHandle, State, Window};

#[tauri::command]
pub async fn preflight_media(
    app_handle: AppHandle,
    state: State<'_, DbState>,
) -> Result<MediaValidationReport, String> {
    validate_all_media(app_handle, state).await
}

#[tauri::command]
pub fn create_game(batch_id: i64, state: State<'_, DbState>) -> Result<GameState, String> {
    let mut conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    let mut rng: OsRngSource = OsRngSource;
    GameRepository::create_game(&mut conn, batch_id, &mut rng)
}

#[tauri::command]
pub fn get_active_game(state: State<'_, DbState>) -> Result<Option<GameState>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    GameRepository::get_active_game(&conn)
}

#[tauri::command]
pub fn get_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    GameRepository::get_game(&conn, game_id)
}

#[tauri::command]
pub fn draw_next(game_id: i64, state: State<'_, DbState>) -> Result<GameState, String> {
    let mut conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    GameRepository::draw_next(&mut conn, game_id)
}

#[tauri::command]
pub fn pause_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, String> {
    let mut conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    GameRepository::pause_game(&mut conn, game_id)
}

#[tauri::command]
pub fn resume_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, String> {
    let mut conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    GameRepository::resume_game(&mut conn, game_id)
}

#[tauri::command]
pub fn end_game(game_id: i64, state: State<'_, DbState>) -> Result<GameState, String> {
    let mut conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    GameRepository::end_game(&mut conn, game_id)
}

#[tauri::command]
pub fn list_games(
    limit: Option<u32>,
    state: State<'_, DbState>,
) -> Result<Vec<GameSummary>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    let effective_limit: u32 = limit.unwrap_or(50);
    GameRepository::list_games(&conn, effective_limit)
}

#[tauri::command]
pub fn set_fullscreen(window: Window, fullscreen: bool) -> Result<(), String> {
    window
        .set_fullscreen(fullscreen)
        .map_err(|e| format!("Failed to set fullscreen: {}", e))
}
