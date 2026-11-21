use crate::db::migrations::get_schema_version;
use crate::db::DbState;
use crate::errors::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tauri::Manager;
use tauri::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSettingKey {
    Language,
    Theme,
    DefaultVideoVolume,
    VideoAutoplay,
    ShowInternalNumbers,
    UseGujaratiNumerals,
    PresentationDisplay,
    EventName,
    TickerText,
    IdlePoster,
    AccentColor,
    SoundEnabled,
    SoundVolume,
    SoundProjector,
    PresentationShowAnswer,
    PresentationShowNumber,
    AutoOpenPresentation,
    AnimationsEnabled,
}

impl FromStr for AppSettingKey {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "language" => Ok(Self::Language),
            "theme" => Ok(Self::Theme),
            "default_video_volume" => Ok(Self::DefaultVideoVolume),
            "video_autoplay" => Ok(Self::VideoAutoplay),
            "show_internal_numbers" => Ok(Self::ShowInternalNumbers),
            "use_gujarati_numerals" => Ok(Self::UseGujaratiNumerals),
            "presentation_display" => Ok(Self::PresentationDisplay),
            "event_name" => Ok(Self::EventName),
            "ticker_text" => Ok(Self::TickerText),
            "idle_poster" => Ok(Self::IdlePoster),
            "accent_color" => Ok(Self::AccentColor),
            "sound_enabled" => Ok(Self::SoundEnabled),
            "sound_volume" => Ok(Self::SoundVolume),
            "sound_projector" => Ok(Self::SoundProjector),
            "presentation_show_answer" => Ok(Self::PresentationShowAnswer),
            "presentation_show_number" => Ok(Self::PresentationShowNumber),
            "auto_open_presentation" => Ok(Self::AutoOpenPresentation),
            "animations_enabled" => Ok(Self::AnimationsEnabled),
            _ => {
                log::warn!("Rejected invalid app setting key: '{}'", s);
                Err(AppError::new(
                    "SETTING_NOT_ALLOWLISTED",
                    format!("Unsupported or unrecognized setting key: '{}'", s),
                )
                .with_param("key", s))
            }
        }
    }
}

