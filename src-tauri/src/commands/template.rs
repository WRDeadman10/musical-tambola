use crate::db::repositories::template::{TemplateRepository, TicketTemplateDto};
use crate::db::DbState;
use crate::errors::AppError;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;
use tauri::{AppHandle, Manager, State, Window};

const MAX_TEMPLATE_SIZE_BYTES: usize = 5 * 1024 * 1024; // 5 MB

pub fn validate_template_image_header(bytes: &[u8]) -> Result<(&'static str, u32, u32), AppError> {
    if bytes.is_empty() {
        return Err(AppError::new(
            "TEMPLATE_EMPTY_FILE",
            "Template background data cannot be empty",
        ));
    }

    if bytes.len() > MAX_TEMPLATE_SIZE_BYTES {
        return Err(AppError::new(
            "TEMPLATE_SIZE_LIMIT_EXCEEDED",
            format!(
                "Template background exceeds 5 MB limit (received {} bytes)",
                bytes.len()
            ),
        ));
    }

    let is_png: bool = bytes.len() >= 8
        && bytes[0] == 0x89
        && bytes[1] == 0x50
        && bytes[2] == 0x4E
        && bytes[3] == 0x47
        && bytes[4] == 0x0D
        && bytes[5] == 0x0A
        && bytes[6] == 0x1A
        && bytes[7] == 0x0A;

    let is_jpeg: bool =
        bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF;

    let ext: &'static str = if is_png {
        "png"
    } else if is_jpeg {
        "jpg"
    } else {
        return Err(AppError::new(
            "TEMPLATE_INVALID_FORMAT",
            "Only PNG and JPEG formats are supported",
        ));
    };

    let reader: image::ImageReader<Cursor<&[u8]>> = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| {
            AppError::new(
                "TEMPLATE_INVALID_FORMAT",
                format!("Failed to parse image header: {}", e),
            )
        })?;

    let (width, height): (u32, u32) = reader.into_dimensions().map_err(|e| {
        AppError::new(
            "TEMPLATE_INVALID_FORMAT",
            format!("Failed to read image dimensions: {}", e),
        )
    })?;

    if height == 0 {
        return Err(AppError::new(
            "TEMPLATE_INVALID_FORMAT",
            "Image height cannot be zero",
        ));
    }

    Ok((ext, width, height))
}

#[tauri::command]
pub fn list_templates(state: State<'_, DbState>) -> Result<Vec<TicketTemplateDto>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    TemplateRepository::list_all(&conn).map_err(AppError::database_error)
}

#[tauri::command]
pub fn get_active_template(
    state: State<'_, DbState>,
) -> Result<Option<TicketTemplateDto>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    TemplateRepository::get_active(&conn).map_err(AppError::database_error)
}

#[tauri::command]
pub fn upload_template_background(
    name: String,
    bytes: Vec<u8>,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<TicketTemplateDto, AppError> {
    crate::errors::require_main_window(&window)?;

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;

    let trimmed_name: String = name.trim().to_string();
    if trimmed_name.is_empty() {
        return Err(AppError::new(
            "TEMPLATE_EMPTY_NAME",
            "Template name cannot be empty",
        ));
    }

    if trimmed_name.len() > 100 {
        return Err(AppError::new(
            "TEMPLATE_NAME_TOO_LONG",
            "Template name must be 100 characters or less",
        ));
    }

    let (ext, _width, _height) = validate_template_image_header(&bytes)?;

    let mut hasher: Sha256 = Sha256::new();
    hasher.update(&bytes);
    let hash_hex: String = format!("{:x}", hasher.finalize());
    let sha16: &str = &hash_hex[0..16];

    let app_data_dir: PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal_error(format!("Failed to get app data dir: {}", e)))?;
    let templates_dir: PathBuf = app_data_dir.join("templates");
    std::fs::create_dir_all(&templates_dir).map_err(|e| {
        AppError::internal_error(format!("Failed to create templates directory: {}", e))
    })?;

    let template_file: PathBuf = templates_dir.join(format!("template_{}.{}", sha16, ext));
    std::fs::write(&template_file, &bytes)
        .map_err(|e| AppError::internal_error(format!("Failed to write template file: {}", e)))?;

    let file_path_str: String = template_file.to_string_lossy().to_string();

    TemplateRepository::insert_template(&conn, &trimmed_name, &file_path_str)
}

#[tauri::command]
pub fn set_active_template(
    id: i64,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<TicketTemplateDto, AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    TemplateRepository::set_active(&conn, id)
}

#[tauri::command]
pub fn delete_template(
    id: i64,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<(), AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    let path_to_clean: Option<String> = TemplateRepository::delete_template(&conn, id)?;

    if let Some(p) = path_to_clean {
        let path = Path::new(&p);
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_png(width: u32, height: u32) -> Vec<u8> {
        let mut img: image::RgbaImage = image::RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([128, 64, 200, 255]);
        }
        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png).unwrap();
        buffer.into_inner()
    }

    #[test]
    fn test_validate_template_accepts_any_aspect_ratio() {
        // Banner pieces, backgrounds and logos come in every shape.
        for (width, height) in [(1800, 900), (1600, 900), (600, 600), (300, 1200)] {
            let bytes = create_test_png(width, height);
            let res = validate_template_image_header(&bytes).unwrap();
            assert_eq!(res.0, "png");
            assert_eq!((res.1, res.2), (width, height));
        }
    }

    #[test]
    fn test_validate_template_rejects_empty_and_oversized() {
        let empty_err = validate_template_image_header(&[]).unwrap_err();
        assert_eq!(empty_err.code, "TEMPLATE_EMPTY_FILE");

        let oversized: Vec<u8> = vec![0u8; 5 * 1024 * 1024 + 1];
        let over_err = validate_template_image_header(&oversized).unwrap_err();
        assert_eq!(over_err.code, "TEMPLATE_SIZE_LIMIT_EXCEEDED");
    }

    #[test]
    fn test_validate_template_rejects_non_image() {
        let dummy_text = b"Hello, this is just plain text content";
        let text_err = validate_template_image_header(dummy_text).unwrap_err();
        assert_eq!(text_err.code, "TEMPLATE_INVALID_FORMAT");
    }
}
