use crate::commands::media::validate_all_media;
use crate::db::repositories::game::GameRepository;
use crate::db::DbState;
use crate::domain::format::{GameFormat, GameFormatConfig, SetGameFormatInput};
use crate::domain::game::{GameState, GameSummary};
use crate::domain::media::MediaValidationReport;
use crate::domain::random::OsRngSource;
use crate::errors::AppError;
use rusqlite::{params, Connection};
use std::sync::MutexGuard;
use tauri::{AppHandle, Emitter, Manager, State, Window};

pub fn handle_multimonitor_presentation_on_game_start(
    app: &AppHandle,
    window: &Window,
    state: &State<'_, DbState>,
) {
    if let Ok(monitors) = app.available_monitors() {
        if monitors.len() >= 2 && app.get_webview_window("presentation").is_none() {
            let auto_open: bool = {
                if let Ok(conn) = state.conn.lock() {
                    conn.query_row(
                        "SELECT value FROM app_settings WHERE key = 'auto_open_presentation';",
                        [],
                        |r| r.get::<_, String>(0),
                    )
                    .map(|v| v != "false")
                    .unwrap_or(true)
                } else {
                    true
                }
            };

            let primary_name: Option<String> = app
                .primary_monitor()
                .ok()
                .flatten()
                .and_then(|m: tauri::Monitor| m.name().cloned());

            let mut target_idx: usize = 1;
            for (idx, mon) in monitors.iter().enumerate() {
                let name = mon.name().cloned().unwrap_or_default();
                let is_primary = primary_name
                    .as_ref()
                    .map(|p| p == &name)
                    .unwrap_or(idx == 0);
                if !is_primary {
                    target_idx = idx;
                    break;
                }
            }

            if auto_open {
                let _ = crate::commands::presentation::open_presentation(
                    Some(target_idx),
                    window.clone(),
                    app.clone(),
                    state.clone(),
                );
            } else {
                let _ = app.emit(
                    "presentation-suggested",
                    serde_json::json!({ "displayIndex": target_idx }),
                );
            }
        }
    }
}

pub fn suggest_presentation_display_if_multimonitor(app: &AppHandle) {
    if let Ok(monitors) = app.available_monitors() {
        if monitors.len() >= 2 && app.get_webview_window("presentation").is_none() {
            let primary_name: Option<String> = app
                .primary_monitor()
                .ok()
                .flatten()
                .and_then(|m: tauri::Monitor| m.name().cloned());

            let mut suggested_idx: usize = 1;
            for (idx, mon) in monitors.iter().enumerate() {
                let name = mon.name().cloned().unwrap_or_default();
                let is_primary = primary_name
                    .as_ref()
                    .map(|p| p == &name)
                    .unwrap_or(idx == 0);
                if !is_primary {
                    suggested_idx = idx;
                    break;
                }
            }

            let _ = app.emit(
                "presentation-suggested",
                serde_json::json!({ "displayIndex": suggested_idx }),
            );
        }
    }
}

#[tauri::command]
pub async fn preflight_media(
    app_handle: AppHandle,
    window: Window,
    state: State<'_, DbState>,
) -> Result<MediaValidationReport, AppError> {
    crate::errors::require_main_window(&window)?;
    validate_all_media(app_handle, window, state).await
}

#[tauri::command]
pub fn create_game(
    batch_id: i64,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<GameState, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    let mut rng: OsRngSource = OsRngSource;
    let res: GameState =
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
            } else if e.contains("BATCH_FORMAT_MISMATCH") {
                AppError::new(
                    "BATCH_FORMAT_MISMATCH",
                    "Selected ticket batch format or song count does not match active game format",
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
        })?;

    handle_multimonitor_presentation_on_game_start(&app, &window, &state);
    let _ = app.emit("game-state-changed", &res);
    Ok(res)
}

