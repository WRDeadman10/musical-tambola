use crate::domain::random::RandomSource;
use crate::domain::tambola::ticket_generator::{generate_ticket, layout_hash, Ticket};
use crate::domain::tambola::{TicketBatch, TicketCell, TicketDto, TicketPage};
use rusqlite::{params, Connection, OptionalExtension, Transaction};

pub struct TicketRepository;

impl TicketRepository {
    pub fn generate_batch<R: RandomSource, F: FnMut(usize, usize)>(
        conn: &mut Connection,
        name: &str,
        count: u32,
        rng: &mut R,
        mut on_progress: F,
    ) -> Result<TicketBatch, String> {
        if !(1..=10_000).contains(&count) {
            return Err(format!(
                "Batch ticket count must be between 1 and 10,000, got {}",
                count
            ));
        }

        let trimmed_name: &str = name.trim();
        if trimmed_name.is_empty() {
            return Err("Batch name cannot be empty".to_string());
        }

        let tx: Transaction<'_> = conn
            .transaction()
            .map_err(|e: rusqlite::Error| format!("Failed to begin transaction: {}", e))?;

        // 1. Generate unique batch code (e.g. B-0001)
        let next_batch_num: u64 = tx
            .query_row(
                "SELECT COALESCE(MAX(id), 0) + 1 FROM ticket_batches;",
                [],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .map_err(|e: rusqlite::Error| format!("Failed to query next batch id: {}", e))?;
        let batch_code: String = format!("B-{:04}", next_batch_num);

        tx.execute(
            "INSERT INTO ticket_batches (batch_code, name, ticket_count) VALUES (?1, ?2, ?3);",
            params![batch_code, trimmed_name, count],
        )
        .map_err(|e: rusqlite::Error| format!("Failed to insert ticket batch: {}", e))?;

        let batch_id: i64 = tx.last_insert_rowid();

        // 2. Monotonic sequence for ticket codes
        let val_opt: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ticket_sequence_counter';",
                [],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .optional()
            .map_err(|e: rusqlite::Error| {
                format!("Failed to read ticket sequence setting: {}", e)
            })?;

        let mut seq: u64 = match val_opt {
            Some(s) => s
                .trim()
                .parse::<u64>()
                .map_err(|e: std::num::ParseIntError| {
                    format!("Corrupted ticket sequence counter '{}': {}", s, e)
                })?,
            None => 0,
        };

        let mut overall_budget: usize = (count as usize) * 10;

        for i in 0..count {
            let mut per_ticket_retries: usize = 50;
            let mut generated_ticket: Option<(Ticket, String)> = None;

            while per_ticket_retries > 0 && overall_budget > 0 {
                per_ticket_retries -= 1;
                overall_budget -= 1;

                let ticket: Ticket = generate_ticket(rng)
                    .map_err(|e| format!("Ticket generation validation failed: {}", e))?;
                let hash: String = layout_hash(&ticket);

                // Check collision against DB
                let exists: bool = tx
                    .query_row(
                        "SELECT 1 FROM tickets WHERE layout_hash = ?1;",
                        params![hash],
                        |_: &rusqlite::Row<'_>| Ok(()),
                    )
                    .optional()
                    .map_err(|e: rusqlite::Error| format!("Layout collision check failed: {}", e))?
                    .is_some();

                if !exists {
                    generated_ticket = Some((ticket, hash));
                    break;
                }
            }

            let (ticket, hash): (Ticket, String) = match generated_ticket {
                Some(t) => t,
                None => {
                    return Err(format!(
                        "Retry budget exhausted: unable to generate unique layout for ticket {} of {}",
                        i + 1,
                        count
                    ));
                }
            };

            seq += 1;
            let ticket_code: String = format!("T-{:06}", seq);

            tx.execute(
                "INSERT INTO tickets (batch_id, ticket_code, layout_hash) VALUES (?1, ?2, ?3);",
                params![batch_id, ticket_code, hash],
            )
            .map_err(|e: rusqlite::Error| format!("Failed to insert ticket: {}", e))?;

            let ticket_id: i64 = tx.last_insert_rowid();

            // Insert 15 cells
            for (r, row) in ticket.cells.iter().enumerate() {
                for (c, cell_opt) in row.iter().enumerate() {
                    if let Some(n) = *cell_opt {
                        tx.execute(
                            "INSERT INTO ticket_cells (ticket_id, row_idx, col_idx, number) VALUES (?1, ?2, ?3, ?4);",
                            params![ticket_id, r as i64, c as i64, n as i64],
                        )
                        .map_err(|e: rusqlite::Error| format!("Failed to insert ticket cell: {}", e))?;
                    }
                }
            }

            on_progress(i as usize + 1, count as usize);
        }

        // Update sequence counter
        tx.execute(
            "INSERT INTO app_settings (key, value) VALUES ('ticket_sequence_counter', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;",
            params![seq.to_string()],
        )
        .map_err(|e: rusqlite::Error| format!("Failed to persist ticket sequence: {}", e))?;

        let created_at: String = tx
            .query_row(
                "SELECT created_at FROM ticket_batches WHERE id = ?1;",
                params![batch_id],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .map_err(|e: rusqlite::Error| format!("Failed to fetch created_at: {}", e))?;

        tx.commit()
            .map_err(|e: rusqlite::Error| format!("Failed to commit batch transaction: {}", e))?;

        Ok(TicketBatch {
            id: batch_id,
            name: trimmed_name.to_string(),
            requested_count: count,
            generated_count: count,
            created_at,
        })
    }

    pub fn list_batches(conn: &Connection) -> Result<Vec<TicketBatch>, String> {
        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT id, name, ticket_count, created_at FROM ticket_batches ORDER BY id DESC;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare list_batches: {}", e))?;

        let rows = stmt
            .query_map([], |r: &rusqlite::Row<'_>| {
                let id: i64 = r.get(0)?;
                let name: String = r.get(1)?;
                let ticket_count: u32 = r.get(2)?;
                let created_at: String = r.get(3)?;

                Ok(TicketBatch {
                    id,
                    name,
                    requested_count: ticket_count,
                    generated_count: ticket_count,
                    created_at,
                })
            })
            .map_err(|e: rusqlite::Error| format!("Query failed: {}", e))?;

        let mut batches: Vec<TicketBatch> = Vec::new();
        for batch in rows.flatten() {
            batches.push(batch);
        }

        Ok(batches)
    }

    pub fn list_batch_tickets(
        conn: &Connection,
        batch_id: i64,
        offset: u32,
        limit: u32,
    ) -> Result<TicketPage, String> {
        let total: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tickets WHERE batch_id = ?1;",
                params![batch_id],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .map_err(|e: rusqlite::Error| format!("Failed to get ticket count: {}", e))?;

        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT id, ticket_code, batch_id, created_at FROM tickets WHERE batch_id = ?1 ORDER BY id ASC LIMIT ?2 OFFSET ?3;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare ticket query: {}", e))?;

        let rows = stmt
            .query_map(params![batch_id, limit, offset], |r: &rusqlite::Row<'_>| {
                let id: i64 = r.get(0)?;
                let code: String = r.get(1)?;
                let b_id: i64 = r.get(2)?;
                let created_at: String = r.get(3)?;
                Ok((id, code, b_id, created_at))
            })
            .map_err(|e: rusqlite::Error| format!("Query failed: {}", e))?;

        let mut items: Vec<TicketDto> = Vec::new();
        for row in rows.flatten() {
            let (id, code, b_id, created_at) = row;
            let cells: Vec<TicketCell> = Self::fetch_ticket_cells(conn, id)?;
            items.push(TicketDto {
                id,
                code,
                batch_id: b_id,
                cells,
                created_at,
            });
        }

        Ok(TicketPage { items, total })
    }

