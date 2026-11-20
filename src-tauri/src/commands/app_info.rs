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
