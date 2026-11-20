use crate::domain::schemes::SchemeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameStatus {
    Ready,
    Active,
    Paused,
    Completed,
    Ended,
}

impl GameStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameStatus::Ready => "READY",
            GameStatus::Active => "ACTIVE",
            GameStatus::Paused => "PAUSED",
            GameStatus::Completed => "COMPLETED",
            GameStatus::Ended => "ENDED",
        }
    }

    pub fn from_db_str(s: &str) -> Result<Self, String> {
        match s {
            "READY" => Ok(GameStatus::Ready),
            "ACTIVE" => Ok(GameStatus::Active),
            "PAUSED" => Ok(GameStatus::Paused),
            "COMPLETED" => Ok(GameStatus::Completed),
            "ENDED" => Ok(GameStatus::Ended),
            other => Err(format!("Unknown game status: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub id: i64,
    pub code: String,
    pub batch_id: i64,
    pub batch_name: String,
    pub status: GameStatus,
    pub current_draw_position: u32,
    pub total_tickets: u32,
    pub created_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalledItem {
    pub position: u32,
    pub number: u8,
    pub title: String,
    pub media_type: String,
    pub source_type: String,
    pub display_url: String,
    pub thumbnail_url: Option<String>,
    pub called_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSchemeState {
    pub scheme_id: i64,
    pub name: String,
    pub scheme_type: SchemeType,
    pub max_winners: u32,
    pub winners_count: u32,
    pub is_full_house: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub game: GameSummary,
    pub current: Option<CalledItem>,
    pub history: Vec<CalledItem>,
    pub schemes: Vec<GameSchemeState>,
}
