use crate::db::migrations::get_schema_version;
use crate::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tauri::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSettingKey {
    Language,
    Theme,
    DefaultVideoVolume,
    VideoAutoplay,
    ShowInternalNumbers,
}

impl FromStr for AppSettingKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "language" => Ok(Self::Language),
            "theme" => Ok(Self::Theme),
            "default_video_volume" => Ok(Self::DefaultVideoVolume),
            "video_autoplay" => Ok(Self::VideoAutoplay),
            "show_internal_numbers" => Ok(Self::ShowInternalNumbers),
            _ => {
                log::warn!("Rejected invalid app setting key: '{}'", s);
                Err(format!("Unsupported or unrecognized setting key: '{}'", s))
            }
        }
    }
}

pub fn validate_app_setting(key_str: &str, value: &str) -> Result<AppSettingKey, String> {
    if key_str.trim().is_empty() {
        log::warn!("Rejected empty setting key");
        return Err("Setting key cannot be empty".to_string());
    }
    if key_str.len() > 256 {
        log::warn!("Rejected oversized setting key (length: {})", key_str.len());
        return Err("Setting key exceeds maximum length of 256 characters".to_string());
    }
    if value.trim().is_empty() {
        log::warn!("Rejected empty setting value for key '{}'", key_str);
        return Err(format!("Setting value for '{}' cannot be empty", key_str));
    }
    if value.len() > 256 {
        log::warn!(
            "Rejected oversized setting value for key '{}' (length: {})",
            key_str,
            value.len()
        );
        return Err("Setting value exceeds maximum length of 256 characters".to_string());
    }

    let key: AppSettingKey = AppSettingKey::from_str(key_str)?;

    match key {
        AppSettingKey::Language => {
            if value != "en" && value != "gu" {
                log::warn!("Invalid language setting value: '{}'", value);
                return Err(format!(
                    "Invalid value '{}' for 'language'. Allowed values are 'en' or 'gu'.",
                    value
                ));
            }
        }
        AppSettingKey::Theme => {
            if value != "dark" && value != "light" && value != "system" {
                log::warn!("Invalid theme setting value: '{}'", value);
                return Err(format!(
                    "Invalid value '{}' for 'theme'. Allowed values are 'dark', 'light', or 'system'.",
                    value
                ));
            }
        }
        AppSettingKey::DefaultVideoVolume => {
            let vol: u8 = value.parse::<u8>().map_err(|_| {
                log::warn!("Non-numeric volume setting value: '{}'", value);
                "Volume must be an integer between 0 and 100".to_string()
            })?;
            if vol > 100 {
                log::warn!("Volume out of range: {}", vol);
                return Err("Volume must not exceed 100".to_string());
            }
        }
        AppSettingKey::VideoAutoplay => {
            if value != "true" && value != "false" {
                log::warn!("Invalid boolean setting value for autoplay: '{}'", value);
                return Err(format!(
                    "Invalid boolean value '{}' for autoplay setting. Allowed values are 'true' or 'false'.",
                    value
                ));
            }
        }
        AppSettingKey::ShowInternalNumbers => {
            if value != "true" && value != "false" {
                log::warn!(
                    "Invalid boolean setting value for show numbers: '{}'",
                    value
                );
                return Err(format!(
                    "Invalid boolean value '{}' for show numbers setting. Allowed values are 'true' or 'false'.",
                    value
                ));
            }
        }
    }

    Ok(key)
}

pub fn set_app_setting_internal(
    conn: &rusqlite::Connection,
    key: &str,
    value: &str,
) -> Result<(), String> {
    validate_app_setting(key, value)?;

    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
        params![key, value],
    )
    .map_err(|e: rusqlite::Error| format!("Failed to update app setting: {}", e))?;

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
}

#[tauri::command]
pub fn get_app_info(state: State<'_, DbState>) -> Result<AppInfo, String> {
    let conn = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    let schema_version: i32 = get_schema_version(&conn)?;

    let language: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'language';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "gu".to_string());

    let theme: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'theme';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "dark".to_string());

    let default_video_volume: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'default_video_volume';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "80".to_string());

    let video_autoplay: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'video_autoplay';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "false".to_string());

    let show_internal_numbers: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'show_internal_numbers';",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "false".to_string());

    let info: AppInfo = AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: state
            .db_path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        db_path: state.db_path.to_string_lossy().to_string(),
        schema_version,
        language,
        theme,
        default_video_volume,
        video_autoplay,
        show_internal_numbers,
    };

    Ok(info)
}

#[tauri::command]
pub fn set_app_setting(
    key: String,
    value: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    set_app_setting_internal(&conn, &key, &value)
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

        // Removed / forbidden settings
        assert!(validate_app_setting("sound_volume", "50").is_err());
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
