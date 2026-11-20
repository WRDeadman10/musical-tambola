use crate::domain::claims::claim_validation::ClaimRejectionCode;
use crate::domain::game::GameSummary;
use crate::domain::schemes::SchemeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryFilter {
    pub game_id: Option<i64>,
    pub scheme_type: Option<SchemeType>,
    pub ticket_code: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimHistoryRow {
    pub claim_id: i64,
    pub game_id: i64,
    pub game_code: String,
    pub ticket_code: String,
    pub scheme_name: Option<String>,
    pub scheme_type: Option<SchemeType>,
    pub status: String,
    pub rejection_code: Option<ClaimRejectionCode>,
    pub rejection_reason: Option<String>,
    pub current_draw_position: u32,
    pub qualification_position: Option<u32>,
    pub winner_position: Option<u32>,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WinnerRow {
    pub winner_id: i64,
    pub claim_id: i64,
    pub game_id: i64,
    pub game_code: String,
    pub ticket_code: String,
    pub scheme_name: String,
    pub scheme_type: SchemeType,
    pub winner_position: u32,
    pub max_winners: u32,
    pub is_full_house: bool,
    pub qualification_position: u32,
    pub awarded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub media_total: u32,
    pub media_valid: u32,
    pub media_invalid: u32,
    pub ticket_batches: u32,
    pub total_tickets: u32,
    pub games_played: u32,
    pub active_game: Option<GameSummary>,
    pub recent_games: Vec<GameSummary>,
    pub recent_winners: Vec<WinnerRow>,
}
