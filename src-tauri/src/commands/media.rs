use crate::db::repositories::media::MediaRepository;
use crate::db::DbState;
use crate::domain::media::{MediaSlot, MediaValidationReport, SaveMediaInput, ValidationStatus};
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
pub fn list_media(state: State<'_, DbState>) -> Result<Vec<MediaSlot>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    MediaRepository::list_all(&conn)
}

#[tauri::command]
pub fn get_media(number: u8, state: State<'_, DbState>) -> Result<MediaSlot, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    MediaRepository::get_by_number(&conn, number)
}

#[tauri::command]
pub fn save_media(input: SaveMediaInput, state: State<'_, DbState>) -> Result<MediaSlot, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    MediaRepository::upsert(&conn, &input)
}

#[tauri::command]
pub fn clear_media(number: u8, state: State<'_, DbState>) -> Result<MediaSlot, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    MediaRepository::clear(&conn, number)
}

#[tauri::command]
pub async fn validate_media(number: u8, state: State<'_, DbState>) -> Result<MediaSlot, String> {
    let slot: MediaSlot = {
        let conn: MutexGuard<'_, Connection> = state
            .conn
            .lock()
            .map_err(|_| "Failed to acquire database lock".to_string())?;
        MediaRepository::get_by_number(&conn, number)?
    };

    let (status, message, code, params_json): (
        ValidationStatus,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = validate_slot(&slot).await;

    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    MediaRepository::update_validation_status(
        &conn,
        number,
        status,
        message.as_deref(),
        code.as_deref(),
        params_json.as_deref(),
    )
}

#[tauri::command]
pub async fn validate_all_media(
    app_handle: AppHandle,
    state: State<'_, DbState>,
) -> Result<MediaValidationReport, String> {
    let slots: Vec<MediaSlot> = {
        let conn: MutexGuard<'_, Connection> = state
            .conn
            .lock()
            .map_err(|_| "Failed to acquire database lock".to_string())?;
        MediaRepository::list_all(&conn)?
    };

    let total: usize = slots.len();
    let mut updated_slots: Vec<MediaSlot> = Vec::with_capacity(total);

    // Limit concurrency to 4 simultaneous checks
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

            let conn: MutexGuard<'_, Connection> = state
                .conn
                .lock()
                .map_err(|_| "Failed to acquire database lock".to_string())?;

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
) -> Result<Option<String>, String> {
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
