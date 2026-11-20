pub mod draw_engine;
pub mod ticket_generator;

use crate::domain::format::GameFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketCell {
    pub row: u8,
    pub col: u8,
    pub number: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketDto {
    pub id: i64,
    pub code: String,
    pub batch_id: i64,
    pub format: GameFormat,
    pub rows: u8,
    pub cols: u8,
    pub cells: Vec<TicketCell>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketBatch {
    pub id: i64,
    pub name: String,
    pub requested_count: u32,
    pub generated_count: u32,
    pub format: GameFormat,
    pub rows: u8,
    pub cols: u8,
    pub song_count: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketPage {
    pub items: Vec<TicketDto>,
    pub total: u64,
}
