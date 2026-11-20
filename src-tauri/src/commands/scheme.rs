use crate::db::repositories::scheme::SchemeRepository;
use crate::db::DbState;
use crate::domain::schemes::{PrizeScheme, SaveSchemeInput, SchemeType, SchemeTypeInfo};
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

#[tauri::command]
pub fn list_schemes(state: State<'_, DbState>) -> Result<Vec<PrizeScheme>, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    SchemeRepository::list(&conn)
}

#[tauri::command]
pub fn save_scheme(
    input: SaveSchemeInput,
    state: State<'_, DbState>,
) -> Result<PrizeScheme, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    SchemeRepository::save(&conn, &input)
}

#[tauri::command]
pub fn set_scheme_active(
    id: i64,
    is_active: bool,
    state: State<'_, DbState>,
) -> Result<PrizeScheme, String> {
    let conn: MutexGuard<'_, Connection> = state
        .conn
        .lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;

    SchemeRepository::set_active(&conn, id, is_active)
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
