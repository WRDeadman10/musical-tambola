use crate::domain::claims::claim_validation::{normalize_ticket_code, ClaimRejectionCode};
use crate::domain::game::GameSummary;
use crate::domain::history::{ClaimHistoryRow, DashboardStats, HistoryFilter, Page, WinnerRow};
use crate::domain::schemes::SchemeType;
use rusqlite::types::ToSql;
use rusqlite::Connection;
use std::str::FromStr;

pub struct HistoryRepository;

impl HistoryRepository {
    pub fn list_claim_history(
        conn: &Connection,
        filter: &HistoryFilter,
    ) -> Result<Page<ClaimHistoryRow>, String> {
        let effective_limit: u32 = filter.limit.clamp(1, 200);
        let normalized_ticket: Option<String> = filter
            .ticket_code
            .as_ref()
            .map(|c| normalize_ticket_code(c.trim()));

        let mut where_clauses: Vec<String> = Vec::new();
        let mut count_params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(gid) = filter.game_id {
            where_clauses.push(format!("c.game_id = ?{}", count_params.len() + 1));
            count_params.push(Box::new(gid));
        }

        if let Some(ref st) = filter.scheme_type {
            where_clauses.push(format!(
                "COALESCE(gs.scheme_type, ps.scheme_type) = ?{}",
                count_params.len() + 1
            ));
            count_params.push(Box::new(st.as_str().to_string()));
        }

        if let Some(ref tc) = normalized_ticket {
            where_clauses.push(format!(
                "(c.ticket_code_submitted = ?{0} OR t.ticket_code = ?{0})",
                count_params.len() + 1
            ));
            count_params.push(Box::new(tc.clone()));
        }

        if let Some(ref st) = filter.status {
            where_clauses.push(format!("c.status = ?{}", count_params.len() + 1));
            count_params.push(Box::new(st.to_uppercase()));
        }

        if let Some(ref df) = filter.date_from {
            where_clauses.push(format!(
                "date(c.claimed_at) >= date(?{})",
                count_params.len() + 1
            ));
            count_params.push(Box::new(df.clone()));
        }

        if let Some(ref dt) = filter.date_to {
            where_clauses.push(format!(
                "date(c.claimed_at) <= date(?{})",
                count_params.len() + 1
            ));
            count_params.push(Box::new(dt.clone()));
        }

        let where_sql: String = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 1. Total count query
        let count_sql: String = format!(
            "SELECT COUNT(*)
             FROM claims c
             JOIN games g ON c.game_id = g.id
             LEFT JOIN tickets t ON c.ticket_id = t.id
             LEFT JOIN game_schemes gs ON c.game_id = gs.game_id AND c.scheme_id = gs.scheme_id
             LEFT JOIN prize_schemes ps ON c.scheme_id = ps.id
             {};",
            where_sql
        );

        let mut count_stmt = conn
            .prepare(&count_sql)
            .map_err(|e| format!("Failed to prepare count query: {}", e))?;

        let count_slice: Vec<&dyn ToSql> = count_params.iter().map(|b| b.as_ref()).collect();
        let total: u32 = count_stmt
            .query_row(count_slice.as_slice(), |r| r.get(0))
            .map_err(|e| format!("Failed to execute count query: {}", e))?;

        // 2. Query page items
        let query_sql: String = format!(
            "SELECT c.id, c.game_id, g.game_code,
                    COALESCE(t.ticket_code, c.ticket_code_submitted),
                    COALESCE(gs.name, ps.name),
                    COALESCE(gs.scheme_type, ps.scheme_type),
                    c.status, c.rejection_code, c.rejection_reason,
                    c.current_draw_position, c.qualification_position,
                    w.winner_position, c.claimed_at
             FROM claims c
             JOIN games g ON c.game_id = g.id
             LEFT JOIN tickets t ON c.ticket_id = t.id
             LEFT JOIN game_schemes gs ON c.game_id = gs.game_id AND c.scheme_id = gs.scheme_id
             LEFT JOIN prize_schemes ps ON c.scheme_id = ps.id
             LEFT JOIN winners w ON c.id = w.claim_id
             {}
             ORDER BY c.id DESC
             LIMIT ?{} OFFSET ?{};",
            where_sql,
            count_params.len() + 1,
            count_params.len() + 2
        );

        let mut query_params: Vec<Box<dyn ToSql>> = count_params;
        query_params.push(Box::new(effective_limit));
        query_params.push(Box::new(filter.offset));

        let mut stmt = conn
            .prepare(&query_sql)
            .map_err(|e| format!("Failed to prepare list_claim_history query: {}", e))?;

        let query_slice: Vec<&dyn ToSql> = query_params.iter().map(|b| b.as_ref()).collect();
        let rows = stmt
            .query_map(query_slice.as_slice(), |r| {
                let claim_id: i64 = r.get(0)?;
                let game_id: i64 = r.get(1)?;
                let game_code: String = r.get(2)?;
                let ticket_code: String = r.get(3)?;
                let scheme_name: Option<String> = r.get(4)?;
                let scheme_type_str: Option<String> = r.get(5)?;
                let status: String = r.get(6)?;
                let rejection_code_str: Option<String> = r.get(7)?;
                let rejection_reason: Option<String> = r.get(8)?;
                let current_draw_position: u32 = r.get(9)?;
                let qualification_position: Option<u32> = r.get(10)?;
                let winner_position: Option<u32> = r.get(11)?;
                let submitted_at: String = r.get(12)?;

                let scheme_type: Option<SchemeType> = scheme_type_str
                    .as_deref()
                    .and_then(|s| SchemeType::from_str(s).ok());
                let rejection_code: Option<ClaimRejectionCode> = rejection_code_str
                    .as_deref()
                    .and_then(|s| ClaimRejectionCode::from_str(s).ok());

                Ok(ClaimHistoryRow {
                    claim_id,
                    game_id,
                    game_code,
                    ticket_code,
                    scheme_name,
                    scheme_type,
                    status,
                    rejection_code,
                    rejection_reason,
                    current_draw_position,
                    qualification_position,
                    winner_position,
                    submitted_at,
                })
            })
            .map_err(|e| format!("Failed to execute list_claim_history query: {}", e))?;

        let mut items: Vec<ClaimHistoryRow> = Vec::new();
        for r in rows {
            items.push(r.map_err(|e| format!("Error mapping claim row: {}", e))?);
        }

        Ok(Page { items, total })
    }