#[tauri::command]
pub fn get_active_game(
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Option<GameState>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    let res: Option<GameState> =
        GameRepository::get_active_game(&conn).map_err(AppError::database_error)?;
    if res.is_some() {
        suggest_presentation_display_if_multimonitor(&app);
    }
    Ok(res)
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
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<GameState, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    let res: GameState =
        GameRepository::draw_next(&mut conn, game_id, expected_position).map_err(|e| {
            if e.contains("All 90 numbers have already been drawn") {
                AppError::new(
                    "GAME_ALREADY_COMPLETED",
                    "All 90 numbers have already been drawn for this game session",
                )
            } else if e.contains("BATCH_FORMAT_MISMATCH") {
                AppError::new(
                    "BATCH_FORMAT_MISMATCH",
                    "Selected ticket batch format or song count does not match active game format",
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
        })?;

    let _ = app.emit("game-state-changed", &res);
    Ok(res)
}

#[tauri::command]
pub fn pause_game(
    game_id: i64,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<GameState, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    let res: GameState = GameRepository::pause_game(&mut conn, game_id).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })?;

    let _ = app.emit("game-state-changed", &res);
    Ok(res)
}

#[tauri::command]
pub fn resume_game(
    game_id: i64,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<GameState, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    let res: GameState = GameRepository::resume_game(&mut conn, game_id).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })?;

    let _ = app.emit("game-state-changed", &res);
    Ok(res)
}

#[tauri::command]
pub fn end_game(
    game_id: i64,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<GameState, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;

    let res: GameState = GameRepository::end_game(&mut conn, game_id).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "GAME_NOT_FOUND",
                format!("Game session was not found: {}", game_id),
            )
            .with_param("gameId", game_id.to_string())
        } else {
            AppError::database_error(e)
        }
    })?;

    let _ = app.emit("game-state-changed", &res);
    Ok(res)
}

#[tauri::command]
pub fn list_games(state: State<'_, DbState>) -> Result<Vec<GameSummary>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    GameRepository::list_games(&conn, 50).map_err(AppError::database_error)
}

#[tauri::command]
pub fn set_fullscreen(fullscreen: bool, window: Window) -> Result<(), AppError> {
    crate::errors::require_main_window(&window)?;
    window.set_fullscreen(fullscreen).map_err(|e| {
        AppError::with_detail(
            "UNKNOWN_ERROR",
            "Failed to change fullscreen mode",
            e.to_string(),
        )
    })
}

#[tauri::command]
pub fn get_game_format(state: State<'_, DbState>) -> Result<GameFormatConfig, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    let format_str: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'active_game_format';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "CLASSIC_90".to_string());

    let format: GameFormat = GameFormat::from_db_str(&format_str).unwrap_or(GameFormat::Classic90);

    let song_count: u32 = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'active_song_count';",
            [],
            |r| r.get(0),
        )
        .ok()
        .and_then(|s: String| s.parse::<u32>().ok())
        .unwrap_or_else(|| format.default_song_count());

    Ok(GameFormatConfig {
        format,
        song_count,
        rows: format.rows(),
        cols: format.cols(),
    })
}

#[tauri::command]
pub fn set_game_format(
    input: SetGameFormatInput,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<GameFormatConfig, AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;

    // Validate that no game is ACTIVE or PAUSED
    let active_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status IN ('ACTIVE', 'PAUSED');",
            [],
            |r| r.get(0),
        )
        .map_err(AppError::database_error)?;

    if active_count > 0 {
        return Err(AppError::new(
            "FORMAT_LOCKED_GAME_ACTIVE",
            "Cannot change game format while a game is active or paused",
        ));
    }

    let chosen_song_count: u32 = input
        .song_count
        .unwrap_or_else(|| input.format.default_song_count());

    let valid_song_count: u32 = input
        .format
        .validate_song_count(chosen_song_count)
        .map_err(|msg| AppError::new("SONG_COUNT_INVALID", msg))?;

    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES ('active_game_format', ?1, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
        params![input.format.as_str()],
    )
    .map_err(AppError::database_error)?;

    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES ('active_song_count', ?1, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
        params![valid_song_count.to_string()],
    )
    .map_err(AppError::database_error)?;

    Ok(GameFormatConfig {
        format: input.format,
        song_count: valid_song_count,
        rows: input.format.rows(),
        cols: input.format.cols(),
    })
}
