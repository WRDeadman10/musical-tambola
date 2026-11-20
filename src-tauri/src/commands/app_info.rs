use crate::db::migrations::get_schema_version;
use crate::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub db_path: String,
    pub schema_version: i32,
    pub language: String,
    pub theme: String,
}

pub fn validate_app_setting(key: &str, value: &str) -> Result<(), String> {
    match key {
        "language" => {
            if value != "en" && value != "gu" {
                return Err(format!(
                    "Invalid value '{}' for 'language'. Allowed values are 'en' or 'gu'.",
                    value
                ));
            }
        }
        "theme" => {
            if value != "dark" && value != "light" {
                return Err(format!(
                    "Invalid value '{}' for 'theme'. Allowed values are 'dark' or 'light'.",
                    value
                ));
            }
        }
        "sound_volume" => {
            let vol: u8 = value
                .parse::<u8>()
                .map_err(|_| "Sound volume must be an integer between 0 and 100".to_string())?;
            if vol > 100 {
                return Err("Sound volume must not exceed 100".to_string());
            }
        }
        "autoplay_enabled" => {
            if value != "true" && value != "false" {
                return Err(format!(
                    "Invalid boolean value '{}' for 'autoplay_enabled'",
                    value
                ));
            }
        }
        "autoplay_interval_seconds" => {
            let secs: u32 = value
                .parse::<u32>()
                .map_err(|_| "Autoplay interval must be a positive integer".to_string())?;
            if secs == 0 || secs > 60 {
                return Err("Autoplay interval must be between 1 and 60 seconds".to_string());
            }
        }
        "show_numbers_admin" => {
            if value != "true" && value != "false" {
                return Err(format!(
                    "Invalid boolean value '{}' for 'show_numbers_admin'",
                    value
                ));
            }
        }
        _ => {
            return Err(format!(
                "Unsupported or unrecognized app setting key: '{}'",
                key
            ));
        }
    }

    Ok(())
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
        assert!(validate_app_setting("sound_volume", "50").is_ok());
        assert!(validate_app_setting("autoplay_enabled", "true").is_ok());
        assert!(validate_app_setting("autoplay_interval_seconds", "5").is_ok());
        assert!(validate_app_setting("show_numbers_admin", "false").is_ok());

        // Invalid keys
        assert!(validate_app_setting("unknown_key", "value").is_err());
        assert!(validate_app_setting("hacked_setting", "123").is_err());

        // Invalid values
        assert!(validate_app_setting("language", "fr").is_err());
        assert!(validate_app_setting("theme", "neon").is_err());
        assert!(validate_app_setting("sound_volume", "150").is_err());
        assert!(validate_app_setting("sound_volume", "-5").is_err());
        assert!(validate_app_setting("autoplay_enabled", "yes").is_err());
        assert!(validate_app_setting("autoplay_interval_seconds", "0").is_err());
        assert!(validate_app_setting("autoplay_interval_seconds", "100").is_err());
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
        }
    }
}