    pub fn list_winners(
        conn: &Connection,
        filter: &HistoryFilter,
    ) -> Result<Page<WinnerRow>, String> {
        let effective_limit: u32 = filter.limit.clamp(1, 200);
        let normalized_ticket: Option<String> = filter
            .ticket_code
            .as_ref()
            .map(|c| normalize_ticket_code(c.trim()));

        let mut where_clauses: Vec<String> = Vec::new();
        let mut count_params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(gid) = filter.game_id {
            where_clauses.push(format!("w.game_id = ?{}", count_params.len() + 1));
            count_params.push(Box::new(gid));
        }

        if let Some(ref st) = filter.scheme_type {
            where_clauses.push(format!(
                "COALESCE(gs.scheme_type, ps.scheme_type) = ?{}",
                count_params.len() + 1
            ));
            count_params.push(Box::new(st.as_str().to_string()));
        }

        if let Some(ref tc) = normalized_ticket {
            where_clauses.push(format!(
                "(t.ticket_code = ?{0} OR c.ticket_code_submitted = ?{0})",
                count_params.len() + 1
            ));
            count_params.push(Box::new(tc.clone()));
        }

        if let Some(ref df) = filter.date_from {
            where_clauses.push(format!(
                "date(w.awarded_at) >= date(?{})",
                count_params.len() + 1
            ));
            count_params.push(Box::new(df.clone()));
        }

        if let Some(ref dt) = filter.date_to {
            where_clauses.push(format!(
                "date(w.awarded_at) <= date(?{})",
                count_params.len() + 1
            ));
            count_params.push(Box::new(dt.clone()));
        }

        let where_sql: String = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 1. Total count query
        let count_sql: String = format!(
            "SELECT COUNT(*)
             FROM winners w
             JOIN games g ON w.game_id = g.id
             JOIN tickets t ON w.ticket_id = t.id
             JOIN claims c ON w.claim_id = c.id
             LEFT JOIN game_schemes gs ON w.game_id = gs.game_id AND w.scheme_id = gs.scheme_id
             LEFT JOIN prize_schemes ps ON w.scheme_id = ps.id
             {};",
            where_sql
        );

        let mut count_stmt = conn
            .prepare(&count_sql)
            .map_err(|e| format!("Failed to prepare winners count query: {}", e))?;

        let count_slice: Vec<&dyn ToSql> = count_params.iter().map(|b| b.as_ref()).collect();
        let total: u32 = count_stmt
            .query_row(count_slice.as_slice(), |r| r.get(0))
            .map_err(|e| format!("Failed to execute winners count query: {}", e))?;

        // 2. Query page items ordered by game DESC, scheme_id ASC, winner_position ASC
        let query_sql: String = format!(
            "SELECT w.id, w.claim_id, w.game_id, g.game_code, t.ticket_code,
                    COALESCE(gs.name, ps.name, 'Unknown Scheme'),
                    COALESCE(gs.scheme_type, ps.scheme_type, 'EARLY_FIVE'),
                    w.winner_position,
                    COALESCE(gs.max_winners, ps.max_winners, 1),
                    w.is_full_house,
                    c.qualification_position,
                    w.awarded_at
             FROM winners w
             JOIN games g ON w.game_id = g.id
             JOIN tickets t ON w.ticket_id = t.id
             JOIN claims c ON w.claim_id = c.id
             LEFT JOIN game_schemes gs ON w.game_id = gs.game_id AND w.scheme_id = gs.scheme_id
             LEFT JOIN prize_schemes ps ON w.scheme_id = ps.id
             {}
             ORDER BY w.game_id DESC, w.scheme_id ASC, w.winner_position ASC
             LIMIT ?{} OFFSET ?{};",
            where_sql,
            count_params.len() + 1,
            count_params.len() + 2
        );

        let mut query_params: Vec<Box<dyn ToSql>> = count_params;
        query_params.push(Box::new(effective_limit));
        query_params.push(Box::new(filter.offset));

        let mut stmt = conn
            .prepare(&query_sql)
            .map_err(|e| format!("Failed to prepare list_winners query: {}", e))?;

        let query_slice: Vec<&dyn ToSql> = query_params.iter().map(|b| b.as_ref()).collect();
        let rows = stmt
            .query_map(query_slice.as_slice(), |r| {
                let winner_id: i64 = r.get(0)?;
                let claim_id: i64 = r.get(1)?;
                let game_id: i64 = r.get(2)?;
                let game_code: String = r.get(3)?;
                let ticket_code: String = r.get(4)?;
                let scheme_name: String = r.get(5)?;
                let scheme_type_str: String = r.get(6)?;
                let winner_position: u32 = r.get(7)?;
                let max_winners: u32 = r.get(8)?;
                let is_full_house: bool = r.get(9)?;
                let qualification_position: u32 = r.get::<_, Option<u32>>(10)?.unwrap_or(0);
                let awarded_at: String = r.get(11)?;

                let scheme_type: SchemeType =
                    SchemeType::from_str(&scheme_type_str).unwrap_or(SchemeType::EarlyFive);

                Ok(WinnerRow {
                    winner_id,
                    claim_id,
                    game_id,
                    game_code,
                    ticket_code,
                    scheme_name,
                    scheme_type,
                    winner_position,
                    max_winners,
                    is_full_house,
                    qualification_position,
                    awarded_at,
                })
            })
            .map_err(|e| format!("Failed to execute list_winners query: {}", e))?;

        let mut items: Vec<WinnerRow> = Vec::new();
        for r in rows {
            items.push(r.map_err(|e| format!("Error mapping winner row: {}", e))?);
        }

        Ok(Page { items, total })
    }

