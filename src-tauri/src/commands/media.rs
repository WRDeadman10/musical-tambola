use crate::db::repositories::media::MediaRepository;
use crate::db::DbState;
use crate::domain::format::GameFormat;
use crate::domain::media::{
    MediaAssignment, MediaFolderOrder, MediaFolderScanResult, MediaSlot, MediaValidationReport,
    SaveMediaInput, ValidationStatus,
};
use crate::errors::AppError;
use crate::services::media_validation::{summarize_report, validate_slot};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, MutexGuard};
use tauri::{AppHandle, Emitter, Manager, State, Window};
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
pub fn save_media(
    input: SaveMediaInput,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<MediaSlot, AppError> {
    crate::errors::require_main_window(&window)?;
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
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;

    MediaRepository::upsert(&conn, &input).map_err(AppError::database_error)
}

#[tauri::command]
pub fn clear_media(
    number: u8,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<MediaSlot, AppError> {
    crate::errors::require_main_window(&window)?;
    if !(1..=90).contains(&number) {
        return Err(AppError::new(
            "MEDIA_SLOT_OUT_OF_RANGE",
            format!("Media slot number must be between 1 and 90, got {}", number),
        )
        .with_param("number", number.to_string()));
    }

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;

    MediaRepository::clear(&conn, number).map_err(AppError::database_error)
}

#[tauri::command]
pub async fn validate_media(
    number: u8,
    window: Window,
    state: State<'_, DbState>,
) -> Result<MediaSlot, AppError> {
    crate::errors::require_main_window(&window)?;
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
    window: Window,
    state: State<'_, DbState>,
) -> Result<MediaValidationReport, AppError> {
    crate::errors::require_main_window(&window)?;
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
    window: Window,
) -> Result<Option<String>, AppError> {
    crate::errors::require_main_window(&window)?;
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

#[tauri::command]
pub async fn pick_media_folder(
    app_handle: AppHandle,
    window: Window,
) -> Result<Option<String>, AppError> {
    crate::errors::require_main_window(&window)?;
    let folder_path: Option<tauri_plugin_dialog::FilePath> =
        app_handle.dialog().file().blocking_pick_folder();
    Ok(folder_path.map(|p: tauri_plugin_dialog::FilePath| match p {
        tauri_plugin_dialog::FilePath::Path(path_buf) => path_buf.to_string_lossy().to_string(),
        tauri_plugin_dialog::FilePath::Url(url) => url.to_string(),
    }))
}

#[tauri::command]
pub fn scan_media_folder(
    root_path: String,
    order: MediaFolderOrder,
    recursive: bool,
    seed: Option<u64>,
    window: Window,
    state: State<'_, DbState>,
) -> Result<MediaFolderScanResult, AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    let active_format_str: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'active_game_format';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "CLASSIC_90".to_string());
    let active_format: GameFormat =
        GameFormat::from_db_str(&active_format_str).unwrap_or(GameFormat::Classic90);

    let song_count: u32 = match active_format {
        GameFormat::Classic90 => 90,
        GameFormat::SongGrid => conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_song_count';",
                [],
                |r| r.get(0),
            )
            .ok()
            .and_then(|s: String| s.parse::<u32>().ok())
            .unwrap_or(45),
    };

    drop(conn);

    let path = std::path::Path::new(&root_path);
    crate::services::media_folder::scan_media_folder(path, order, recursive, seed, song_count)
        .map_err(|e| AppError::new("FOLDER_SCAN_ERROR", e))
}

#[tauri::command]
pub fn apply_media_assignments(
    assignments: Vec<MediaAssignment>,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<Vec<MediaSlot>, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    MediaRepository::apply_assignments(&mut conn, &assignments).map_err(AppError::database_error)
}

fn cleanup_old_poster_if_unreferenced(conn: &Connection, old_path_str: Option<&str>) {
    if let Some(old_path) = old_path_str {
        let trimmed: &str = old_path.trim();
        if trimmed.is_empty() {
            return;
        }
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM game_media_snapshots WHERE poster_path = ?1;",
                params![trimmed],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if count == 0 {
            let p: &Path = Path::new(trimmed);
            if p.exists() {
                let _ = std::fs::remove_file(p);
            }
        }
    }
}

