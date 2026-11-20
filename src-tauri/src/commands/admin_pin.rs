use crate::db::DbState;
use crate::errors::AppError;
use crate::services::admin_pin::AdminPinState;
use serde::{Deserialize, Serialize};
use tauri::{State, Window};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPinStatusDto {
    pub pin_set: bool,
    pub is_unlocked: bool,
}

#[tauri::command]
pub fn set_admin_pin(
    pin: String,
    current_pin: Option<String>,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<(), AppError> {
    crate::errors::require_main_window(&window)?;
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    let existing_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'admin_pin_hash';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .ok()
        .filter(|s: &String| !s.trim().is_empty());

    if let Some(ref old_hash) = existing_hash {
        let cur: &str = current_pin.as_deref().unwrap_or("");
        if !crate::services::admin_pin::verify_pin_hash(cur, old_hash)? {
            return Err(AppError::new(
                "ADMIN_PIN_INVALID",
                "Current admin PIN is incorrect",
            ));
        }
    }

    let hashed: String = crate::services::admin_pin::hash_pin(&pin)?;
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES ('admin_pin_hash', ?1, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
        rusqlite::params![hashed],
    )
    .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

    pin_state.grant_session();
    Ok(())
}

#[tauri::command]
pub fn verify_admin_pin(
    pin: String,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<bool, AppError> {
    crate::errors::require_main_window(&window)?;
    pin_state.check_rate_limit()?;

    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    let stored_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'admin_pin_hash';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .ok()
        .filter(|s: &String| !s.trim().is_empty());

    match stored_hash {
        None => {
            pin_state.grant_session();
            Ok(true)
        }
        Some(hash) => {
            let matches: bool = crate::services::admin_pin::verify_pin_hash(&pin, &hash)?;
            if matches {
                pin_state.grant_session();
                Ok(true)
            } else {
                pin_state.record_failure();
                Err(AppError::new("ADMIN_PIN_INVALID", "Invalid admin PIN"))
            }
        }
    }
}

#[tauri::command]
pub fn clear_admin_pin(
    current_pin: String,
    window: Window,
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<(), AppError> {
    crate::errors::require_main_window(&window)?;
    pin_state.check_rate_limit()?;

    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    let stored_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'admin_pin_hash';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .ok()
        .filter(|s: &String| !s.trim().is_empty());

    if let Some(hash) = stored_hash {
        let matches: bool = crate::services::admin_pin::verify_pin_hash(&current_pin, &hash)?;
        if !matches {
            pin_state.record_failure();
            return Err(AppError::new("ADMIN_PIN_INVALID", "Invalid admin PIN"));
        }
    }

    conn.execute(
        "UPDATE app_settings SET value = '', updated_at = CURRENT_TIMESTAMP WHERE key = 'admin_pin_hash';",
        [],
    )
    .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

    pin_state.clear_session();
    Ok(())
}

#[tauri::command]
pub fn get_admin_pin_status(
    state: State<'_, DbState>,
    pin_state: State<'_, AdminPinState>,
) -> Result<AdminPinStatusDto, AppError> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = state
        .conn
        .lock()
        .map_err(|_| AppError::new("LOCK_ERROR", "Failed to acquire database lock"))?;

    let stored_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'admin_pin_hash';",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .ok()
        .filter(|s: &String| !s.trim().is_empty());

    let pin_set: bool = stored_hash.is_some();
    let is_unlocked: bool = if pin_set {
        pin_state.is_session_valid()
    } else {
        true
    };

    Ok(AdminPinStatusDto {
        pin_set,
        is_unlocked,
    })
}