    pub fn get_dashboard_stats(conn: &Connection) -> Result<DashboardStats, String> {
        let media_total: u32 = conn
            .query_row("SELECT COUNT(*) FROM media_assets;", [], |r| r.get(0))
            .unwrap_or(0);

        let media_valid: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM media_assets WHERE is_valid = 1;",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let media_invalid: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM media_assets WHERE is_valid = 0 AND validation_error IS NOT NULL;",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let ticket_batches: u32 = conn
            .query_row("SELECT COUNT(*) FROM ticket_batches;", [], |r| r.get(0))
            .unwrap_or(0);

        let total_tickets: u32 = conn
            .query_row("SELECT COUNT(*) FROM tickets;", [], |r| r.get(0))
            .unwrap_or(0);

        let games_played: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM games WHERE status IN ('COMPLETED', 'ENDED');",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let active_game: Option<GameSummary> =
            crate::db::repositories::game::GameRepository::get_active_game(conn)?
                .map(|state| state.game);

        let recent_games: Vec<GameSummary> =
            crate::db::repositories::game::GameRepository::list_games(conn, 5)?;

        let recent_winners_page: Page<WinnerRow> = Self::list_winners(
            conn,
            &HistoryFilter {
                game_id: None,
                scheme_type: None,
                ticket_code: None,
                status: None,
                date_from: None,
                date_to: None,
                offset: 0,
                limit: 5,
            },
        )?;

        Ok(DashboardStats {
            media_total,
            media_valid,
            media_invalid,
            ticket_batches,
            total_tickets,
            games_played,
            active_game,
            recent_games,
            recent_winners: recent_winners_page.items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::claim::ClaimRepository;
    use crate::db::repositories::game::GameRepository;
    use crate::db::repositories::media::MediaRepository;
    use crate::db::repositories::ticket::TicketRepository;
    use crate::domain::media::{MediaType, SaveMediaInput, SourceType};
    use crate::domain::random::DeterministicRandomSource;
    use crate::domain::tambola::ticket_generator::Ticket;
    use rusqlite::params;

    fn setup_test_db() -> Connection {
        let conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn populate_valid_media_and_batch(conn: &mut Connection, ticket_count: u32) -> i64 {
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

        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(777);
        let batch = TicketRepository::generate_batch(
            conn,
            "History Test Batch",
            ticket_count,
            &mut rng,
            |_, _| {},
        )
        .unwrap();
        batch.id
    }

    fn dto_to_ticket(cells_vec: &[crate::domain::tambola::TicketCell]) -> Ticket {
        let mut cells: [[Option<u8>; 9]; 3] = [[None; 9]; 3];
        for c in cells_vec {
            let r: usize = c.row as usize;
            let col: usize = c.col as usize;
            if r < 3 && col < 9 {
                cells[r][col] = Some(c.number);
            }
        }
        Ticket { cells }
    }

    fn build_valid_permutation(first_num: u8, priority_nums: &[u8]) -> Vec<u8> {
        let mut seq: Vec<u8> = Vec::with_capacity(90);
        let mut used: [bool; 91] = [false; 91];

        seq.push(first_num);
        used[first_num as usize] = true;

        for &n in priority_nums {
            if (1..=90).contains(&n) && !used[n as usize] {
                seq.push(n);
                used[n as usize] = true;
            }
        }

        for n in 1..=90 {
            if !used[n as usize] {
                seq.push(n);
                used[n as usize] = true;
            }
        }
        seq
    }

    #[test]
    fn test_history_queries_and_filters() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 2);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(42);

        let t1_dto = TicketRepository::get_ticket(&conn, "T-000001").unwrap();
        let t1: Ticket = dto_to_ticket(&t1_dto.cells);
        let t1_top: Vec<u8> = t1.cells[0].iter().filter_map(|&c| c).collect();

        let early_five_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let g = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g_id: i64 = g.game.id;
        let g_first: u8 = g.current.unwrap().number;

        let g_seq: Vec<u8> = build_valid_permutation(g_first, &t1_top);
        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g_seq).unwrap(), g_id],
        )
        .unwrap();

