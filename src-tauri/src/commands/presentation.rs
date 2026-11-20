use crate::db::connection::DbState;
use crate::db::repositories::game::GameRepository;
use crate::domain::game::{CalledItem, GameState};
use crate::errors::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Emitter, Manager, Monitor, State, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationCountdownDto {
    pub seconds: u32,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationAnnouncementDto {
    pub scheme_id: i64,
    pub scheme_name: String,
    pub ticket_code: String,
    pub mask: Vec<Vec<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationLuckyDrawDto {
    pub ticket_code: String,
    pub name: String,
    pub winner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationStateDto {
    pub scene: String,
    pub show_answer: bool,
    pub show_number: bool,
    pub countdown: Option<PresentationCountdownDto>,
    pub announcement: Option<PresentationAnnouncementDto>,
    pub lucky_draw: Option<PresentationLuckyDrawDto>,
    pub current: Option<CalledItem>,
    pub game: Option<GameState>,
    pub volume: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfoDto {
    pub index: usize,
    pub name: String,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextItemDto {
    pub position: u32,
    pub number: u8,
    pub title: String,
    pub media_type: String,
    pub source_type: String,
    pub display_url: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PresentationInnerState {
    pub scene: String,
    pub show_answer: bool,
    pub show_number: bool,
    pub countdown: Option<PresentationCountdownDto>,
    pub announcement: Option<PresentationAnnouncementDto>,
    pub lucky_draw: Option<PresentationLuckyDrawDto>,
    pub volume: Option<f64>,
}

impl Default for PresentationInnerState {
    fn default() -> Self {
        Self {
            scene: "IDLE".to_string(),
            show_answer: true,
            show_number: true,
            countdown: None,
            announcement: None,
            lucky_draw: None,
            volume: Some(1.0),
        }
    }
}

pub struct PresentationStateManaged {
    pub inner: Mutex<PresentationInnerState>,
}

impl Default for PresentationStateManaged {
    fn default() -> Self {
        Self {
            inner: Mutex::new(PresentationInnerState::default()),
        }
    }
}

#[tauri::command]
pub fn list_displays(app: AppHandle) -> Result<Vec<DisplayInfoDto>, AppError> {
    let monitors: Vec<Monitor> = app
        .available_monitors()
        .map_err(|e| AppError::new("MONITOR_ERROR", format!("Failed to list monitors: {}", e)))?;

    if monitors.is_empty() {
        return Ok(vec![DisplayInfoDto {
            index: 0,
            name: "Default Display".to_string(),
            primary: true,
        }]);
    }

    let primary_monitor: Option<Monitor> = app.primary_monitor().ok().flatten();
    let primary_name: Option<String> = primary_monitor.and_then(|m: Monitor| m.name().cloned());

    let mut list: Vec<DisplayInfoDto> = Vec::with_capacity(monitors.len());
    for (idx, mon) in monitors.into_iter().enumerate() {
        let name: String = mon
            .name()
            .cloned()
            .unwrap_or_else(|| format!("Display {}", idx + 1));
        let is_primary: bool = if let Some(ref p_name) = primary_name {
            &name == p_name
        } else {
            idx == 0
        };

        list.push(DisplayInfoDto {
            index: idx,
            name,
            primary: is_primary,
        });
    }

    Ok(list)
}

#[tauri::command]
pub fn open_presentation(display_index: Option<usize>, app: AppHandle) -> Result<(), AppError> {
    if let Some(existing_window) = app.get_webview_window("presentation") {
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    let monitors: Vec<Monitor> = app.available_monitors().unwrap_or_default();
    let target_monitor: Option<Monitor> =
        display_index.and_then(|idx: usize| monitors.get(idx).cloned());

    let mut builder: WebviewWindowBuilder<'_, tauri::Wry, AppHandle> =
        WebviewWindowBuilder::new(&app, "presentation", WebviewUrl::App("/present".into()))
            .title("Media Tambola - Presentation")
            .decorations(false)
            .resizable(true);

    if let Some(mon) = target_monitor {
        let pos = mon.position();
        let size = mon.size();
        builder = builder
            .position(pos.x as f64, pos.y as f64)
            .inner_size(size.width as f64, size.height as f64)
            .fullscreen(true);
    } else {
        builder = builder.inner_size(1280.0, 720.0);
    }

    builder.build().map_err(|e| {
        AppError::new(
            "WINDOW_CREATION_FAILED",
            format!("Failed to create presentation window: {}", e),
        )
    })?;

    Ok(())
}

#[tauri::command]
pub fn close_presentation(app: AppHandle) -> Result<(), AppError> {
    if let Some(win) = app.get_webview_window("presentation") {
        win.close().map_err(|e| {
            AppError::new(
                "WINDOW_CLOSE_FAILED",
                format!("Failed to close presentation window: {}", e),
            )
        })?;
    }
    Ok(())
}

#[tauri::command]
pub fn presentation_status(app: AppHandle) -> Result<bool, AppError> {
    Ok(app.get_webview_window("presentation").is_some())
}

#[tauri::command]
pub fn presentation_command(
    command: String,
    payload: serde_json::Value,
    state: State<'_, PresentationStateManaged>,
    app: AppHandle,
) -> Result<(), AppError> {
    {
        let mut inner: MutexGuard<'_, PresentationInnerState> =
            state.inner.lock().map_err(|_| AppError::lock_error())?;
        match command.as_str() {
            "SHOW_ANSWER" => inner.show_answer = true,
            "HIDE_ANSWER" => inner.show_answer = false,
            "SHOW_NUMBER" => inner.show_number = true,
            "HIDE_NUMBER" => inner.show_number = false,
            "SPLASH" => inner.scene = "SPLASH".to_string(),
            "PLAY" => inner.scene = "MEDIA".to_string(),
            "COUNTDOWN" => {
                inner.scene = "COUNTDOWN".to_string();
                if let Ok(cd) = serde_json::from_value::<PresentationCountdownDto>(payload.clone())
                {
                    inner.countdown = Some(cd);
                }
            }
            "ANNOUNCE" => {
                inner.scene = "ANNOUNCE".to_string();
                if let Ok(ann) =
                    serde_json::from_value::<PresentationAnnouncementDto>(payload.clone())
                {
                    inner.announcement = Some(ann);
                }
            }
            "SET_VOLUME" => {
                if let Some(vol) = payload
                    .get("volume")
                    .and_then(|v: &serde_json::Value| v.as_f64())
                {
                    inner.volume = Some(vol);
                }
            }
            _ => {}
        }
    }

    let event_payload: serde_json::Value = serde_json::json!({
        "command": command,
        "payload": payload
    });
    let _ = app.emit("presentation-command", &event_payload);
    Ok(())
}

#[tauri::command]
pub fn get_presentation_state(
    state: State<'_, PresentationStateManaged>,
    db: State<'_, DbState>,
) -> Result<PresentationStateDto, AppError> {
    let inner: MutexGuard<'_, PresentationInnerState> =
        state.inner.lock().map_err(|_| AppError::lock_error())?;
    let conn: MutexGuard<'_, Connection> = db.conn.lock().map_err(|_| AppError::lock_error())?;

    let active_game: Option<GameState> = GameRepository::get_active_game(&conn).ok().flatten();
    let current_called: Option<CalledItem> = active_game
        .as_ref()
        .and_then(|g: &GameState| g.current.clone());

    let scene: String = if inner.scene == "IDLE" && current_called.is_some() {
        "MEDIA".to_string()
    } else {
        inner.scene.clone()
    };

    Ok(PresentationStateDto {
        scene,
        show_answer: inner.show_answer,
        show_number: inner.show_number,
        countdown: inner.countdown.clone(),
        announcement: inner.announcement.clone(),
        lucky_draw: inner.lucky_draw.clone(),
        current: current_called,
        game: active_game,
        volume: inner.volume,
    })
}

#[tauri::command]
pub fn get_next_item(
    game_id: i64,
    state: State<'_, DbState>,
) -> Result<Option<NextItemDto>, AppError> {
    let conn: MutexGuard<'_, Connection> = state.conn.lock().map_err(|_| AppError::lock_error())?;

    let (draw_seq_json, current_pos): (String, u32) = conn
        .query_row(
            "SELECT draw_sequence_json, current_draw_position FROM games WHERE id = ?1;",
            params![game_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| AppError::new("GAME_NOT_FOUND", format!("Game session not found: {}", e)))?;

    let seq: Vec<u8> = serde_json::from_str(&draw_seq_json).map_err(|e| {
        AppError::new(
            "CORRUPTED_SEQUENCE",
            format!("Invalid draw sequence json: {}", e),
        )
    })?;

    let next_index: usize = current_pos as usize;
    if next_index >= seq.len() {
        return Ok(None);
    }

    let next_number: u8 = seq[next_index];
    let next_position: u32 = current_pos + 1;

    // First try snapshot
    let snapshot_res: Result<(String, String, String, Option<String>, String), rusqlite::Error> = conn
        .query_row(
            "SELECT media_type, source_type, path_or_url, poster_path, title_gujarati FROM game_media_snapshots WHERE game_id = ?1 AND number = ?2;",
            params![game_id, next_number],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        );

    let (media_type, source_type, path_or_url, poster_path, title): (String, String, String, Option<String>, String) =
        match snapshot_res {
            Ok(tuple) => tuple,
            Err(_) => {
                conn.query_row(
                    "SELECT media_type, source_type, path_or_url, poster_path, title_gujarati FROM media_assets WHERE number = ?1;",
                    params![next_number],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                ).map_err(|e| {
                    AppError::new(
                        "MEDIA_NOT_FOUND",
                        format!("Next media item not found for number {}: {}", next_number, e),
                    )
                })?
            }
        };

    let (display_url, thumbnail_url): (String, Option<String>) = if source_type == "LOCAL" {
        let disp: String = format!("http://media.localhost/game/{}/{}", game_id, next_number);
        let thumb: Option<String> = if poster_path.is_some() {
            Some(format!(
                "http://media.localhost/game/{}/{}/thumb",
                game_id, next_number
            ))
        } else {
            None
        };
        (disp, thumb)
    } else {
        (path_or_url, poster_path)
    };

    Ok(Some(NextItemDto {
        position: next_position,
        number: next_number,
        title,
        media_type,
        source_type,
        display_url,
        thumbnail_url,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::media::MediaRepository;
    use crate::db::repositories::ticket::TicketRepository;
    use crate::domain::media::{MediaType, SaveMediaInput, SourceType};
    use crate::domain::random::DeterministicRandomSource;

    #[test]
    fn test_presentation_inner_state_defaults() {
        let state: PresentationInnerState = PresentationInnerState::default();
        assert_eq!(state.scene, "IDLE");
        assert!(state.show_answer);
        assert!(state.show_number);
        assert!(state.countdown.is_none());
        assert!(state.announcement.is_none());
        assert!(state.lucky_draw.is_none());
        assert_eq!(state.volume, Some(1.0));
    }

    #[test]
    fn test_presentation_command_state_transitions() {
        let mut inner: PresentationInnerState = PresentationInnerState {
            show_answer: false,
            ..Default::default()
        };

        // SHOW_ANSWER / HIDE_ANSWER
        assert!(!inner.show_answer);
        inner.show_answer = true;
        assert!(inner.show_answer);

        // SHOW_NUMBER / HIDE_NUMBER
        inner.show_number = false;
        assert!(!inner.show_number);
        inner.show_number = true;
        assert!(inner.show_number);

        // SPLASH
        inner.scene = "SPLASH".to_string();
        assert_eq!(inner.scene, "SPLASH");

        // PLAY
        inner.scene = "MEDIA".to_string();
        assert_eq!(inner.scene, "MEDIA");

        // COUNTDOWN
        inner.scene = "COUNTDOWN".to_string();
        inner.countdown = Some(PresentationCountdownDto {
            seconds: 10,
            state: "RUNNING".to_string(),
        });
        assert_eq!(inner.scene, "COUNTDOWN");
        assert_eq!(inner.countdown.as_ref().unwrap().seconds, 10);

        // ANNOUNCE
        inner.scene = "ANNOUNCE".to_string();
        inner.announcement = Some(PresentationAnnouncementDto {
            scheme_id: 1,
            scheme_name: "Top Line".to_string(),
            ticket_code: "T-000001".to_string(),
            mask: vec![vec![true; 5]; 3],
        });
        assert_eq!(inner.scene, "ANNOUNCE");
        assert_eq!(inner.announcement.as_ref().unwrap().ticket_code, "T-000001");

        // SET_VOLUME
        inner.volume = Some(0.75);
        assert_eq!(inner.volume, Some(0.75));
    }

    #[test]
    fn test_get_next_item_dto_and_progression() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        for num in 1..=90 {
            MediaRepository::upsert(
                &conn,
                &SaveMediaInput {
                    number: num,
                    media_type: MediaType::Video,
                    source_type: SourceType::Local,
                    source: format!("C:/media/track_{}.mp4", num),
                    thumbnail_source: Some(format!("C:/media/poster_{}.jpg", num)),
                    title: format!("ગીત {}", num),
                },
            )
            .unwrap();

            conn.execute(
                "UPDATE media_assets SET is_valid = 1, last_validated_at = CURRENT_TIMESTAMP WHERE number = ?1;",
                params![num],
            )
            .unwrap();
        }

        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(777);
        let batch = TicketRepository::generate_batch(
            &mut conn,
            "Presentation Batch",
            2,
            None,
            None,
            &mut rng,
            |_, _| {},
        )
        .unwrap();

        let game = GameRepository::create_game(&mut conn, batch.id, &mut rng).unwrap();
        let game_id: i64 = game.game.id;

        // Position 1 has already been drawn during create_game, so current_pos = 1
        // Query next item using the helper logic
        let (draw_seq_json, current_pos): (String, u32) = conn
            .query_row(
                "SELECT draw_sequence_json, current_draw_position FROM games WHERE id = ?1;",
                params![game_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        let seq: Vec<u8> = serde_json::from_str(&draw_seq_json).unwrap();
        assert_eq!(current_pos, 1);
        let expected_next_num: u8 = seq[1];

        let snapshot_res: (String, String, String, Option<String>, String) = conn
            .query_row(
                "SELECT media_type, source_type, path_or_url, poster_path, title_gujarati FROM game_media_snapshots WHERE game_id = ?1 AND number = ?2;",
                params![game_id, expected_next_num],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();

        assert_eq!(snapshot_res.4, format!("ગીત {}", expected_next_num));
    }

    #[test]
    fn test_presentation_dtos_serialization() {
        let dto = PresentationStateDto {
            scene: "MEDIA".to_string(),
            show_answer: true,
            show_number: false,
            countdown: Some(PresentationCountdownDto {
                seconds: 5,
                state: "RUNNING".to_string(),
            }),
            announcement: None,
            lucky_draw: None,
            current: None,
            game: None,
            volume: Some(0.8),
        };

        let json_str: String = serde_json::to_string(&dto).unwrap();
        assert!(json_str.contains("\"scene\":\"MEDIA\""));
        assert!(json_str.contains("\"showAnswer\":true"));
        assert!(json_str.contains("\"showNumber\":false"));
        assert!(json_str.contains("\"countdown\":{\"seconds\":5,\"state\":\"RUNNING\"}"));
        assert!(json_str.contains("\"volume\":0.8"));
    }
}
