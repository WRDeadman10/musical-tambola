pub mod claim_validation;

use crate::domain::schemes::SchemeType;
use crate::domain::tambola::ticket_generator::Ticket;
pub use claim_validation::{
    evaluate_claim, normalize_ticket_code, ClaimDecision, ClaimRejectionCode, GameSchemeSnapshot,
    TicketExistingWin,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimResult {
    pub claim_id: i64,
    pub status: String, // "ACCEPTED" | "REJECTED"
    pub rejection_code: Option<ClaimRejectionCode>,
    pub rejection_reason: Option<String>,
    pub game_id: i64,
    pub ticket_code: String,
    pub scheme_id: i64,
    pub scheme_name: String,
    pub scheme_type: SchemeType,
    pub current_draw_position: u32,
    pub qualification_position: Option<u32>,
    pub winner_position: Option<u32>,
    pub max_winners: u32,
    pub ticket: Option<Ticket>,
    pub called_numbers_on_ticket: Vec<u8>,
    pub submitted_at: String,
}