pub fn validate_app_setting(key_str: &str, value: &str) -> Result<AppSettingKey, AppError> {
    if key_str.trim().is_empty() {
        log::warn!("Rejected empty setting key");
        return Err(AppError::new(
            "SETTING_NOT_ALLOWLISTED",
            "Setting key cannot be empty",
        ));
    }
    if key_str.len() > 256 {
        log::warn!("Rejected oversized setting key (length: {})", key_str.len());
        return Err(AppError::new(
            "SETTING_NOT_ALLOWLISTED",
            "Setting key exceeds maximum length of 256 characters",
        ));
    }

    let key: AppSettingKey = AppSettingKey::from_str(key_str)?;

    match key {
        AppSettingKey::EventName => {
            if value.len() > 200 {
                log::warn!("Rejected oversized event name (length: {})", value.len());
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Event name exceeds maximum length of 200 characters",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::TickerText => {
            if value.len() > 200 {
                log::warn!("Rejected oversized ticker text (length: {})", value.len());
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Ticker text exceeds maximum length of 200 characters",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::IdlePoster => {
            if value.len() > 256 {
                log::warn!(
                    "Rejected oversized idle poster path (length: {})",
                    value.len()
                );
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Idle poster path exceeds maximum length of 256 characters",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::AccentColor => {
            let is_valid_hex: bool = value.len() == 7
                && value.starts_with('#')
                && value[1..].chars().all(|c: char| c.is_ascii_hexdigit());
            if !is_valid_hex {
                log::warn!("Invalid accent color format: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Accent color must be a valid hex color code in #RRGGBB format",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::SoundEnabled => {
            if value != "true" && value != "false" {
                log::warn!("Invalid sound enabled value: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Sound enabled must be 'true' or 'false'",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::SoundVolume => {
            let vol_res = value.parse::<u8>();
            if vol_res.is_err() {
                log::warn!("Non-numeric sound volume setting value: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Sound volume must be an integer between 0 and 100",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
            let vol: u8 = vol_res.unwrap();
            if vol > 100 {
                log::warn!("Sound volume out of range: {}", vol);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Sound volume must not exceed 100",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::AnimationsEnabled => {
            if value != "true" && value != "false" {
                log::warn!("Invalid animations enabled value: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Animations enabled must be 'true' or 'false'",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::SoundProjector => {
            if value != "true" && value != "false" {
                log::warn!("Invalid sound projector value: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Sound projector must be 'true' or 'false'",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::PresentationShowAnswer => {
            if value != "true" && value != "false" {
                log::warn!("Invalid presentation show answer value: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Presentation show answer must be 'true' or 'false'",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::PresentationShowNumber => {
            if value != "true" && value != "false" {
                log::warn!("Invalid presentation show number value: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Presentation show number must be 'true' or 'false'",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        AppSettingKey::AutoOpenPresentation => {
            if value != "true" && value != "false" {
                log::warn!("Invalid auto open presentation value: '{}'", value);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Auto open presentation must be 'true' or 'false'",
                )
                .with_param("key", key_str)
                .with_param("value", value));
            }
        }
        _ => {
            if value.trim().is_empty() {
                log::warn!("Rejected empty setting value for key '{}'", key_str);
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    format!("Setting value for '{}' cannot be empty", key_str),
                )
                .with_param("key", key_str));
            }
            if value.len() > 256 {
                log::warn!(
                    "Rejected oversized setting value for key '{}' (length: {})",
                    key_str,
                    value.len()
                );
                return Err(AppError::new(
                    "SETTING_INVALID_VALUE",
                    "Setting value exceeds maximum length of 256 characters",
                )
                .with_param("key", key_str));
            }

            match key {
                AppSettingKey::Language => {
                    if value != "en" && value != "gu" {
                        log::warn!("Invalid language setting value: '{}'", value);
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            format!(
                                "Invalid value '{}' for 'language'. Allowed values are 'en' or 'gu'.",
                                value
                            ),
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                }
                AppSettingKey::Theme => {
                    if value != "dark" && value != "light" && value != "system" {
                        log::warn!("Invalid theme setting value: '{}'", value);
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            format!(
                                "Invalid value '{}' for 'theme'. Allowed values are 'dark', 'light', or 'system'.",
                                value
                            ),
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                }
                AppSettingKey::DefaultVideoVolume => {
                    let vol_res = value.parse::<u8>();
                    if vol_res.is_err() {
                        log::warn!("Non-numeric volume setting value: '{}'", value);
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            "Volume must be an integer between 0 and 100",
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                    let vol: u8 = vol_res.unwrap();
                    if vol > 100 {
                        log::warn!("Volume out of range: {}", vol);
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            "Volume must not exceed 100",
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                }
                AppSettingKey::VideoAutoplay => {
                    if value != "true" && value != "false" {
                        log::warn!("Invalid boolean setting value for autoplay: '{}'", value);
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            format!(
                                "Invalid boolean value '{}' for autoplay setting. Allowed values are 'true' or 'false'.",
                                value
                            ),
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                }
                AppSettingKey::ShowInternalNumbers => {
                    if value != "true" && value != "false" {
                        log::warn!(
                            "Invalid boolean setting value for show numbers: '{}'",
                            value
                        );
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            format!(
                                "Invalid boolean value '{}' for show numbers setting. Allowed values are 'true' or 'false'.",
                                value
                            ),
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                }
                AppSettingKey::UseGujaratiNumerals => {
                    if value != "true" && value != "false" {
                        log::warn!(
                            "Invalid boolean setting value for use gujarati numerals: '{}'",
                            value
                        );
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            format!(
                                "Invalid boolean value '{}' for use_gujarati_numerals setting. Allowed values are 'true' or 'false'.",
                                value
                            ),
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                }
                AppSettingKey::PresentationDisplay => {
                    let disp_res = value.parse::<usize>();
                    if disp_res.is_err() {
                        log::warn!("Invalid presentation display setting value: '{}'", value);
                        return Err(AppError::new(
                            "SETTING_INVALID_VALUE",
                            "Presentation display must be a non-negative integer",
                        )
                        .with_param("key", key_str)
                        .with_param("value", value));
                    }
                }
                _ => {}
            }
        }
    }

    Ok(key)
}

pub fn set_app_setting_internal(
    conn: &rusqlite::Connection,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    validate_app_setting(key, value)?;

    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
        params![key, value],
    )
    .map_err(|e: rusqlite::Error| {
        AppError::with_detail(
            "DATABASE_ERROR",
            "Failed to update app setting",
            e.to_string(),
        )
    })?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub db_path: String,
    pub schema_version: i32,
    pub language: String,
    pub theme: String,
    pub default_video_volume: String,
    pub video_autoplay: String,
    pub show_internal_numbers: String,
    pub use_gujarati_numerals: String,
    pub event_name: String,
    pub ticker_text: String,
    pub idle_poster: String,
    pub accent_color: String,
    pub sound_enabled: String,
    pub sound_volume: String,
    pub sound_projector: String,
    pub animations_enabled: String,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, DbState>) -> Result<AppInfo, AppError> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    let schema_version: i32 = get_schema_version(&conn).map_err(|e: String| {
        AppError::with_detail("DATABASE_ERROR", "Failed to query schema version", e)
    })?;

    let language: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'language';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "gu".to_string());

    let theme: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'theme';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "dark".to_string());

    let default_video_volume: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'default_video_volume';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "80".to_string());

    let video_autoplay: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'video_autoplay';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "false".to_string());

    let show_internal_numbers: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'show_internal_numbers';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "false".to_string());

    let use_gujarati_numerals: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'use_gujarati_numerals';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "false".to_string());

    let event_name: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'event_name';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "Musical Tambola".to_string());

    let ticker_text: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'ticker_text';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_default();

    let idle_poster: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'idle_poster';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_default();

    let accent_color: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'accent_color';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "#6366f1".to_string());

    let sound_enabled: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'sound_enabled';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "true".to_string());

    let sound_volume: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'sound_volume';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "80".to_string());

    let sound_projector: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'sound_projector';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "true".to_string());

    let animations_enabled: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'animations_enabled';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "true".to_string());

    let info: AppInfo = AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: state
            .db_path
            .parent()
            .map(|p: &std::path::Path| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        db_path: state.db_path.to_string_lossy().to_string(),
        schema_version,
        language,
        theme,
        default_video_volume,
        video_autoplay,
        show_internal_numbers,
        use_gujarati_numerals,
        event_name,
        ticker_text,
        idle_poster,
        accent_color,
        sound_enabled,
        sound_volume,
        sound_projector,
        animations_enabled,
    };

    Ok(info)
}

#[tauri::command]
pub fn set_app_setting(
    key: String,
    value: String,
    window: tauri::Window,
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<(), AppError> {
    use tauri::Emitter;
    crate::errors::require_main_window(&window)?;
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    set_app_setting_internal(&conn, &key, &value)?;

    let event_payload: serde_json::Value = serde_json::json!({
        "command": "SETTINGS_CHANGED",
        "payload": {
            "key": key,
            "value": value
        }
    });
    let _ = app.emit("presentation-command", &event_payload);

    Ok(())
}

#[tauri::command]
pub fn upload_idle_poster(
    bytes: Vec<u8>,
    window: tauri::Window,
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<String, AppError> {
    use tauri::Emitter;
    crate::errors::require_main_window(&window)?;

    const MAX_POSTER_SIZE: usize = 5 * 1024 * 1024;
    if bytes.len() > MAX_POSTER_SIZE {
        return Err(AppError::new(
            "IDLE_POSTER_TOO_LARGE",
            format!(
                "Idle poster size ({} bytes) exceeds maximum allowed limit of 5 MB",
                bytes.len()
            ),
        ));
    }

    let is_jpeg: bool =
        bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF;
    let is_png: bool =
        bytes.len() >= 8 && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    let ext: &'static str = if is_jpeg {
        "jpg"
    } else if is_png {
        "png"
    } else {
        return Err(AppError::new(
            "IDLE_POSTER_INVALID_FORMAT",
            "Idle poster must be a valid PNG or JPEG image",
        ));
    };

    use sha2::{Digest, Sha256};
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(&bytes);
    let hash_hex: String = format!("{:x}", hasher.finalize());
    let sha16: &str = &hash_hex[0..16];

    let app_data_dir: std::path::PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal_error(format!("Failed to get app data dir: {}", e)))?;
    let posters_dir: std::path::PathBuf = app_data_dir.join("posters");
    std::fs::create_dir_all(&posters_dir).map_err(|e: std::io::Error| {
        AppError::internal_error(format!("Failed to create posters directory: {}", e))
    })?;

    let poster_file: std::path::PathBuf =
        posters_dir.join(format!("idle_poster_{}.{}", sha16, ext));
    std::fs::write(&poster_file, &bytes).map_err(|e: std::io::Error| {
        AppError::internal_error(format!("Failed to write idle poster file: {}", e))
    })?;

    let poster_path_str: String = poster_file.to_string_lossy().to_string();

    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    set_app_setting_internal(&conn, "idle_poster", &poster_path_str)?;

    let event_payload: serde_json::Value = serde_json::json!({
        "command": "SETTINGS_CHANGED",
        "payload": {
            "key": "idle_poster",
            "value": &poster_path_str
        }
    });
    let _ = app.emit("presentation-command", &event_payload);

    Ok(poster_path_str)
}

#[tauri::command]
pub fn clear_idle_poster(
    window: tauri::Window,
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    pin_state: State<'_, crate::services::admin_pin::AdminPinState>,
) -> Result<(), AppError> {
    use tauri::Emitter;
    crate::errors::require_main_window(&window)?;
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    crate::services::admin_pin::require_admin_pin_if_game_in_progress(&conn, &pin_state)?;
    set_app_setting_internal(&conn, "idle_poster", "")?;

    let event_payload: serde_json::Value = serde_json::json!({
        "command": "SETTINGS_CHANGED",
        "payload": {
            "key": "idle_poster",
            "value": ""
        }
    });
    let _ = app.emit("presentation-command", &event_payload);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_app_setting_allowlist_and_values() {
        // Valid keys and values
        assert!(validate_app_setting("language", "en").is_ok());
        assert!(validate_app_setting("language", "gu").is_ok());
        assert!(validate_app_setting("theme", "dark").is_ok());
        assert!(validate_app_setting("theme", "light").is_ok());
        assert!(validate_app_setting("theme", "system").is_ok());
        assert!(validate_app_setting("default_video_volume", "80").is_ok());
        assert!(validate_app_setting("video_autoplay", "true").is_ok());
        assert!(validate_app_setting("show_internal_numbers", "false").is_ok());
        assert!(validate_app_setting("presentation_display", "0").is_ok());
        assert!(validate_app_setting("presentation_display", "1").is_ok());
        assert!(validate_app_setting("presentation_display", "-1").is_err());
        assert!(validate_app_setting("presentation_display", "invalid").is_err());

        // Branding & sounds valid keys and values
        assert!(validate_app_setting("event_name", "Grand Tambola").is_ok());
        assert!(validate_app_setting("event_name", "").is_ok());
        assert!(validate_app_setting("event_name", &"a".repeat(201)).is_err());
        assert!(validate_app_setting("ticker_text", "Welcome to the game!").is_ok());
        assert!(validate_app_setting("ticker_text", "").is_ok());
        assert!(validate_app_setting("ticker_text", &"a".repeat(201)).is_err());
        assert!(validate_app_setting("idle_poster", "C:/posters/idle.png").is_ok());
        assert!(validate_app_setting("idle_poster", "").is_ok());
        assert!(validate_app_setting("idle_poster", &"a".repeat(257)).is_err());
        assert!(validate_app_setting("accent_color", "#ff5500").is_ok());
        assert!(validate_app_setting("accent_color", "#AABBCC").is_ok());
        assert!(validate_app_setting("accent_color", "red").is_err());
        assert!(validate_app_setting("accent_color", "#12345").is_err());
        assert!(validate_app_setting("accent_color", "#1234567").is_err());
        assert!(validate_app_setting("sound_enabled", "true").is_ok());
        assert!(validate_app_setting("sound_enabled", "false").is_ok());
        assert!(validate_app_setting("sound_enabled", "yes").is_err());
        assert!(validate_app_setting("sound_volume", "50").is_ok());
        assert!(validate_app_setting("sound_volume", "101").is_err());
        assert!(validate_app_setting("sound_volume", "invalid").is_err());
        assert!(validate_app_setting("sound_projector", "true").is_ok());
        assert!(validate_app_setting("sound_projector", "false").is_ok());
        assert!(validate_app_setting("sound_projector", "no").is_err());
        assert!(validate_app_setting("animations_enabled", "true").is_ok());
        assert!(validate_app_setting("animations_enabled", "false").is_ok());
        assert!(validate_app_setting("animations_enabled", "off").is_err());

        // Removed / forbidden settings
        assert!(validate_app_setting("sound_pitch", "50").is_err());
        assert!(validate_app_setting("autoplay_enabled", "false").is_err());
        assert!(validate_app_setting("autoplay_interval_seconds", "5").is_err());
        assert!(validate_app_setting("show_numbers_admin", "false").is_err());

        // Empty / oversized checks
        assert!(validate_app_setting("", "en").is_err());
        assert!(validate_app_setting("language", "").is_err());
        assert!(validate_app_setting(&"a".repeat(300), "en").is_err());
        assert!(validate_app_setting("language", &"a".repeat(300)).is_err());

        // Invalid values
        assert!(validate_app_setting("language", "fr").is_err());
        assert!(validate_app_setting("theme", "neon").is_err());
        assert!(validate_app_setting("default_video_volume", "150").is_err());
        assert!(validate_app_setting("video_autoplay", "yes").is_err());
    }

    #[test]
    fn test_app_setting_persistence_across_reopen() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();

        // Initial setup and migration
        {
            let conn = Connection::open(&path).expect("Failed to open DB");
            run_migrations(&conn).expect("Failed migrations");

            // Initial language should be default seeded 'gu'
            let initial_lang: String = conn
                .query_row(
                    "SELECT value FROM app_settings WHERE key = 'language';",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(initial_lang, "gu");

            // Attempting to set an invalid setting must fail
            assert!(
                set_app_setting_internal(&conn, "unauthorized_key", "malicious_val").is_err(),
                "Invalid setting key must be rejected"
            );
            assert!(
                set_app_setting_internal(&conn, "language", "xx").is_err(),
                "Invalid setting value must be rejected"
            );

            // Update language to 'en'
            set_app_setting_internal(&conn, "language", "en").expect("Valid update must succeed");

            // Update theme to 'system'
            set_app_setting_internal(&conn, "theme", "system").expect("Valid update must succeed");
        }

        // Re-open DB from disk and verify persistence
        {
            let conn = Connection::open(&path).expect("Failed to reopen DB");
            let reloaded_lang: String = conn
                .query_row(
                    "SELECT value FROM app_settings WHERE key = 'language';",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(
                reloaded_lang, "en",
                "Language setting must persist across reopen"
            );

            let reloaded_theme: String = conn
                .query_row(
                    "SELECT value FROM app_settings WHERE key = 'theme';",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(
                reloaded_theme, "system",
                "Theme setting must persist across reopen"
            );
        }
    }
}
