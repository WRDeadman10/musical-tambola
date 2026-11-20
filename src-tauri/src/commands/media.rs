use crate::db::repositories::media::MediaRepository;
use crate::db::DbState;
use crate::domain::format::GameFormat;
use crate::domain::media::{MediaSlot, MediaValidationReport, SaveMediaInput, ValidationStatus};
use crate::errors::AppError;
use crate::services::media_validation::{summarize_report, validate_slot};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::{Arc, MutexGuard};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    done: usize,
    total: usize,
}

type ValidationTaskResult = (
    u8,
    ValidationStatus,
    Option<String>,
    Option<String>,
    Option<String>,
);

#[tauri::command]
pub fn list_media(state: State<'_, DbState>) -> Result<Vec<MediaSlot>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    MediaRepository::list_all(&conn).map_err(AppError::database_error)
}

#[tauri::command]
pub fn get_media(number: u8, state: State<'_, DbState>) -> Result<MediaSlot, AppError> {
    if !(1..=90).contains(&number) {
        return Err(AppError::new(
            "MEDIA_SLOT_OUT_OF_RANGE",
            format!("Media slot number must be between 1 and 90, got {}", number),
        )
        .with_param("number", number.to_string()));
    }

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    MediaRepository::get_by_number(&conn, number).map_err(AppError::database_error)
}

#[tauri::command]
pub fn save_media(input: SaveMediaInput, state: State<'_, DbState>) -> Result<MediaSlot, AppError> {
    if !(1..=90).contains(&input.number) {
        return Err(AppError::new(
            "MEDIA_SLOT_OUT_OF_RANGE",
            format!(
                "Media slot number must be between 1 and 90, got {}",
                input.number
            ),
        )
        .with_param("number", input.number.to_string()));
    }
    if input.title.trim().is_empty() {
        return Err(AppError::new(
            "MEDIA_EMPTY_TITLE",
            "Media title cannot be empty",
        ));
    }
    if input.source.trim().is_empty() {
        return Err(AppError::new(
            "MEDIA_EMPTY_SOURCE",
            "Media source path or URL cannot be empty",
        ));
    }

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    MediaRepository::upsert(&conn, &input).map_err(AppError::database_error)
}

#[tauri::command]
pub fn clear_media(number: u8, state: State<'_, DbState>) -> Result<MediaSlot, AppError> {
    if !(1..=90).contains(&number) {
        return Err(AppError::new(
            "MEDIA_SLOT_OUT_OF_RANGE",
            format!("Media slot number must be between 1 and 90, got {}", number),
        )
        .with_param("number", number.to_string()));
    }

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    MediaRepository::clear(&conn, number).map_err(AppError::database_error)
}

#[tauri::command]
pub async fn validate_media(number: u8, state: State<'_, DbState>) -> Result<MediaSlot, AppError> {
    let slot: MediaSlot = {
        let conn: MutexGuard<'_, Connection> =
            state.conn.lock().map_err(|_| AppError::lock_error())?;
        MediaRepository::get_by_number(&conn, number).map_err(AppError::database_error)?
    };

    let (status, message, code, params_json): (
        ValidationStatus,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = validate_slot(&slot).await;

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    MediaRepository::update_validation_status(
        &conn,
        number,
        status,
        message.as_deref(),
        code.as_deref(),
        params_json.as_deref(),
    )
    .map_err(AppError::database_error)
}

#[tauri::command]
pub async fn validate_all_media(
    app_handle: AppHandle,
    state: State<'_, DbState>,
) -> Result<MediaValidationReport, AppError> {
    let slots: Vec<MediaSlot> = {
        let conn: MutexGuard<'_, Connection> =
            state.conn.lock().map_err(|_| AppError::lock_error())?;

        let format_str: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_game_format';",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "CLASSIC_90".to_string());
        let format: GameFormat =
            GameFormat::from_db_str(&format_str).unwrap_or(GameFormat::Classic90);

        let song_count: u32 = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_song_count';",
                [],
                |r| r.get(0),
            )
            .ok()
            .and_then(|s: String| s.parse::<u32>().ok())
            .unwrap_or_else(|| format.default_song_count());

        let all_slots: Vec<MediaSlot> =
            MediaRepository::list_all(&conn).map_err(AppError::database_error)?;
        all_slots
            .into_iter()
            .filter(|s: &MediaSlot| (s.number as u32) <= song_count)
            .collect()
    };

    let total: usize = slots.len();
    let mut updated_slots: Vec<MediaSlot> = Vec::with_capacity(total);

    let semaphore: Arc<tokio::sync::Semaphore> = Arc::new(tokio::sync::Semaphore::new(4));
    let mut handles: Vec<tokio::task::JoinHandle<ValidationTaskResult>> = Vec::with_capacity(total);

    for slot in slots {
        let sem: Arc<tokio::sync::Semaphore> = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let (status, msg, code, params_json) = validate_slot(&slot).await;
            (slot.number, status, msg, code, params_json)
        }));
    }

    let mut done_count: usize = 0;
    for handle in handles {
        if let Ok((number, status, msg, code, params_json)) = handle.await {
            done_count += 1;
            let _ = app_handle.emit(
                "media-validation-progress",
                ProgressPayload {
                    done: done_count,
                    total,
                },
            );

            let conn: MutexGuard<'_, Connection> =
                state.conn.lock().map_err(|_| AppError::lock_error())?;

            if let Ok(updated) = MediaRepository::update_validation_status(
                &conn,
                number,
                status,
                msg.as_deref(),
                code.as_deref(),
                params_json.as_deref(),
            ) {
                updated_slots.push(updated);
            }
        }
    }

    Ok(summarize_report(&updated_slots))
}

#[tauri::command]
pub async fn pick_media_file(
    kind: String,
    app_handle: AppHandle,
) -> Result<Option<String>, AppError> {
    let mut builder = app_handle.dialog().file();

    let extensions: Vec<&str> = match kind.to_ascii_uppercase().as_str() {
        "IMAGE" => vec!["jpg", "jpeg", "png", "webp", "gif"],
        "VIDEO" => vec!["mp4", "webm"],
        _ => vec!["jpg", "jpeg", "png", "webp", "gif", "mp4", "webm"],
    };

    builder = builder.add_filter("Media Files", &extensions);

    let file_path: Option<tauri_plugin_dialog::FilePath> = builder.blocking_pick_file();
    Ok(file_path.map(|p: tauri_plugin_dialog::FilePath| match p {
        tauri_plugin_dialog::FilePath::Path(path_buf) => path_buf.to_string_lossy().to_string(),
        tauri_plugin_dialog::FilePath::Url(url) => url.to_string(),
    }))
}