        // Draw until top row is complete (p5)
        let mut t1_drawn: u32 = 0;
        let mut p5: u32 = 0;
        for (idx, &num) in g_seq.iter().enumerate() {
            if t1_top.contains(&num) {
                t1_drawn += 1;
                if t1_drawn == 5 {
                    p5 = (idx + 1) as u32;
                    break;
                }
            }
        }
        for p in 1..p5 {
            GameRepository::draw_next(&mut conn, g_id, p).unwrap();
        }

        // Submit 1 accepted claim for Early Five
        let claim_acc =
            ClaimRepository::submit_claim(&mut conn, g_id, early_five_id, "T-000001").unwrap();
        assert_eq!(claim_acc.status, "ACCEPTED");

        // Submit 1 rejected claim for duplicate Early Five
        let claim_rej =
            ClaimRepository::submit_claim(&mut conn, g_id, early_five_id, "T-000001").unwrap();
        assert_eq!(claim_rej.status, "REJECTED");

        // 1. list_claim_history with no filter: total == 2
        let all_claims = HistoryRepository::list_claim_history(
            &conn,
            &HistoryFilter {
                game_id: None,
                scheme_type: None,
                ticket_code: None,
                status: None,
                date_from: None,
                date_to: None,
                offset: 0,
                limit: 50,
            },
        )
        .unwrap();
        assert_eq!(all_claims.total, 2);
        assert_eq!(all_claims.items.len(), 2);
        assert_eq!(all_claims.items[0].status, "REJECTED");
        assert_eq!(all_claims.items[1].status, "ACCEPTED");
        assert_eq!(all_claims.items[1].winner_position, Some(1));

