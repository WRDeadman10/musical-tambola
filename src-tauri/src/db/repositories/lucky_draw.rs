use crate::domain::random::RandomSource;
use crate::errors::AppError;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckyDrawInputDto {
    pub source: String, // "NAMES" | "TICKETS"
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub exclude_previous_winners: bool,
    pub name: Option<String>,
    pub batch_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckyWinnerDto {
    pub id: i64,
    pub draw_id: i64,
    pub value: String,
    pub position: i64,
    pub drawn_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckyDrawDto {
    pub id: i64,
    pub name: String,
    pub source: String,
    pub candidate_count: i64,
    pub winners: Vec<LuckyWinnerDto>,
    pub created_at: String,
}

pub struct LuckyDrawRepository;

impl LuckyDrawRepository {
    pub fn get_by_id(conn: &Connection, draw_id: i64) -> Result<Option<LuckyDrawDto>, AppError> {
        let draw_row: Option<(i64, String, String, String)> = conn
            .query_row(
                "SELECT id, name, source, created_at FROM lucky_draws WHERE id = ?1;",
                params![draw_id],
                |r: &Row<'_>| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .ok();

        let (id, name, source, created_at) = match draw_row {
            Some(d) => d,
            None => return Ok(None),
        };

        let candidate_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM lucky_draw_entries WHERE draw_id = ?1 AND is_winner = 0;",
                params![draw_id],
                |r: &Row<'_>| r.get(0),
            )
            .unwrap_or(0);

        let mut winners: Vec<LuckyWinnerDto> = Vec::new();
        let mut w_stmt = conn
            .prepare(
                "SELECT id, draw_id, winner, draw_position, drawn_at
                 FROM lucky_draw_winners
                 WHERE draw_id = ?1
                 ORDER BY draw_position ASC;",
            )
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        let w_rows = w_stmt
            .query_map(params![draw_id], |r: &Row<'_>| {
                let w_id: i64 = r.get(0)?;
                let d_id: i64 = r.get(1)?;
                let val: String = r.get(2)?;
                let pos: i64 = r.get(3)?;
                let d_at: String = r.get(4)?;
                Ok(LuckyWinnerDto {
                    id: w_id,
                    draw_id: d_id,
                    value: val.clone(),
                    position: pos,
                    drawn_at: d_at,
                    winner: Some(val),
                    history: None,
                })
            })
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        for w in w_rows.flatten() {
            winners.push(w);
        }

        Ok(Some(LuckyDrawDto {
            id,
            name,
            source,
            candidate_count,
            winners,
            created_at,
        }))
    }

    pub fn create_draw(
        conn: &mut Connection,
        name: Option<&str>,
        source: &str,
        items: &[String],
        batch_id: Option<i64>,
        exclude_previous_winners: bool,
    ) -> Result<LuckyDrawDto, AppError> {
        let trimmed_name: String = match name {
            Some(n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => {
                let next_id: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(id), 0) + 1 FROM lucky_draws;",
                        [],
                        |r: &Row<'_>| r.get(0),
                    )
                    .unwrap_or(1);
                format!("Lucky Draw #{}", next_id)
            }
        };

        if source != "NAMES" && source != "TICKETS" {
            return Err(AppError::new(
                "LUCKY_DRAW_INVALID_SOURCE",
                format!("Invalid source '{}', must be 'NAMES' or 'TICKETS'", source),
            ));
        }

        let mut candidate_pool: Vec<String> = Vec::new();
        if source == "NAMES" {
            for it in items {
                let trimmed: &str = it.trim();
                if !trimmed.is_empty() {
                    candidate_pool.push(trimmed.to_string());
                }
            }
        } else if let Some(b_id) = batch_id {
            let mut stmt = conn
                .prepare("SELECT ticket_code FROM tickets WHERE batch_id = ?1 ORDER BY id ASC;")
                .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
            let rows = stmt
                .query_map(params![b_id], |r: &Row<'_>| r.get(0))
                .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
            for code_res in rows {
                let code: String = code_res
                    .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
                candidate_pool.push(code);
            }
        } else if !items.is_empty() {
            for it in items {
                let trimmed: &str = it.trim();
                if !trimmed.is_empty() {
                    candidate_pool.push(trimmed.to_string());
                }
            }
        } else {
            let active_batch_opt: Option<i64> = conn
                .query_row(
                    "SELECT ticket_batch_id FROM games WHERE status IN ('ACTIVE', 'PAUSED') LIMIT 1;",
                    [],
                    |r: &Row<'_>| r.get(0),
                )
                .ok();

            if let Some(b_id) = active_batch_opt {
                let mut stmt = conn
                    .prepare("SELECT ticket_code FROM tickets WHERE batch_id = ?1 ORDER BY id ASC;")
                    .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
                let rows = stmt
                    .query_map(params![b_id], |r: &Row<'_>| r.get(0))
                    .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
                for code_res in rows {
                    let code: String = code_res
                        .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
                    candidate_pool.push(code);
                }
            }
        }

        if candidate_pool.is_empty() {
            return Err(AppError::new(
                "LUCKY_DRAW_EMPTY_ITEMS",
                "At least one item is required to create a lucky draw",
            ));
        }

        if exclude_previous_winners {
            let mut previous_winners: Vec<String> = Vec::new();
            let mut stmt = conn
                .prepare("SELECT DISTINCT winner FROM lucky_draw_winners;")
                .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
            let rows = stmt
                .query_map([], |r: &Row<'_>| r.get(0))
                .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
            for val in rows.flatten() {
                previous_winners.push(val);
            }

            if source == "TICKETS" {
                let mut stmt_t = conn
                    .prepare(
                        "SELECT t.ticket_code FROM winners w JOIN tickets t ON w.ticket_id = t.id;",
                    )
                    .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
                let rows_t = stmt_t
                    .query_map([], |r: &Row<'_>| r.get(0))
                    .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
                for val in rows_t.flatten() {
                    previous_winners.push(val);
                }
            }

            candidate_pool.retain(|c: &String| !previous_winners.contains(c));
        }

        if candidate_pool.is_empty() {
            return Err(AppError::new(
                "LUCKY_DRAW_ALL_EXCLUDED",
                "All items in the pool were excluded as previous winners",
            ));
        }

        let tx = conn.unchecked_transaction().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Transaction error: {}", e))
        })?;

        tx.execute(
            "INSERT INTO lucky_draws (name, source, batch_id, exclude_previous_winners)
             VALUES (?1, ?2, ?3, ?4);",
            params![trimmed_name, source, batch_id, exclude_previous_winners],
        )
        .map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Failed to insert lucky draw: {}", e))
        })?;

        let draw_id: i64 = tx.last_insert_rowid();

        for it in &candidate_pool {
            tx.execute(
                "INSERT INTO lucky_draw_entries (draw_id, entry_value, is_winner)
                 VALUES (?1, ?2, 0);",
                params![draw_id, it],
            )
            .map_err(|e: rusqlite::Error| {
                AppError::database_error(format!("Failed to insert draw entry: {}", e))
            })?;
        }

        tx.commit().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Commit error: {}", e))
        })?;

        Self::get_by_id(conn, draw_id)?
            .ok_or_else(|| AppError::database_error("Failed to reload created lucky draw"))
    }

    pub fn draw_winner<R: RandomSource>(
        conn: &mut Connection,
        draw_id: i64,
        rng: &mut R,
    ) -> Result<LuckyWinnerDto, AppError> {
        let draw_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM lucky_draws WHERE id = ?1;",
                params![draw_id],
                |r: &Row<'_>| r.get::<_, i64>(0),
            )
            .map(|c: i64| c > 0)
            .unwrap_or(false);

        if !draw_exists {
            return Err(AppError::new(
                "LUCKY_DRAW_NOT_FOUND",
                format!("Lucky draw with id {} not found", draw_id),
            ));
        }

        let mut stmt = conn
            .prepare("SELECT id, entry_value FROM lucky_draw_entries WHERE draw_id = ?1 AND is_winner = 0;")
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        let rows = stmt
            .query_map(params![draw_id], |r: &Row<'_>| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        let mut available: Vec<(i64, String)> = Vec::new();
        for r in rows {
            available
                .push(r.map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?);
        }

        if available.is_empty() {
            return Err(AppError::new(
                "LUCKY_DRAW_EXHAUSTED",
                "All candidates in this lucky draw have already won",
            ));
        }

        let winner_idx: usize = rng.next_index(available.len());
        let (entry_id, winner_val): (i64, String) = available[winner_idx].clone();

        let tx = conn.unchecked_transaction().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Transaction error: {}", e))
        })?;

        tx.execute(
            "UPDATE lucky_draw_entries SET is_winner = 1, drawn_at = CURRENT_TIMESTAMP WHERE id = ?1;",
            params![entry_id],
        )
        .map_err(|e: rusqlite::Error| AppError::database_error(format!("Failed to update entry: {}", e)))?;

        let current_winners_count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM lucky_draw_winners WHERE draw_id = ?1;",
                params![draw_id],
                |r: &Row<'_>| r.get(0),
            )
            .unwrap_or(0);

        let draw_position: i64 = current_winners_count + 1;

        tx.execute(
            "INSERT INTO lucky_draw_winners (draw_id, winner, draw_position)
             VALUES (?1, ?2, ?3);",
            params![draw_id, winner_val, draw_position],
        )
        .map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Failed to record winner: {}", e))
        })?;

        let winner_record_id: i64 = tx.last_insert_rowid();

        let drawn_at: String = tx
            .query_row(
                "SELECT drawn_at FROM lucky_draw_winners WHERE id = ?1;",
                params![winner_record_id],
                |r: &Row<'_>| r.get(0),
            )
            .unwrap_or_else(|_| "now".to_string());

        tx.commit().map_err(|e: rusqlite::Error| {
            AppError::database_error(format!("Commit error: {}", e))
        })?;

        let mut history: Vec<String> = Vec::new();
        let mut hist_stmt = conn
            .prepare("SELECT winner FROM lucky_draw_winners WHERE draw_id = ?1 ORDER BY draw_position ASC;")
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
        let hist_rows = hist_stmt
            .query_map(params![draw_id], |r: &Row<'_>| r.get(0))
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        for h in hist_rows {
            history.push(h.map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?);
        }

        Ok(LuckyWinnerDto {
            id: winner_record_id,
            draw_id,
            value: winner_val.clone(),
            position: draw_position,
            drawn_at,
            winner: Some(winner_val),
            history: Some(history),
        })
    }

    pub fn draw_winner_from_input<R: RandomSource>(
        conn: &mut Connection,
        input: &LuckyDrawInputDto,
        rng: &mut R,
    ) -> Result<LuckyWinnerDto, AppError> {
        let draw: LuckyDrawDto = Self::create_draw(
            conn,
            input.name.as_deref(),
            &input.source,
            &input.items,
            input.batch_id,
            input.exclude_previous_winners,
        )?;

        Self::draw_winner(conn, draw.id, rng)
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<LuckyDrawDto>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, source, created_at
                 FROM lucky_draws
                 ORDER BY id DESC;",
            )
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        let rows = stmt
            .query_map([], |r: &Row<'_>| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

        let mut list: Vec<LuckyDrawDto> = Vec::new();
        for r_res in rows {
            let (id, name, source, created_at) =
                r_res.map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;

            let candidate_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM lucky_draw_entries WHERE draw_id = ?1 AND is_winner = 0;",
                    params![id],
                    |r: &Row<'_>| r.get(0),
                )
                .unwrap_or(0);

            let mut winners: Vec<LuckyWinnerDto> = Vec::new();
            let mut w_stmt = conn
                .prepare(
                    "SELECT id, draw_id, winner, draw_position, drawn_at
                     FROM lucky_draw_winners
                     WHERE draw_id = ?1
                     ORDER BY draw_position ASC;",
                )
                .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
            let w_rows = w_stmt
                .query_map(params![id], |r: &Row<'_>| {
                    let w_id: i64 = r.get(0)?;
                    let d_id: i64 = r.get(1)?;
                    let val: String = r.get(2)?;
                    let pos: i64 = r.get(3)?;
                    let d_at: String = r.get(4)?;
                    Ok(LuckyWinnerDto {
                        id: w_id,
                        draw_id: d_id,
                        value: val.clone(),
                        position: pos,
                        drawn_at: d_at,
                        winner: Some(val),
                        history: None,
                    })
                })
                .map_err(|e: rusqlite::Error| AppError::database_error(e.to_string()))?;
            for val in w_rows.flatten() {
                winners.push(val);
            }

            list.push(LuckyDrawDto {
                id,
                name,
                source,
                candidate_count,
                winners,
                created_at,
            });
        }

        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::domain::random::DeterministicRandomSource;

    #[test]
    fn test_lucky_draw_lifecycle_and_no_repeats() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        let items: Vec<String> = vec!["Aarav".to_string(), "Diya".to_string(), "Kavya".to_string()];

        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(12345);
        let draw: LuckyDrawDto = LuckyDrawRepository::create_draw(
            &mut conn,
            Some("Grand Prize"),
            "NAMES",
            &items,
            None,
            false,
        )
        .unwrap();

        assert_eq!(draw.candidate_count, 3);
        assert_eq!(draw.winners.len(), 0);

        // Draw 1
        let res1 = LuckyDrawRepository::draw_winner(&mut conn, draw.id, &mut rng).unwrap();
        assert_eq!(res1.position, 1);
        let winner1 = res1.value;

        // Draw 2
        let res2 = LuckyDrawRepository::draw_winner(&mut conn, draw.id, &mut rng).unwrap();
        assert_eq!(res2.position, 2);
        let winner2 = res2.value;
        assert_ne!(winner1, winner2, "Must not repeat winner within same draw");

        // Draw 3
        let res3 = LuckyDrawRepository::draw_winner(&mut conn, draw.id, &mut rng).unwrap();
        assert_eq!(res3.position, 3);
        let winner3 = res3.value;
        assert_ne!(winner2, winner3);
        assert_ne!(winner1, winner3);

        // Draw 4 (Exhausted)
        let res4_err = LuckyDrawRepository::draw_winner(&mut conn, draw.id, &mut rng).unwrap_err();
        assert_eq!(res4_err.code, "LUCKY_DRAW_EXHAUSTED");

        // Verify summary persistence
        let list = LuckyDrawRepository::list_all(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].winners.len(), 3);
        assert_eq!(list[0].candidate_count, 0);
        let recorded_winners: Vec<String> =
            list[0].winners.iter().map(|w| w.value.clone()).collect();
        assert_eq!(recorded_winners, vec![winner1, winner2, winner3]);
    }

    #[test]
    fn test_lucky_draw_exclude_previous_winners() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(999);

        // Draw 1: Alice and Bob
        let items1: Vec<String> = vec!["Alice".to_string(), "Bob".to_string()];
        let draw1: LuckyDrawDto = LuckyDrawRepository::create_draw(
            &mut conn,
            Some("Draw 1"),
            "NAMES",
            &items1,
            None,
            false,
        )
        .unwrap();

        let res1 = LuckyDrawRepository::draw_winner(&mut conn, draw1.id, &mut rng).unwrap();
        let prev_winner = res1.value; // Either Alice or Bob

        // Draw 2: with Alice, Bob, and Charlie, but excludePreviousWinners = true
        let items2: Vec<String> = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let draw2: LuckyDrawDto = LuckyDrawRepository::create_draw(
            &mut conn,
            Some("Draw 2"),
            "NAMES",
            &items2,
            None,
            true,
        )
        .unwrap();

        // Candidates in draw 2 must NOT include prev_winner
        let remaining_candidates: Vec<String> = conn
            .prepare("SELECT entry_value FROM lucky_draw_entries WHERE draw_id = ?1;")
            .unwrap()
            .query_map(params![draw2.id], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(!remaining_candidates.contains(&prev_winner));
        assert!(remaining_candidates.contains(&"Charlie".to_string()));
    }
}
