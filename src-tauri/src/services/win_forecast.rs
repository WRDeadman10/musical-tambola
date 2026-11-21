use crate::db::repositories::ticket::TicketRepository;
use crate::domain::claims::claim_validation::GameSchemeSnapshot;
use crate::domain::schemes::{
    ticket_to_logical_cells, LogicalTicketCells, PatternKind, PatternMask, SchemeType,
};
use crate::domain::win_forecast::{
    calculate_game_win_forecast, GameWinForecastDto, TicketForecastCandidate,
};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::str::FromStr;

pub struct WinForecastService;

impl WinForecastService {
    pub fn get_game_win_forecast(
        conn: &Connection,
        game_id: i64,
    ) -> Result<GameWinForecastDto, String> {
        let (game_code, _batch_id, format_str, _song_count, draw_seq_json): (
            String,
            i64,
            String,
            u32,
            String,
        ) = conn
            .query_row(
                "SELECT game_code, ticket_batch_id, format, song_count, draw_sequence_json FROM games WHERE id = ?1;",
                params![game_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .map_err(|e: rusqlite::Error| format!("Game {} not found: {}", game_id, e))?;

        let draw_sequence: Vec<u8> = serde_json::from_str(&draw_seq_json)
            .map_err(|e| format!("Failed to parse draw sequence JSON: {}", e))?;

        // 1. Fetch game schemes snapshot
        let mut scheme_stmt = conn
            .prepare(
                "SELECT scheme_id, name, scheme_type, pattern_kind, mask_json, early_n, is_full_house, max_winners
                 FROM game_schemes
                 WHERE game_id = ?1
                 ORDER BY id ASC;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare schemes query: {}", e))?;

        let scheme_rows = scheme_stmt
            .query_map(params![game_id], |r| {
                let scheme_id: i64 = r.get(0)?;
                let scheme_name: String = r.get(1)?;
                let scheme_type_str: String = r.get(2)?;
                let pattern_kind_str: String = r.get(3)?;
                let mask_json: String = r.get(4)?;
                let early_n: Option<u32> = r.get(5)?;
                let is_full_house: bool = r.get(6)?;
                let max_winners: u32 = r.get(7)?;

                let scheme_type: SchemeType =
                    SchemeType::from_str(&scheme_type_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e,
                        )))
                    })?;

                let pattern_kind: PatternKind =
                    PatternKind::from_str(&pattern_kind_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e,
                        )))
                    })?;

                let mask: PatternMask = serde_json::from_str(&mask_json).unwrap_or([[false; 5]; 3]);

                Ok(GameSchemeSnapshot {
                    scheme_id,
                    scheme_name,
                    scheme_type,
                    pattern_kind,
                    mask,
                    early_n,
                    is_full_house,
                    max_winners,
                })
            })
            .map_err(|e: rusqlite::Error| format!("Failed to query game schemes: {}", e))?;

        let mut schemes: Vec<GameSchemeSnapshot> = Vec::new();
        for s in scheme_rows {
            schemes.push(s.map_err(|e: rusqlite::Error| format!("Error mapping scheme: {}", e))?);
        }

        // 2. Fetch game tickets
        let mut ticket_stmt = conn
            .prepare(
                "SELECT t.id, t.ticket_code
                 FROM tickets t
                 JOIN game_tickets gt ON t.id = gt.ticket_id
                 WHERE gt.game_id = ?1
                 ORDER BY t.id ASC;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare tickets query: {}", e))?;

        let ticket_rows = ticket_stmt
            .query_map(params![game_id], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e: rusqlite::Error| format!("Failed to query tickets: {}", e))?;

        let mut tickets: Vec<TicketForecastCandidate> = Vec::new();
        for row in ticket_rows {
            let (ticket_id, ticket_code) =
                row.map_err(|e: rusqlite::Error| format!("Error mapping ticket: {}", e))?;
            let ticket_dto = TicketRepository::get_ticket(conn, &ticket_code)?;

            let cells: LogicalTicketCells = if ticket_dto.cols == 5 {
                let mut grid: LogicalTicketCells = [[0; 5]; 3];
                for cell in &ticket_dto.cells {
                    if (cell.row as usize) < 3 && (cell.col as usize) < 5 {
                        grid[cell.row as usize][cell.col as usize] = cell.number;
                    }
                }
                grid
            } else {
                let mut classic_cells: [[Option<u8>; 9]; 3] = [[None; 9]; 3];
                for cell in &ticket_dto.cells {
                    if (cell.row as usize) < 3 && (cell.col as usize) < 9 {
                        classic_cells[cell.row as usize][cell.col as usize] = Some(cell.number);
                    }
                }
                let t_classic = crate::domain::tambola::ticket_generator::Ticket {
                    cells: classic_cells,
                };
                ticket_to_logical_cells(&t_classic)
            };

            tickets.push(TicketForecastCandidate {
                ticket_id,
                ticket_code,
                cells,
            });
        }

        // 3. Fetch song titles
        let mut song_titles_by_number: HashMap<u8, String> = HashMap::new();
        let mut songs_stmt = conn
            .prepare("SELECT number, title_gujarati FROM game_media_snapshots WHERE game_id = ?1;")
            .map_err(|e: rusqlite::Error| format!("Failed to prepare titles query: {}", e))?;

        let song_rows = songs_stmt
            .query_map(params![game_id], |r| {
                Ok((r.get::<_, u8>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e: rusqlite::Error| format!("Failed to query titles: {}", e))?;

        for (num, title) in song_rows.flatten() {
            song_titles_by_number.insert(num, title);
        }

        Ok(calculate_game_win_forecast(
            game_id,
            &game_code,
            &format_str,
            &draw_sequence,
            &schemes,
            &tickets,
            &song_titles_by_number,
        ))
    }
}
