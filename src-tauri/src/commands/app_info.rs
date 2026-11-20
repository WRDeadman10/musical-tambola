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

    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
        params![key, value],
    )
    .map_err(|e: rusqlite::Error| format!("Failed to update app setting: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    #[test]
    fn test_app_setting_persistence_across_reopen() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();

        // Initial setup and migration
        {
            let conn = Connection::open(&path).expect("Failed to open DB");
            run_migrations(&conn).expect("Failed migrations");

            let db_state = DbState {
                conn: Mutex::new(conn),
                db_path: path.clone(),
            };

            // Initial language should be default seeded 'gu'
            let stmt = db_state.conn.lock().unwrap();
            let initial_lang: String = stmt
                .query_row(
                    "SELECT value FROM app_settings WHERE key = 'language';",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(initial_lang, "gu");

            // Update language to 'en'
            stmt.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES ('language', 'en', CURRENT_TIMESTAMP)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
                [],
            )
            .unwrap();
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