        // 2. Filter by status == ACCEPTED
        let acc_claims = HistoryRepository::list_claim_history(
            &conn,
            &HistoryFilter {
                game_id: None,
                scheme_type: None,
                ticket_code: None,
                status: Some("ACCEPTED".to_string()),
                date_from: None,
                date_to: None,
                offset: 0,
                limit: 50,
            },
        )
        .unwrap();
        assert_eq!(acc_claims.total, 1);
        assert_eq!(acc_claims.items[0].claim_id, claim_acc.claim_id);

        // 3. Filter by ticket code normalized ("1" -> "T-000001")
        let ticket_claims = HistoryRepository::list_claim_history(
            &conn,
            &HistoryFilter {
                game_id: None,
                scheme_type: None,
                ticket_code: Some("1".to_string()),
                status: None,
                date_from: None,
                date_to: None,
                offset: 0,
                limit: 50,
            },
        )
        .unwrap();
        assert_eq!(ticket_claims.total, 2);

        // 4. Filter by date boundaries
        let today: String = conn
            .query_row("SELECT date('now');", [], |r| r.get(0))
            .unwrap();
        let date_claims = HistoryRepository::list_claim_history(
            &conn,
            &HistoryFilter {
                game_id: None,
                scheme_type: None,
                ticket_code: None,
                status: None,
                date_from: Some(today.clone()),
                date_to: Some(today.clone()),
                offset: 0,
                limit: 50,
            },
        )
        .unwrap();
        assert_eq!(date_claims.total, 2);