#[tauri::command]
pub fn save_media_poster(
    number: u8,
    image_bytes: Vec<u8>,
    app_handle: AppHandle,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<MediaSlot, AppError> {
    crate::errors::require_main_window(&window)?;
    if !(1..=90).contains(&number) {
        return Err(AppError::new(
            "MEDIA_SLOT_OUT_OF_RANGE",
            format!("Media slot number must be between 1 and 90, got {}", number),
        ));
    }

    if image_bytes.len() > 2 * 1024 * 1024 {
        return Err(AppError::new(
            "POSTER_TOO_LARGE",
            format!(
                "Poster size ({} bytes) exceeds maximum allowed limit of 2 MB",
                image_bytes.len()
            ),
        ));
    }

    if image_bytes.len() < 3
        || image_bytes[0] != 0xFF
        || image_bytes[1] != 0xD8
        || image_bytes[2] != 0xFF
    {
        return Err(AppError::new(
            "POSTER_INVALID_JPEG",
            "Poster must be a valid JPEG image (magic bytes FF D8 FF)",
        ));
    }

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&image_bytes);
    let hash_hex: String = format!("{:x}", hasher.finalize());
    let sha8: &str = &hash_hex[0..8];

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal_error(format!("Failed to get app data dir: {}", e)))?;
    let posters_dir = app_data_dir.join("posters");
    std::fs::create_dir_all(&posters_dir).map_err(|e| {
        AppError::internal_error(format!("Failed to create posters directory: {}", e))
    })?;

    let poster_file = posters_dir.join(format!("poster_{}_{}.jpg", number, sha8));
    std::fs::write(&poster_file, &image_bytes)
        .map_err(|e| AppError::internal_error(format!("Failed to write poster file: {}", e)))?;

    let poster_path_str: String = poster_file.to_string_lossy().to_string();

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;

    let old_poster_path: Option<String> = conn
        .query_row(
            "SELECT poster_path FROM media_assets WHERE number = ?1;",
            params![number],
            |r| r.get(0),
        )
        .unwrap_or(None);

    let result = MediaRepository::save_poster(&conn, number, &poster_path_str)
        .map_err(AppError::database_error)?;

    if let Some(old) = old_poster_path {
        if old != poster_path_str {
            cleanup_old_poster_if_unreferenced(&conn, Some(&old));
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn clear_media_poster(
    number: u8,
    _app_handle: AppHandle,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<MediaSlot, AppError> {
    crate::errors::require_main_window(&window)?;
    if !(1..=90).contains(&number) {
        return Err(AppError::new(
            "MEDIA_SLOT_OUT_OF_RANGE",
            format!("Media slot number must be between 1 and 90, got {}", number),
        ));
    }

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;

    let old_poster_path: Option<String> = conn
        .query_row(
            "SELECT poster_path FROM media_assets WHERE number = ?1;",
            params![number],
            |r| r.get(0),
        )
        .unwrap_or(None);

    cleanup_old_poster_if_unreferenced(&conn, old_poster_path.as_deref());

    MediaRepository::clear_poster(&conn, number).map_err(AppError::database_error)
}

#[tauri::command]
pub fn get_ffmpeg_status(
    app_handle: AppHandle,
) -> Result<crate::services::ffmpeg::FfmpegStatusDto, AppError> {
    Ok(crate::services::ffmpeg::get_ffmpeg_status_impl(&app_handle))
}

#[tauri::command]
pub async fn download_ffmpeg(
    window: Window,
    app_handle: AppHandle,
) -> Result<crate::services::ffmpeg::FfmpegStatusDto, AppError> {
    crate::errors::require_main_window(&window)?;
    crate::services::ffmpeg::download_ffmpeg_impl(&app_handle).await
}

#[tauri::command]
pub fn generate_media_posters(
    input: crate::services::ffmpeg::GeneratePostersInput,
    window: Window,
    app_handle: AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<crate::services::ffmpeg::GeneratePostersResultDto, AppError> {
    crate::errors::require_main_window(&window)?;
    {
        let conn: MutexGuard<'_, Connection> =
            state.conn.lock().map_err(|_| AppError::lock_error())?;
        crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    }

    crate::services::ffmpeg::generate_media_posters_impl(&state, &app_handle, input)
}
