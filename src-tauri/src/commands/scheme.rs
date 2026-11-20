use crate::db::repositories::scheme::SchemeRepository;
use crate::db::DbState;
use crate::domain::schemes::{PrizeScheme, SaveSchemeInput, SchemeType, SchemeTypeInfo};
use crate::errors::AppError;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

#[tauri::command]
pub fn list_schemes(state: State<'_, DbState>) -> Result<Vec<PrizeScheme>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    SchemeRepository::list(&conn).map_err(AppError::database_error)
}

#[tauri::command]
pub fn save_scheme(
    input: SaveSchemeInput,
    state: State<'_, DbState>,
) -> Result<PrizeScheme, AppError> {
    if !(1..=100).contains(&input.max_winners) {
        return Err(AppError::new(
            "SCHEME_INVALID_CAPACITY",
            format!(
                "Max winners capacity must be between 1 and 100, got {}",
                input.max_winners
            ),
        )
        .with_param("maxWinners", input.max_winners.to_string()));
    }

    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    SchemeRepository::save(&conn, &input).map_err(|e| {
        if e.contains("already exists") || e.contains("uniqueness") {
            AppError::new(
                "SCHEME_NAME_DUPLICATE",
                format!(
                    "A prize scheme with this name already exists: {}",
                    input.name
                ),
            )
            .with_param("name", input.name)
        } else if e.contains("not found") {
            let sid = input.id.unwrap_or(0);
            AppError::new(
                "SCHEME_NOT_FOUND",
                format!("Prize scheme was not found: {}", sid),
            )
            .with_param("schemeId", sid.to_string())
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn set_scheme_active(
    id: i64,
    is_active: bool,
    state: State<'_, DbState>,
) -> Result<PrizeScheme, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    SchemeRepository::set_active(&conn, id, is_active).map_err(|e| {
        if e.contains("not found") {
            AppError::new(
                "SCHEME_NOT_FOUND",
                format!("Prize scheme was not found: {}", id),
            )
            .with_param("schemeId", id.to_string())
        } else {
            AppError::database_error(e)
        }
    })
}

#[tauri::command]
pub fn list_scheme_types() -> Vec<SchemeTypeInfo> {
    vec![
        SchemeTypeInfo {
            scheme_type: SchemeType::EarlyFive,
            is_full_house: SchemeType::EarlyFive.is_full_house(),
            rule_description: SchemeType::EarlyFive.rule_description().to_string(),
        },
        SchemeTypeInfo {
            scheme_type: SchemeType::FourCorners,
            is_full_house: SchemeType::FourCorners.is_full_house(),
            rule_description: SchemeType::FourCorners.rule_description().to_string(),
        },
        SchemeTypeInfo {
            scheme_type: SchemeType::TopLine,
            is_full_house: SchemeType::TopLine.is_full_house(),
            rule_description: SchemeType::TopLine.rule_description().to_string(),
        },
        SchemeTypeInfo {
            scheme_type: SchemeType::MiddleLine,
            is_full_house: SchemeType::MiddleLine.is_full_house(),
            rule_description: SchemeType::MiddleLine.rule_description().to_string(),
        },
        SchemeTypeInfo {
            scheme_type: SchemeType::BottomLine,
            is_full_house: SchemeType::BottomLine.is_full_house(),
            rule_description: SchemeType::BottomLine.rule_description().to_string(),
        },
        SchemeTypeInfo {
            scheme_type: SchemeType::FullHouse,
            is_full_house: SchemeType::FullHouse.is_full_house(),
            rule_description: SchemeType::FullHouse.rule_description().to_string(),
        },
    ]
}