        // 5. list_winners query
        let winners = HistoryRepository::list_winners(
            &conn,
            &HistoryFilter {
                game_id: None,
                scheme_type: None,
                ticket_code: None,
                status: None,
                date_from: None,
                date_to: None,
                offset: 0,
                limit: 50,
            },
        )
        .unwrap();
        assert_eq!(winners.total, 1);
        assert_eq!(winners.items.len(), 1);
        assert_eq!(winners.items[0].ticket_code, "T-000001");
        assert_eq!(winners.items[0].winner_position, 1);
        assert_eq!(winners.items[0].scheme_name, "Early Five");
        assert_eq!(winners.items[0].scheme_type, SchemeType::EarlyFive);
    }

    #[test]
    fn test_snapshot_scheme_name_survives_live_rename() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 1);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(55);

        let t1_dto = TicketRepository::get_ticket(&conn, "T-000001").unwrap();
        let t1: Ticket = dto_to_ticket(&t1_dto.cells);
        let t1_top: Vec<u8> = t1.cells[0].iter().filter_map(|&c| c).collect();

        let early_five_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let g = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g_id: i64 = g.game.id;
        let g_first: u8 = g.current.unwrap().number;

        let g_seq: Vec<u8> = build_valid_permutation(g_first, &t1_top);
        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g_seq).unwrap(), g_id],
        )
        .unwrap();

        let mut t1_drawn: u32 = 0;
        let mut p5: u32 = 0;
        for (idx, &num) in g_seq.iter().enumerate() {
            if t1_top.contains(&num) {
                t1_drawn += 1;
                if t1_drawn == 5 {
                    p5 = (idx + 1) as u32;
                    break;
                }
            }
        }
        for p in 1..p5 {
            GameRepository::draw_next(&mut conn, g_id, p).unwrap();
        }

        let claim =
            ClaimRepository::submit_claim(&mut conn, g_id, early_five_id, "T-000001").unwrap();
        assert_eq!(claim.status, "ACCEPTED");

        // Now rename the live prize scheme in prize_schemes
        conn.execute(
            "UPDATE prize_schemes SET name = 'Renamed Modern Five' WHERE id = ?1;",
            params![early_five_id],
        )
        .unwrap();

        // Query winners & claim history: scheme_name must still be snapshot name "Early Five"
        let winners = HistoryRepository::list_winners(
            &conn,
            &HistoryFilter {
                game_id: Some(g_id),
                scheme_type: None,
                ticket_code: None,
                status: None,
                date_from: None,
                date_to: None,
                offset: 0,
                limit: 10,
            },
        )
        .unwrap();
        assert_eq!(winners.items[0].scheme_name, "Early Five");

        let claims = HistoryRepository::list_claim_history(
            &conn,
            &HistoryFilter {
                game_id: Some(g_id),
                scheme_type: None,
                ticket_code: None,
                status: None,
                date_from: None,
                date_to: None,
                offset: 0,
                limit: 10,
            },
        )
        .unwrap();
        assert_eq!(claims.items[0].scheme_name.as_deref(), Some("Early Five"));
    }

    #[test]
    fn test_dashboard_stats_and_counts() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 3);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(88);

        let g = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g_id: i64 = g.game.id;

        // End Game 1 so games_played increments
        GameRepository::end_game(&mut conn, g_id).unwrap();

        let stats = HistoryRepository::get_dashboard_stats(&conn).unwrap();
        assert_eq!(stats.media_total, 90);
        assert_eq!(stats.media_valid, 90);
        assert_eq!(stats.media_invalid, 0);
        assert_eq!(stats.ticket_batches, 1);
        assert_eq!(stats.total_tickets, 3);
        assert_eq!(stats.games_played, 1);
        assert!(stats.active_game.is_none());
        assert_eq!(stats.recent_games.len(), 1);
    }

    #[test]
    fn test_never_delete_history_guarantee() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 2);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(99);

        let g = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g_id: i64 = g.game.id;

        let early_five_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let _ = ClaimRepository::submit_claim(&mut conn, g_id, early_five_id, "T-000001");

        // Attempting to delete the ticket batch that was used in the game MUST fail!
        let del_batch_res = TicketRepository::delete_batch(&conn, batch_id);
        assert!(del_batch_res.is_err());
        assert!(del_batch_res.unwrap_err().contains("assigned to games"));

        // Ensure games, claims, and winners remain intact
        let game_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM games WHERE id = ?1;",
                params![g_id],
                |r| r.get(0),
            )
            .unwrap();
        let claim_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM claims WHERE game_id = ?1;",
                params![g_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(game_count, 1);
        assert_eq!(claim_count, 1);
    }
}
