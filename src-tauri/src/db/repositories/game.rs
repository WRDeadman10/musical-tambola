use crate::domain::format::GameFormat;
use crate::domain::game::{CalledItem, GameSchemeState, GameState, GameStatus, GameSummary};
use crate::domain::random::RandomSource;
use crate::domain::schemes::SchemeType;
use crate::domain::tambola::draw_engine::generate_draw_sequence_for_count;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::str::FromStr;

pub struct GameRepository;

type GameRowTuple = (
    String,
    i64,
    String,
    String,
    u32,
    u32,
    String,
    Option<String>,
    Option<String>,
    String,
    u32,
);

fn validate_draw_sequence_json(json_str: &str, expected_count: u32) -> Result<Vec<u8>, String> {
    let seq: Vec<u8> = serde_json::from_str(json_str)
        .map_err(|e| format!("Corrupted draw sequence JSON: {}", e))?;

    let count: usize = seq.len();
    if count != (expected_count as usize) {
        return Err(format!(
            "Corrupted draw sequence: expected exactly {} numbers, got {}",
            expected_count, count
        ));
    }

    let mut seen: [bool; 91] = [false; 91];
    for &num in &seq {
        if num < 1 || (num as u32) > expected_count {
            return Err(format!(
                "Corrupted draw sequence: number {} is out of valid range 1..={}",
                num, expected_count
            ));
        }
        if seen[num as usize] {
            return Err(format!(
                "Corrupted draw sequence: duplicate number {} found",
                num
            ));
        }
        seen[num as usize] = true;
    }

    Ok(seq)
}

impl GameRepository {
    /// Creates a new game session in an atomic transaction.
    /// Rejects if any game is ACTIVE or PAUSED, if any of 90 media slots is not VALID,
    /// or if no active prize schemes exist.
    pub fn create_game(
        conn: &mut Connection,
        batch_id: i64,
        rng: &mut impl RandomSource,
    ) -> Result<GameState, String> {
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        // 1. Check for existing active or paused games
        let active_count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM games WHERE status IN ('ACTIVE', 'PAUSED');",
                [],
                |r| r.get(0),
            )
            .map_err(|e| format!("Failed to check active games: {}", e))?;

        if active_count > 0 {
            return Err("An active or paused game already exists. Complete or end it before starting a new game.".to_string());
        }

        // 2. Query active format and song count
        let active_format_str: String = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_game_format';",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "CLASSIC_90".to_string());
        let active_format: GameFormat =
            GameFormat::from_db_str(&active_format_str).unwrap_or(GameFormat::Classic90);

        let active_song_count: u32 = match active_format {
            GameFormat::Classic90 => 90,
            GameFormat::SongGrid => tx
                .query_row(
                    "SELECT value FROM app_settings WHERE key = 'active_song_count';",
                    [],
                    |r| r.get(0),
                )
                .ok()
                .and_then(|s: String| s.parse::<u32>().ok())
                .unwrap_or(45),
        };