    pub fn get_ticket(conn: &Connection, code: &str) -> Result<TicketDto, String> {
        let trimmed: &str = code.trim();
        let (id, b_id, created_at): (i64, i64, String) = conn
            .query_row(
                "SELECT id, batch_id, created_at FROM tickets WHERE ticket_code = ?1;",
                params![trimmed],
                |r: &rusqlite::Row<'_>| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(|e: rusqlite::Error| format!("Failed to query ticket: {}", e))?
            .ok_or_else(|| format!("Ticket '{}' not found", trimmed))?;

        let cells: Vec<TicketCell> = Self::fetch_ticket_cells(conn, id)?;

        Ok(TicketDto {
            id,
            code: trimmed.to_string(),
            batch_id: b_id,
            cells,
            created_at,
        })
    }

    pub fn search_tickets(
        conn: &Connection,
        query: &str,
        limit: u32,
    ) -> Result<Vec<TicketDto>, String> {
        let pattern: String = format!("{}%", query.trim());
        let effective_limit: u32 = if limit == 0 { 24 } else { limit.min(100) };

        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT id, ticket_code, batch_id, created_at FROM tickets WHERE ticket_code LIKE ?1 ORDER BY id ASC LIMIT ?2;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare search: {}", e))?;

        let rows = stmt
            .query_map(
                params![pattern, effective_limit],
                |r: &rusqlite::Row<'_>| {
                    let id: i64 = r.get(0)?;
                    let code: String = r.get(1)?;
                    let b_id: i64 = r.get(2)?;
                    let created_at: String = r.get(3)?;
                    Ok((id, code, b_id, created_at))
                },
            )
            .map_err(|e: rusqlite::Error| format!("Search failed: {}", e))?;

        let mut results: Vec<TicketDto> = Vec::new();
        for row in rows.flatten() {
            let (id, code, b_id, created_at) = row;
            let cells: Vec<TicketCell> = Self::fetch_ticket_cells(conn, id)?;
            results.push(TicketDto {
                id,
                code,
                batch_id: b_id,
                cells,
                created_at,
            });
        }

        Ok(results)
    }

