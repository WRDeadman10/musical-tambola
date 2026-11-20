use crate::db::repositories::lucky_draw::{LuckyDrawDto, LuckyDrawRepository, LuckyWinnerDto};
use crate::db::DbState;
use crate::domain::random::OsRngSource;
use crate::errors::AppError;
use rusqlite::{params, Connection};
use std::sync::MutexGuard;
use tauri::{AppHandle, Emitter, State, Window};

#[tauri::command]
pub fn create_lucky_draw(
    name: Option<String>,
    source: String,
    names: Option<Vec<String>>,
    batch_id: Option<i64>,
    exclude_previous_winners: Option<bool>,
    window: Window,
    state: State<'_, DbState>,
) -> Result<LuckyDrawDto, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;
    let items: Vec<String> = names.unwrap_or_default();
    let exclude: bool = exclude_previous_winners.unwrap_or(false);

    LuckyDrawRepository::create_draw(
        &mut conn,
        name.as_deref(),
        &source,
        &items,
        batch_id,
        exclude,
    )
}

#[tauri::command]
pub fn draw_lucky_winner(
    draw_id: i64,
    window: Window,
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<LuckyWinnerDto, AppError> {
    crate::errors::require_main_window(&window)?;
    let mut conn: MutexGuard<'_, Connection> =
        state.conn.lock().map_err(|_| AppError::lock_error())?;
    let mut rng: OsRngSource = OsRngSource;

    let result: LuckyWinnerDto = LuckyDrawRepository::draw_winner(&mut conn, draw_id, &mut rng)?;

    let draw_name: String = conn
        .query_row(
            "SELECT name FROM lucky_draws WHERE id = ?1;",
            params![result.draw_id],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap_or_else(|_| "Lucky Draw".to_string());

    let mut stmt = conn
        .prepare("SELECT entry_value FROM lucky_draw_entries WHERE draw_id = ?1 LIMIT 20;")
        .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
    let candidates_sample: Vec<String> = stmt
        .query_map(params![result.draw_id], |r: &rusqlite::Row<'_>| r.get(0))
        .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?
        .filter_map(|r: Result<String, rusqlite::Error>| r.ok())
        .collect();

    let event_payload: serde_json::Value = serde_json::json!({
        "command": "LUCKY_DRAW",
        "payload": {
            "drawName": draw_name,
            "candidatesSample": candidates_sample,
            "winner": result.value.clone()
        }
    });
    let _ = app.emit("presentation-command", &event_payload);

    Ok(result)
}

#[tauri::command]
pub fn list_lucky_draws(state: State<'_, DbState>) -> Result<Vec<LuckyDrawDto>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;
    LuckyDrawRepository::list_all(&conn)
}

#[cfg(test)]
mod tests {
    use crate::db::repositories::lucky_draw::LuckyDrawInputDto;

    #[test]
    fn test_lucky_draw_input_dto_deserialization() {
        let json_str =
            r#"{"source":"NAMES","items":["Anand","Bhavik"],"excludePreviousWinners":true}"#;
        let dto: LuckyDrawInputDto = serde_json::from_str(json_str).unwrap();
        assert_eq!(dto.source, "NAMES");
        assert_eq!(dto.items.len(), 2);
        assert!(dto.exclude_previous_winners);
    }
}