        // 3. Validate that media assets 1..=active_song_count exist and are marked 'VALID'
        let valid_media_count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM media_assets WHERE number <= ?1 AND is_valid = 1 AND last_validated_at IS NOT NULL;",
                params![active_song_count],
                |r| r.get(0),
            )
            .map_err(|e| format!("Failed to count valid media assets: {}", e))?;

        if valid_media_count != (active_song_count as i64) {
            return Err(format!(
                "Cannot start game: all {} media slots must be configured and valid (currently {} valid).",
                active_song_count, valid_media_count
            ));
        }

        // 4. Verify batch exists, check ticket count, and format match
        let (batch_ticket_count, batch_format_str, batch_song_count): (i64, String, u32) = tx
            .query_row(
                "SELECT ticket_count, format, song_count FROM ticket_batches WHERE id = ?1;",
                params![batch_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(|e| format!("Failed to check ticket batch: {}", e))?
            .ok_or_else(|| format!("Ticket batch {} not found.", batch_id))?;

        if batch_ticket_count == 0 {
            return Err("Cannot start game: selected batch contains no tickets.".to_string());
        }

        let batch_format: GameFormat =
            GameFormat::from_db_str(&batch_format_str).unwrap_or(GameFormat::Classic90);
        if batch_format != active_format || batch_song_count != active_song_count {
            return Err(format!(
                "BATCH_FORMAT_MISMATCH: Batch format {} ({} songs) does not match active game format {} ({} songs)",
                batch_format.as_str(),
                batch_song_count,
                active_format.as_str(),
                active_song_count
            ));
        }

        // 5. Verify that at least one active prize scheme exists
        let active_schemes_count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM prize_schemes WHERE is_active = 1;",
                [],
                |r| r.get(0),
            )
            .map_err(|e| format!("Failed to count active prize schemes: {}", e))?;

        if active_schemes_count == 0 {
            return Err(
                "Cannot start game: at least one active prize scheme is required.".to_string(),
            );
        }

        // 6. Generate monotonic game code G-YYYY-NNN per year
        let year: String = tx
            .query_row("SELECT strftime('%Y', 'now');", [], |r| r.get(0))
            .map_err(|e| format!("Failed to query current year: {}", e))?;

        let seq_key: String = format!("game_sequence_{}", year);
        let current_seq_str: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1;",
                params![seq_key],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to read game sequence: {}", e))?;

        let next_seq: u64 = match current_seq_str {
            Some(s) => {
                s.trim().parse::<u64>().map_err(|e| {
                    format!(
                        "Corrupted game sequence counter in app_settings for key '{}': {}",
                        seq_key, e
                    )
                })? + 1
            }
            None => 1,
        };

        tx.execute(
            "INSERT INTO app_settings (key, value, updated_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
            params![seq_key, next_seq.to_string()],
        )
        .map_err(|e| format!("Failed to update game sequence: {}", e))?;

        let game_code: String = format!("G-{}-{:03}", year, next_seq);

        // 7. Generate draw sequence for active song count
        let draw_seq: Vec<u8> = generate_draw_sequence_for_count(active_song_count, rng);
        let draw_seq_json: String = serde_json::to_string(&draw_seq)
            .map_err(|e| format!("Failed to serialize draw sequence: {}", e))?;

        // 8. Insert game session (status ACTIVE, draw position 1)
        tx.execute(
            "INSERT INTO games (game_code, status, ticket_batch_id, draw_sequence_json, current_draw_position, format, song_count, started_at, created_at)
             VALUES (?1, 'ACTIVE', ?2, ?3, 1, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);",
            params![game_code, batch_id, draw_seq_json, active_format.as_str(), active_song_count],
        )
        .map_err(|e| format!("Failed to insert game: {}", e))?;

        let game_id: i64 = tx.last_insert_rowid();

        // 9. Snapshot game tickets
        tx.execute(
            "INSERT INTO game_tickets (game_id, ticket_id)
             SELECT ?1, id FROM tickets WHERE batch_id = ?2;",
            params![game_id, batch_id],
        )
        .map_err(|e| format!("Failed to snapshot game tickets: {}", e))?;

        // 10. Snapshot media assets up to active song count
        tx.execute(
            "INSERT INTO game_media_snapshots (game_id, number, media_type, source_type, path_or_url, poster_path, title_gujarati)
             SELECT ?1, number, media_type, source_type, path_or_url, poster_path, title_gujarati FROM media_assets WHERE number <= ?2;",
            params![game_id, active_song_count],
        )
        .map_err(|e| format!("Failed to snapshot media assets: {}", e))?;

        // 11. Snapshot active prize schemes
        tx.execute(
            "INSERT INTO game_schemes (game_id, scheme_id, name, scheme_type, rule_config_json, max_winners)
             SELECT ?1, id, name, scheme_type, rule_config_json, max_winners FROM prize_schemes WHERE is_active = 1
             ORDER BY display_order ASC, id ASC;",
            params![game_id],
        )
        .map_err(|e| format!("Failed to snapshot game schemes: {}", e))?;

        // 12. Record first draw call (position 1)
        let first_number: u8 = draw_seq[0];
        tx.execute(
            "INSERT INTO game_calls (game_id, draw_position, number, drawn_at)
             VALUES (?1, 1, ?2, CURRENT_TIMESTAMP);",
            params![game_id, first_number],
        )
        .map_err(|e| format!("Failed to record first draw call: {}", e))?;

        tx.commit()
            .map_err(|e| format!("Failed to commit create_game transaction: {}", e))?;

        Self::get_game(conn, game_id)
    }

    /// Advances game to the next number in the persisted sequence.
    /// Strictly idempotent: if current_draw_position != expected_position, returns the current GameState unchanged.
    pub fn draw_next(
        conn: &mut Connection,
        game_id: i64,
        expected_position: u32,
    ) -> Result<GameState, String> {
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        let (status_str, current_pos, draw_seq_json, song_count): (String, u32, String, u32) = tx
            .query_row(
                "SELECT status, current_draw_position, draw_sequence_json, song_count FROM games WHERE id = ?1;",
                params![game_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(|e| format!("Failed to load game {}: {}", game_id, e))?;

        if status_str == "PAUSED" {
            return Err("Game is paused. Resume the game before drawing next number.".to_string());
        }

        if status_str != "ACTIVE" {
            return Err(format!(
                "Cannot draw next number: game is in {} status",
                status_str
            ));
        }

        // Idempotence check: if position already moved past expected_position, return current state without drawing
        if current_pos != expected_position {
            log::info!(
                "draw_next idempotent skip for game {}: current position {} != expected position {}",
                game_id, current_pos, expected_position
            );
            drop(tx);
            return Self::get_game(conn, game_id);
        }

        if current_pos >= song_count {
            return Err(format!(
                "All {} numbers have already been drawn.",
                song_count
            ));
        }

        let draw_seq: Vec<u8> = validate_draw_sequence_json(&draw_seq_json, song_count)?;

        let next_pos: u32 = current_pos + 1;
        let next_number: u8 = draw_seq[(next_pos - 1) as usize];

        tx.execute(
            "INSERT INTO game_calls (game_id, draw_position, number, drawn_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP);",
            params![game_id, next_pos, next_number],
        )
        .map_err(|e| format!("Failed to insert game call at position {}: {}", next_pos, e))?;

        if next_pos >= song_count {
            tx.execute(
                "UPDATE games SET current_draw_position = ?2, status = 'COMPLETED', ended_at = CURRENT_TIMESTAMP WHERE id = ?1;",
                params![game_id, song_count],
            )
            .map_err(|e| format!("Failed to update completed game status: {}", e))?;
        } else {
            tx.execute(
                "UPDATE games SET current_draw_position = ?2 WHERE id = ?1;",
                params![game_id, next_pos],
            )
            .map_err(|e| format!("Failed to update draw position: {}", e))?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit draw_next transaction: {}", e))?;

        Self::get_game(conn, game_id)
    }

    /// Pauses an active game session using a single atomic conditional UPDATE.
    pub fn pause_game(conn: &mut Connection, game_id: i64) -> Result<GameState, String> {
        let rows_affected: usize = conn
            .execute(
                "UPDATE games SET status = 'PAUSED' WHERE id = ?1 AND status = 'ACTIVE';",
                params![game_id],
            )
            .map_err(|e| format!("Failed to pause game: {}", e))?;

        if rows_affected == 0 {
            let status_opt: Option<String> = conn
                .query_row(
                    "SELECT status FROM games WHERE id = ?1;",
                    params![game_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| format!("Database error checking game: {}", e))?;

            return match status_opt {
                Some(status) => Err(format!(
                    "Cannot pause game: game is currently in {} status",
                    status
                )),
                None => Err(format!("Game {} not found", game_id)),
            };
        }

        Self::get_game(conn, game_id)
    }

    /// Resumes a paused game session using a single atomic conditional UPDATE.
    pub fn resume_game(conn: &mut Connection, game_id: i64) -> Result<GameState, String> {
        let rows_affected: usize = conn
            .execute(
                "UPDATE games SET status = 'ACTIVE' WHERE id = ?1 AND status = 'PAUSED';",
                params![game_id],
            )
            .map_err(|e| format!("Failed to resume game: {}", e))?;

        if rows_affected == 0 {
            let status_opt: Option<String> = conn
                .query_row(
                    "SELECT status FROM games WHERE id = ?1;",
                    params![game_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| format!("Database error checking game: {}", e))?;

            return match status_opt {
                Some(status) => Err(format!(
                    "Cannot resume game: game is currently in {} status",
                    status
                )),
                None => Err(format!("Game {} not found", game_id)),
            };
        }

        Self::get_game(conn, game_id)
    }

    /// Ends an active or paused game session permanently using a single atomic conditional UPDATE.
    pub fn end_game(conn: &mut Connection, game_id: i64) -> Result<GameState, String> {
        let rows_affected: usize = conn
            .execute(
                "UPDATE games SET status = 'ENDED', ended_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status IN ('ACTIVE', 'PAUSED');",
                params![game_id],
            )
            .map_err(|e| format!("Failed to end game: {}", e))?;

        if rows_affected == 0 {
            let status_opt: Option<String> = conn
                .query_row(
                    "SELECT status FROM games WHERE id = ?1;",
                    params![game_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| format!("Database error checking game: {}", e))?;

            return match status_opt {
                Some(status) => Err(format!(
                    "Cannot end game: game is already in {} status",
                    status
                )),
                None => Err(format!("Game {} not found", game_id)),
            };
        }

        Self::get_game(conn, game_id)
    }

    /// Recovers the current active or paused game session (if any).
    pub fn get_active_game(conn: &Connection) -> Result<Option<GameState>, String> {
        let game_id_opt: Option<i64> = conn
            .query_row(
                "SELECT id FROM games WHERE status IN ('ACTIVE', 'PAUSED') ORDER BY id DESC LIMIT 1;",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query active game: {}", e))?;

        match game_id_opt {
            Some(id) => Self::get_game(conn, id).map(Some),
            None => Ok(None),
        }
    }

    /// Retrieves complete GameState for any game by id.
    pub fn get_game(conn: &Connection, game_id: i64) -> Result<GameState, String> {
        let (
            code,
            batch_id,
            batch_name,
            status_str,
            current_pos,
            total_tickets,
            created_at,
            started_at,
            ended_at,
            format_str,
            song_count,
        ): GameRowTuple = conn
            .query_row(
                "SELECT g.game_code, g.ticket_batch_id, b.name, g.status, g.current_draw_position,
                        (SELECT COUNT(*) FROM game_tickets gt WHERE gt.game_id = g.id),
                        g.created_at, g.started_at, g.ended_at, g.format, g.song_count
                 FROM games g
                 JOIN ticket_batches b ON g.ticket_batch_id = b.id
                 WHERE g.id = ?1;",
                params![game_id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                        r.get(7)?,
                        r.get(8)?,
                        r.get(9)?,
                        r.get(10)?,
                    ))
                },
            )
            .map_err(|e| format!("Game {} not found: {}", game_id, e))?;

        let status: GameStatus = GameStatus::from_db_str(&status_str)?;
        let format: GameFormat =
            GameFormat::from_db_str(&format_str).unwrap_or(GameFormat::Classic90);

        // Validate persisted draw sequence JSON integrity
        let draw_seq_json: String = conn
            .query_row(
                "SELECT draw_sequence_json FROM games WHERE id = ?1;",
                params![game_id],
                |r| r.get(0),
            )
            .map_err(|e| format!("Failed to read draw sequence for game {}: {}", game_id, e))?;

        validate_draw_sequence_json(&draw_seq_json, song_count)?;

        let game_summary = GameSummary {
            id: game_id,
            code,
            batch_id,
            batch_name,
            status,
            current_draw_position: current_pos,
            total_tickets,
            format,
            song_count,
            total_numbers: song_count,
            created_at,
            started_at,
            ended_at,
        };

        // Query chronological draw history with snapshot media details
        let mut history_stmt = conn
            .prepare(
                "SELECT c.draw_position, c.number, s.title_gujarati, s.media_type, s.source_type,
                        s.path_or_url, s.poster_path, c.drawn_at
                 FROM game_calls c
                 JOIN game_media_snapshots s ON c.game_id = s.game_id AND c.number = s.number
                 WHERE c.game_id = ?1
                 ORDER BY c.draw_position ASC;",
            )
            .map_err(|e| format!("Failed to prepare history query: {}", e))?;

        let rows = history_stmt
            .query_map(params![game_id], |r| {
                let position: u32 = r.get(0)?;
                let number: u8 = r.get(1)?;
                let title: String = r.get(2)?;
                let media_type: String = r.get(3)?;
                let source_type: String = r.get(4)?;
                let path_or_url: String = r.get(5)?;
                let poster_path: Option<String> = r.get(6)?;
                let called_at: String = r.get(7)?;

                let (display_url, thumbnail_url) = if source_type == "LOCAL" {
                    let disp = format!("http://media.localhost/game/{}/{}", game_id, number);
                    let thumb = if poster_path.is_some() {
                        Some(format!(
                            "http://media.localhost/game/{}/{}/thumb",
                            game_id, number
                        ))
                    } else {
                        Some(disp.clone())
                    };
                    (disp, thumb)
                } else {
                    (path_or_url, poster_path)
                };

                Ok(CalledItem {
                    position,
                    number,
                    title,
                    media_type,
                    source_type,
                    display_url,
                    thumbnail_url,
                    called_at,
                })
            })
            .map_err(|e| format!("Failed to query call history: {}", e))?;

        let mut history: Vec<CalledItem> = Vec::new();
        for row in rows {
            history.push(row.map_err(|e| format!("Error mapping call history row: {}", e))?);
        }

        let current: Option<CalledItem> = history.last().cloned();

        // Query snapshot schemes
        let mut schemes_stmt = conn
            .prepare(
                "SELECT s.scheme_id, s.name, s.scheme_type, s.max_winners,
                        (SELECT COUNT(*) FROM winners w WHERE w.game_id = s.game_id AND w.scheme_id = s.scheme_id)
                 FROM game_schemes s
                 WHERE s.game_id = ?1
                 ORDER BY s.id ASC;",
            )
            .map_err(|e| format!("Failed to prepare schemes query: {}", e))?;

        let scheme_rows = schemes_stmt
            .query_map(params![game_id], |r| {
                let scheme_id: i64 = r.get(0)?;
                let name: String = r.get(1)?;
                let scheme_type_str: String = r.get(2)?;
                let max_winners: u32 = r.get(3)?;
                let winners_count: u32 = r.get(4)?;

                let scheme_type: SchemeType =
                    SchemeType::from_str(&scheme_type_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e,
                        )))
                    })?;

                let is_full_house: bool = scheme_type == SchemeType::FullHouse;

                Ok(GameSchemeState {
                    scheme_id,
                    name,
                    scheme_type,
                    max_winners,
                    winners_count,
                    is_full_house,
                })
            })
            .map_err(|e| format!("Failed to query game schemes: {}", e))?;

        let mut schemes: Vec<GameSchemeState> = Vec::new();
        for s in scheme_rows {
            schemes.push(s.map_err(|e| format!("Error mapping game scheme row: {}", e))?);
        }

        Ok(GameState {
            game: game_summary,
            current,
            history,
            schemes,
        })
    }

    /// Lists games descending with limit.
    pub fn list_games(conn: &Connection, limit: u32) -> Result<Vec<GameSummary>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT g.id, g.game_code, g.ticket_batch_id, b.name, g.status, g.current_draw_position,
                        (SELECT COUNT(*) FROM game_tickets gt WHERE gt.game_id = g.id),
                        g.created_at, g.started_at, g.ended_at, g.format, g.song_count
                 FROM games g
                 JOIN ticket_batches b ON g.ticket_batch_id = b.id
                 ORDER BY g.id DESC
                 LIMIT ?1;",
            )
            .map_err(|e| format!("Failed to prepare list_games query: {}", e))?;

        let rows = stmt
            .query_map(params![limit], |r| {
                let id: i64 = r.get(0)?;
                let code: String = r.get(1)?;
                let batch_id: i64 = r.get(2)?;
                let batch_name: String = r.get(3)?;
                let status_str: String = r.get(4)?;
                let current_pos: u32 = r.get(5)?;
                let total_tickets: u32 = r.get(6)?;
                let created_at: String = r.get(7)?;
                let started_at: Option<String> = r.get(8)?;
                let ended_at: Option<String> = r.get(9)?;
                let format_str: String = r.get(10)?;
                let format: GameFormat =
                    GameFormat::from_db_str(&format_str).unwrap_or(GameFormat::Classic90);
                let song_count: u32 = r.get(11)?;

                let status: GameStatus = match GameStatus::from_db_str(&status_str) {
                    Ok(s) => s,
                    Err(_) => GameStatus::Ready,
                };

                Ok(GameSummary {
                    id,
                    code,
                    batch_id,
                    batch_name,
                    status,
                    current_draw_position: current_pos,
                    total_tickets,
                    format,
                    song_count,
                    total_numbers: song_count,
                    created_at,
                    started_at,
                    ended_at,
                })
            })
            .map_err(|e| format!("Failed to execute list_games: {}", e))?;

        let mut list: Vec<GameSummary> = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| format!("Error mapping game summary: {}", e))?);
        }

        Ok(list)
    }

    /// Builds a map of number -> draw_position from persisted game_calls.
    pub fn build_call_position_map(
        conn: &Connection,
        game_id: i64,
    ) -> Result<HashMap<u8, u32>, String> {
        let mut stmt = conn
            .prepare("SELECT number, draw_position FROM game_calls WHERE game_id = ?1;")
            .map_err(|e| format!("Failed to prepare call positions query: {}", e))?;

        let rows = stmt
            .query_map(params![game_id], |r| {
                let num: u8 = r.get(0)?;
                let pos: u32 = r.get(1)?;
                Ok((num, pos))
            })
            .map_err(|e| format!("Failed to query call positions: {}", e))?;

        let mut map: HashMap<u8, u32> = HashMap::new();
        for r in rows {
            let (num, pos) = r.map_err(|e| format!("Error mapping call position: {}", e))?;
            map.insert(num, pos);
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::media::MediaRepository;
    use crate::db::repositories::ticket::TicketRepository;
    use crate::domain::media::{MediaType, SaveMediaInput, SourceType};
    use crate::domain::random::DeterministicRandomSource;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn populate_valid_media_and_batch(conn: &mut Connection) -> i64 {
        // Populate all 90 media assets with VALID status
        for num in 1..=90 {
            MediaRepository::upsert(
                conn,
                &SaveMediaInput {
                    number: num,
                    media_type: MediaType::Image,
                    source_type: SourceType::Local,
                    source: format!("C:/media/item_{}.jpg", num),
                    thumbnail_source: None,
                    title: format!("વસ્તુ {}", num),
                },
            )
            .unwrap();

            conn.execute(
                "UPDATE media_assets SET is_valid = 1, last_validated_at = CURRENT_TIMESTAMP WHERE number = ?1;",
                params![num],
            )
            .unwrap();
        }

        // Generate a ticket batch with 5 tickets
        let mut rng = DeterministicRandomSource::new(1234);
        let batch = TicketRepository::generate_batch(
            conn,
            "Test Batch",
            5,
            None,
            None,
            &mut rng,
            |_, _| {},
        )
        .unwrap();
        batch.id
    }

    #[test]
    fn test_create_game_validations_and_rejections() {
        let mut conn = setup_test_db();
        let mut rng = DeterministicRandomSource::new(1);

        // 1. Rejected if media not populated / invalid
        let err_media = GameRepository::create_game(&mut conn, 1, &mut rng).unwrap_err();
        assert!(err_media.contains("all 90 media slots must be configured and valid"));

        // Now populate valid media & batch
        let batch_id = populate_valid_media_and_batch(&mut conn);

        // 2. Reject if no active schemes exist
        conn.execute("UPDATE prize_schemes SET is_active = 0;", [])
            .unwrap();
        let err_schemes = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap_err();
        assert!(err_schemes.contains("at least one active prize scheme is required"));

        // Reactivate schemes
        conn.execute("UPDATE prize_schemes SET is_active = 1;", [])
            .unwrap();

        // 3. Successful create_game
        let state = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        assert_eq!(state.game.status, GameStatus::Active);
        assert_eq!(state.game.current_draw_position, 1);
        assert_eq!(state.history.len(), 1);
        assert!(state.current.is_some());
        assert_eq!(state.current.as_ref().unwrap().position, 1);
        assert_eq!(state.game.total_tickets, 5);

        // 4. Reject second game while active
        let err_second = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap_err();
        assert!(err_second.contains("An active or paused game already exists"));
    }

    #[test]
    fn test_draw_next_progression_and_completion_at_90() {
        let mut conn = setup_test_db();
        let batch_id = populate_valid_media_and_batch(&mut conn);
        let mut rng = DeterministicRandomSource::new(777);

        let initial_state = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id = initial_state.game.id;
        assert_eq!(initial_state.game.current_draw_position, 1);

        // Advance draws from 2 to 90
        for pos in 2..=90 {
            let state = GameRepository::draw_next(&mut conn, game_id, pos - 1).unwrap();
            assert_eq!(state.game.current_draw_position, pos);
            assert_eq!(state.history.len(), pos as usize);
            assert_eq!(state.current.as_ref().unwrap().position, pos);

            if pos < 90 {
                assert_eq!(state.game.status, GameStatus::Active);
            } else {
                assert_eq!(state.game.status, GameStatus::Completed);
                assert!(state.game.ended_at.is_some());
            }
        }

        // Attempting 91st draw must be rejected
        let err_91 = GameRepository::draw_next(&mut conn, game_id, 90).unwrap_err();
        assert!(
            err_91.contains("All 90 numbers have already been drawn")
                || err_91.contains("COMPLETED")
        );
    }

    #[test]
    fn test_draw_next_idempotence() {
        let mut conn = setup_test_db();
        let batch_id = populate_valid_media_and_batch(&mut conn);
        let mut rng = DeterministicRandomSource::new(4321);

        let state1 = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id = state1.game.id;
        assert_eq!(state1.game.current_draw_position, 1);

        // First draw: expected_position 1 -> advances to pos 2
        let state2 = GameRepository::draw_next(&mut conn, game_id, 1).unwrap();
        assert_eq!(state2.game.current_draw_position, 2);
        assert_eq!(state2.history.len(), 2);

        // Duplicate call with stale expected_position 1 (e.g. double click) -> idempotent skip
        let state_dup = GameRepository::draw_next(&mut conn, game_id, 1).unwrap();
        assert_eq!(state_dup.game.current_draw_position, 2);
        assert_eq!(state_dup.history.len(), 2);

        // Calls count in DB must still be exactly 2
        let calls_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM game_calls WHERE game_id = ?1;",
                params![game_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(calls_count, 2);
    }

    #[test]
    fn test_pause_and_resume_lifecycle() {
        let mut conn = setup_test_db();
        let batch_id = populate_valid_media_and_batch(&mut conn);
        let mut rng = DeterministicRandomSource::new(888);

        let state = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id = state.game.id;

        // Pause
        let paused_state = GameRepository::pause_game(&mut conn, game_id).unwrap();
        assert_eq!(paused_state.game.status, GameStatus::Paused);

        // Draw while paused must be rejected
        let err_draw = GameRepository::draw_next(&mut conn, game_id, 1).unwrap_err();
        assert!(err_draw.contains("Game is paused"));

        // Resume
        let resumed_state = GameRepository::resume_game(&mut conn, game_id).unwrap();
        assert_eq!(resumed_state.game.status, GameStatus::Active);

        // Draw next succeeds after resume
        let next_state = GameRepository::draw_next(&mut conn, game_id, 1).unwrap();
        assert_eq!(next_state.game.current_draw_position, 2);

        // End game
        let ended_state = GameRepository::end_game(&mut conn, game_id).unwrap();
        assert_eq!(ended_state.game.status, GameStatus::Ended);
        assert!(ended_state.game.ended_at.is_some());

        // Ending an already ended game must be rejected
        let err_already_ended = GameRepository::end_game(&mut conn, game_id).unwrap_err();
        assert!(err_already_ended.contains("Cannot end game: game is already in ENDED status"));
    }

    #[test]
    fn test_corrupted_sequence_handling() {
        let mut conn = setup_test_db();
        let batch_id = populate_valid_media_and_batch(&mut conn);
        let mut rng = DeterministicRandomSource::new(111);

        let state = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id = state.game.id;

        // Corrupt sequence JSON (length 3 instead of 90)
        conn.execute(
            "UPDATE games SET draw_sequence_json = '[1, 2, 3]' WHERE id = ?1;",
            params![game_id],
        )
        .unwrap();

        // get_game must fail gracefully with descriptive error
        let err_get = GameRepository::get_game(&conn, game_id).unwrap_err();
        assert!(err_get.contains("Corrupted draw sequence: expected exactly 90 numbers"));

        // draw_next must also fail gracefully
        let err_draw = GameRepository::draw_next(&mut conn, game_id, 1).unwrap_err();
        assert!(err_draw.contains("Corrupted draw sequence: expected exactly 90 numbers"));
    }

    #[test]
    fn test_recovery_from_database_state() {
        let mut conn = setup_test_db();
        let batch_id = populate_valid_media_and_batch(&mut conn);
        let mut rng = DeterministicRandomSource::new(999);

        let state = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id = state.game.id;

        // Draw 10 numbers total (pos 1 to 10)
        for p in 1..10 {
            GameRepository::draw_next(&mut conn, game_id, p).unwrap();
        }

        // Query active game directly (simulating app relaunch / recovery)
        let recovered_opt = GameRepository::get_active_game(&conn).unwrap();
        assert!(recovered_opt.is_some());
        let recovered = recovered_opt.unwrap();

        assert_eq!(recovered.game.id, game_id);
        assert_eq!(recovered.game.status, GameStatus::Active);
        assert_eq!(recovered.game.current_draw_position, 10);
        assert_eq!(recovered.history.len(), 10);
        assert_eq!(recovered.current.as_ref().unwrap().position, 10);

        // Position map check
        let pos_map = GameRepository::build_call_position_map(&conn, game_id).unwrap();
        assert_eq!(pos_map.len(), 10);
        for item in &recovered.history {
            assert_eq!(pos_map.get(&item.number), Some(&item.position));
        }
    }

    #[test]
    fn test_recovery_reopen_db_file() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir().join(format!(
            "tambola_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path: std::path::PathBuf = temp_dir.join("test_recovery.sqlite");

        let game_id: i64;
        let expected_11th_number: u8;

        {
            let mut conn = Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            run_migrations(&conn).unwrap();
            let batch_id = populate_valid_media_and_batch(&mut conn);

            let mut rng = DeterministicRandomSource::new(333);
            let state = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
            game_id = state.game.id;

            // Draw up to position 10
            for p in 1..10 {
                GameRepository::draw_next(&mut conn, game_id, p).unwrap();
            }

            // Read the persisted sequence to know the 11th number (index 10 in 0-indexed slice)
            let seq_json: String = conn
                .query_row(
                    "SELECT draw_sequence_json FROM games WHERE id = ?1;",
                    params![game_id],
                    |r| r.get(0),
                )
                .unwrap();
            let seq: Vec<u8> = serde_json::from_str(&seq_json).unwrap();
            expected_11th_number = seq[10];

            // Connection drops when exiting scope
        }

        // Reopen new connection to same database file on disk
        {
            let mut conn2 = Connection::open(&db_path).unwrap();
            conn2.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

            let active_opt = GameRepository::get_active_game(&conn2).unwrap();
            assert!(active_opt.is_some());
            let active = active_opt.unwrap();

            assert_eq!(active.game.id, game_id);
            assert_eq!(active.game.current_draw_position, 10);
            assert_eq!(active.history.len(), 10);
            assert_eq!(active.current.as_ref().unwrap().position, 10);

            // Draw 11th call on the reopened connection with expected_position 10
            let next_state = GameRepository::draw_next(&mut conn2, game_id, 10).unwrap();
            assert_eq!(next_state.game.current_draw_position, 11);
            assert_eq!(
                next_state.current.as_ref().unwrap().number,
                expected_11th_number
            );
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_snapshot_immutability() {
        let mut conn = setup_test_db();
        let batch_id = populate_valid_media_and_batch(&mut conn);
        let mut rng = DeterministicRandomSource::new(555);

        let state = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id = state.game.id;
        let original_title = state.history[0].title.clone();

        // Mutate original media row and prize scheme in master tables
        conn.execute(
            "UPDATE media_assets SET title_gujarati = 'બદલાયેલું શીર્ષક' WHERE number = ?1;",
            params![state.history[0].number],
        )
        .unwrap();

        conn.execute(
            "UPDATE prize_schemes SET max_winners = 999 WHERE id = ?1;",
            params![state.schemes[0].scheme_id],
        )
        .unwrap();

        // Reload game: snapshot values must remain completely unchanged
        let reloaded = GameRepository::get_game(&conn, game_id).unwrap();
        assert_eq!(reloaded.history[0].title, original_title);
        assert_eq!(
            reloaded.schemes[0].max_winners,
            state.schemes[0].max_winners
        );
        assert_ne!(reloaded.schemes[0].max_winners, 999);
    }

    #[test]
    fn test_song_grid_game_lifecycle_45_numbers() {
        let mut conn = setup_test_db();
        conn.execute(
            "UPDATE app_settings SET value = 'SONG_GRID' WHERE key = 'active_game_format';",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE app_settings SET value = '45' WHERE key = 'active_song_count';",
            [],
        )
        .unwrap();

        for num in 1..=45 {
            MediaRepository::upsert(
                &conn,
                &SaveMediaInput {
                    number: num,
                    media_type: MediaType::Image,
                    source_type: SourceType::Local,
                    source: format!("C:/media/item_{}.jpg", num),
                    thumbnail_source: None,
                    title: format!("Song {}", num),
                },
            )
            .unwrap();
            conn.execute(
                "UPDATE media_assets SET is_valid = 1, last_validated_at = CURRENT_TIMESTAMP WHERE number = ?1;",
                params![num],
            )
            .unwrap();
        }

        let mut rng = DeterministicRandomSource::new(999);
        let batch = TicketRepository::generate_batch(
            &mut conn,
            "SongGrid Batch 45",
            5,
            Some(GameFormat::SongGrid),
            Some(45),
            &mut rng,
            |_, _| {},
        )
        .unwrap();

        let state = GameRepository::create_game(&mut conn, batch.id, &mut rng).unwrap();
        assert_eq!(state.game.format, GameFormat::SongGrid);
        assert_eq!(state.game.song_count, 45);
        assert_eq!(state.game.total_numbers, 45);
        assert_eq!(state.game.status, GameStatus::Active);
        assert_eq!(state.game.current_draw_position, 1);

        let game_id = state.game.id;
        for pos in 1..45 {
            let next_state = GameRepository::draw_next(&mut conn, game_id, pos).unwrap();
            if pos < 44 {
                assert_eq!(next_state.game.status, GameStatus::Active);
                assert_eq!(next_state.game.current_draw_position, pos + 1);
            } else {
                assert_eq!(next_state.game.status, GameStatus::Completed);
                assert_eq!(next_state.game.current_draw_position, 45);
                assert!(next_state.game.ended_at.is_some());
            }
        }

        let post_draw = GameRepository::draw_next(&mut conn, game_id, 45);
        assert!(post_draw.is_err());
        let post_err = post_draw.unwrap_err();
        assert!(post_err.contains("COMPLETED") || post_err.contains("All 45 numbers"));
    }

    #[test]
    fn test_format_mismatch_rejection() {
        let mut conn = setup_test_db();
        populate_valid_media_and_batch(&mut conn);

        let mut rng = DeterministicRandomSource::new(123);
        let song_batch = TicketRepository::generate_batch(
            &mut conn,
            "Song Batch",
            3,
            Some(GameFormat::SongGrid),
            Some(45),
            &mut rng,
            |_, _| {},
        )
        .unwrap();

        let res = GameRepository::create_game(&mut conn, song_batch.id, &mut rng);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("BATCH_FORMAT_MISMATCH"));
    }
}