    pub fn delete_batch(conn: &Connection, batch_id: i64) -> Result<(), String> {
        // 1. Guard against deletion if tickets were used in game_tickets, claims, or winners
        let in_game: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM game_tickets WHERE ticket_id IN (SELECT id FROM tickets WHERE batch_id = ?1);",
                params![batch_id],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .unwrap_or(0);

        if in_game > 0 {
            return Err(
                "Cannot delete batch: tickets from this batch have been assigned to games"
                    .to_string(),
            );
        }

        let in_claims: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM claims WHERE ticket_id IN (SELECT id FROM tickets WHERE batch_id = ?1);",
                params![batch_id],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .unwrap_or(0);

        if in_claims > 0 {
            return Err(
                "Cannot delete batch: tickets from this batch have claims recorded".to_string(),
            );
        }

        let in_winners: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM winners WHERE ticket_id IN (SELECT id FROM tickets WHERE batch_id = ?1);",
                params![batch_id],
                |r: &rusqlite::Row<'_>| r.get(0),
            )
            .unwrap_or(0);

        if in_winners > 0 {
            return Err(
                "Cannot delete batch: tickets from this batch are recorded as winners".to_string(),
            );
        }

        conn.execute(
            "DELETE FROM ticket_batches WHERE id = ?1;",
            params![batch_id],
        )
        .map_err(|e: rusqlite::Error| format!("Failed to delete batch: {}", e))?;

        Ok(())
    }

    fn fetch_ticket_cells(conn: &Connection, ticket_id: i64) -> Result<Vec<TicketCell>, String> {
        let mut stmt: rusqlite::Statement<'_> = conn
            .prepare(
                "SELECT row_idx, col_idx, number FROM ticket_cells WHERE ticket_id = ?1 ORDER BY row_idx ASC, col_idx ASC;",
            )
            .map_err(|e: rusqlite::Error| format!("Failed to prepare cell query: {}", e))?;

        let rows = stmt
            .query_map(params![ticket_id], |r: &rusqlite::Row<'_>| {
                let row: u8 = r.get(0)?;
                let col: u8 = r.get(1)?;
                let number: u8 = r.get(2)?;
                Ok(TicketCell { row, col, number })
            })
            .map_err(|e: rusqlite::Error| format!("Cell query failed: {}", e))?;

        let mut cells: Vec<TicketCell> = Vec::with_capacity(15);
        for cell in rows.flatten() {
            cells.push(cell);
        }

        Ok(cells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::domain::random::{ConstantRandomSource, DeterministicRandomSource, OsRngSource};

    #[test]
    fn test_generate_batch_and_monotonic_codes() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let mut rng: OsRngSource = OsRngSource::new();
        let batch1: TicketBatch =
            TicketRepository::generate_batch(&mut conn, "Batch Alpha", 10, &mut rng, |_, _| {})
                .unwrap();
        assert_eq!(batch1.requested_count, 10);
        assert_eq!(batch1.generated_count, 10);

        let page1: TicketPage =
            TicketRepository::list_batch_tickets(&conn, batch1.id, 0, 10).unwrap();
        assert_eq!(page1.total, 10);
        assert_eq!(page1.items[0].code, "T-000001");
        assert_eq!(page1.items[9].code, "T-000010");

        // Next batch continues sequence
        let batch2: TicketBatch =
            TicketRepository::generate_batch(&mut conn, "Batch Beta", 5, &mut rng, |_, _| {})
                .unwrap();
        let page2: TicketPage =
            TicketRepository::list_batch_tickets(&conn, batch2.id, 0, 5).unwrap();
        assert_eq!(page2.items[0].code, "T-000011");
        assert_eq!(page2.items[4].code, "T-000015");
    }

    #[test]
    fn test_retry_budget_exhaustion_rollback() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 1. Generate one ticket normally
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(100);
        TicketRepository::generate_batch(&mut conn, "Initial", 1, &mut rng, |_, _| {}).unwrap();

        // 2. Inject ConstantRandomSource that always generates duplicate layouts
        let mut const_rng: ConstantRandomSource = ConstantRandomSource::new(0);
        let fail_res: Result<TicketBatch, String> = TicketRepository::generate_batch(
            &mut conn,
            "Failing Batch",
            5,
            &mut const_rng,
            |_, _| {},
        );

        assert!(fail_res.is_err());
        assert!(fail_res.unwrap_err().contains("Retry budget exhausted"));

        // Assert rollback: no batch with name "Failing Batch" exists
        let batches: Vec<TicketBatch> = TicketRepository::list_batches(&conn).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].name, "Initial");
    }

    #[test]
    fn test_delete_batch_guard() {
        let mut conn: Connection = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let mut rng: OsRngSource = OsRngSource::new();
        let batch: TicketBatch =
            TicketRepository::generate_batch(&mut conn, "Batch To Delete", 3, &mut rng, |_, _| {})
                .unwrap();

        // Safe delete succeeds
        assert!(TicketRepository::delete_batch(&conn, batch.id).is_ok());
        let batches: Vec<TicketBatch> = TicketRepository::list_batches(&conn).unwrap();
        assert_eq!(batches.len(), 0);
    }
}
