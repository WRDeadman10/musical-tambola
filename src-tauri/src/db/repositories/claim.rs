use crate::domain::claims::{
    evaluate_claim, normalize_ticket_code, ClaimDecision, ClaimRejectionCode, ClaimResult,
    GameSchemeSnapshot, TicketExistingWin,
};
use crate::domain::game::GameStatus;
use crate::domain::schemes::SchemeType;
use crate::domain::tambola::ticket_generator::Ticket;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::str::FromStr;

pub struct ClaimRepository;

fn extract_called_numbers_on_ticket(
    ticket: Option<&Ticket>,
    call_pos_map: &HashMap<u8, u32>,
    current_draw_position: u32,
) -> Vec<u8> {
    let mut called_nums: Vec<u8> = Vec::new();
    if let Some(t) = ticket {
        for row in &t.cells {
            for n in row.iter().flatten() {
                if let Some(&p) = call_pos_map.get(n) {
                    if p <= current_draw_position {
                        called_nums.push(*n);
                    }
                }
            }
        }
    }
    called_nums
}

impl ClaimRepository {
    /// Submits a claim against an active game session inside a single atomic BEGIN IMMEDIATE transaction.
    /// Evaluates rules via the pure domain engine, records the claim in SQLite, allocates winner slot on acceptance,
    /// and handles concurrency / constraint races gracefully.
    pub fn submit_claim(
        conn: &mut Connection,
        game_id: i64,
        scheme_id: i64,
        raw_ticket_code: &str,
    ) -> Result<ClaimResult, String> {
        let normalized_code: String = normalize_ticket_code(raw_ticket_code);

        // Run in an IMMEDIATE transaction so checks, claim insert, and winner allocation are atomic
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // 1. Fetch game status and current_draw_position
        let game_info: Option<(GameStatus, u32)> = tx
            .query_row(
                "SELECT status, current_draw_position FROM games WHERE id = ?1;",
                params![game_id],
                |r| {
                    let status_str: String = r.get(0)?;
                    let pos: u32 = r.get(1)?;
                    let status: GameStatus = GameStatus::from_str(&status_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e,
                        )))
                    })?;
                    Ok((status, pos))
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query game info: {}", e))?;

        let (game_status, current_draw_position): (Option<GameStatus>, u32) = match game_info {
            Some((s, p)) => (Some(s), p),
            None => (None, 0),
        };

        // 2. Fetch ticket if exists by normalized ticket code
        let ticket_db_info: Option<(i64, String, String)> = tx
            .query_row(
                "SELECT id, ticket_code, layout_hash FROM tickets WHERE ticket_code = ?1;",
                params![normalized_code],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(|e| format!("Failed to query ticket: {}", e))?;

        let (ticket_id_opt, ticket_obj, is_ticket_in_game): (Option<i64>, Option<Ticket>, bool) =
            match ticket_db_info {
                Some((tid, _tcode, _lhash)) => {
                    // Check if mapped to game_tickets
                    let in_game: bool = tx
                        .query_row(
                            "SELECT 1 FROM game_tickets WHERE game_id = ?1 AND ticket_id = ?2;",
                            params![game_id, tid],
                            |_| Ok(()),
                        )
                        .optional()
                        .map_err(|e| format!("Failed to check game_tickets: {}", e))?
                        .is_some();

                    // Reconstruct ticket cells from ticket_cells
                    let mut cells: [[Option<u8>; 9]; 3] = [[None; 9]; 3];
                    let mut cells_stmt = tx
                    .prepare(
                        "SELECT row_idx, col_idx, number FROM ticket_cells WHERE ticket_id = ?1;",
                    )
                    .map_err(|e| format!("Failed to prepare cells query: {}", e))?;
                    let cell_rows = cells_stmt
                        .query_map(params![tid], |r| {
                            let row: usize = r.get(0)?;
                            let col: usize = r.get(1)?;
                            let num: u8 = r.get(2)?;
                            Ok((row, col, num))
                        })
                        .map_err(|e| format!("Failed to query cells: {}", e))?;

                    for c in cell_rows {
                        let (row, col, num) =
                            c.map_err(|e| format!("Error mapping cell: {}", e))?;
                        if row < 3 && col < 9 {
                            cells[row][col] = Some(num);
                        }
                    }
                    drop(cells_stmt);

                    let t: Ticket = Ticket { cells };
                    (Some(tid), Some(t), in_game)
                }
                None => (None, None, false),
            };

        // 3. Fetch scheme snapshot from game_schemes
        let scheme_snapshot: Option<GameSchemeSnapshot> = tx
            .query_row(
                "SELECT scheme_id, name, scheme_type, max_winners FROM game_schemes WHERE game_id = ?1 AND scheme_id = ?2;",
                params![game_id, scheme_id],
                |r| {
                    let sid: i64 = r.get(0)?;
                    let name: String = r.get(1)?;
                    let stype_str: String = r.get(2)?;
                    let max_w: u32 = r.get(3)?;
                    let stype: SchemeType = SchemeType::from_str(&stype_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e,
                        )))
                    })?;
                    Ok(GameSchemeSnapshot {
                        scheme_id: sid,
                        scheme_name: name,
                        scheme_type: stype,
                        max_winners: max_w,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query game scheme snapshot: {}", e))?;

        // Fallback display lookup in master prize_schemes if not in game snapshot
        let (display_scheme_name, display_scheme_type, display_max_winners): (
            String,
            SchemeType,
            u32,
        ) = match &scheme_snapshot {
            Some(s) => (s.scheme_name.clone(), s.scheme_type, s.max_winners),
            None => {
                let master: Option<(String, SchemeType, u32)> = tx
                    .query_row(
                        "SELECT name, scheme_type, max_winners FROM prize_schemes WHERE id = ?1;",
                        params![scheme_id],
                        |r| {
                            let name: String = r.get(0)?;
                            let stype_str: String = r.get(1)?;
                            let max_w: u32 = r.get(2)?;
                            let stype: SchemeType =
                                SchemeType::from_str(&stype_str).map_err(|e| {
                                    rusqlite::Error::ToSqlConversionFailure(Box::new(
                                        std::io::Error::new(std::io::ErrorKind::InvalidData, e),
                                    ))
                                })?;
                            Ok((name, stype, max_w))
                        },
                    )
                    .optional()
                    .unwrap_or(None);
                master.unwrap_or_else(|| ("Unknown Scheme".to_string(), SchemeType::EarlyFive, 1))
            }
        };

        // 4. Fetch call_position_by_number
        let mut call_pos_map: HashMap<u8, u32> = HashMap::new();
        let mut calls_stmt = tx
            .prepare("SELECT number, draw_position FROM game_calls WHERE game_id = ?1;")
            .map_err(|e| format!("Failed to prepare calls query: {}", e))?;
        let call_rows = calls_stmt
            .query_map(params![game_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| format!("Failed to query calls: {}", e))?;
        for r in call_rows {
            let (num, pos) = r.map_err(|e| format!("Error mapping call: {}", e))?;
            call_pos_map.insert(num, pos);
        }
        drop(calls_stmt);

        // 5. Fetch ticket's existing wins in this game
        let mut ticket_wins: Vec<TicketExistingWin> = Vec::new();
        if let Some(tid) = ticket_id_opt {
            let mut wins_stmt = tx
                .prepare(
                    "SELECT scheme_id, is_full_house FROM winners WHERE game_id = ?1 AND ticket_id = ?2;",
                )
                .map_err(|e| format!("Failed to prepare ticket wins query: {}", e))?;
            let win_rows = wins_stmt
                .query_map(params![game_id, tid], |r| {
                    Ok(TicketExistingWin {
                        scheme_id: r.get(0)?,
                        is_full_house: r.get(1)?,
                    })
                })
                .map_err(|e| format!("Failed to query ticket wins: {}", e))?;
            for w in win_rows {
                ticket_wins.push(w.map_err(|e| format!("Error mapping ticket win: {}", e))?);
            }
            drop(wins_stmt);
        }

        // 6. Fetch scheme's accepted winner count
        let accepted_count: u32 = tx
            .query_row(
                "SELECT COUNT(*) FROM winners WHERE game_id = ?1 AND scheme_id = ?2;",
                params![game_id, scheme_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // 7. Evaluate claim with pure decision function
        let decision: ClaimDecision = evaluate_claim(
            game_status,
            current_draw_position,
            ticket_obj.as_ref(),
            is_ticket_in_game,
            scheme_snapshot.as_ref(),
            &call_pos_map,
            &ticket_wins,
            accepted_count,
        );

        // 8. Handle Decision & DB persistence
        match decision {
            ClaimDecision::Accept {
                qualification_position,
                winner_position,
            } => {
                let is_fh: bool = display_scheme_type.is_full_house();

                // Insert claim row
                let (claim_id, claimed_at): (i64, String) = tx
                    .query_row(
                        "INSERT INTO claims (
                            game_id, ticket_id, ticket_code_submitted, scheme_id,
                            current_draw_position, qualification_position, status,
                            rejection_code, rejection_reason, claimed_at
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'ACCEPTED', NULL, NULL, CURRENT_TIMESTAMP)
                         RETURNING id, claimed_at;",
                        params![
                            game_id,
                            ticket_id_opt,
                            normalized_code,
                            scheme_id,
                            current_draw_position,
                            qualification_position,
                        ],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .map_err(|e| format!("Failed to insert accepted claim: {}", e))?;

                // Attempt to insert into winners
                let winner_insert_result = tx.execute(
                    "INSERT INTO winners (
                        game_id, scheme_id, ticket_id, claim_id, is_full_house, winner_position, awarded_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP);",
                    params![
                        game_id,
                        scheme_id,
                        ticket_id_opt.unwrap(),
                        claim_id,
                        is_fh,
                        winner_position,
                    ],
                );

                match winner_insert_result {
                    Ok(_) => {
                        tx.commit()
                            .map_err(|e| format!("Failed to commit accepted claim: {}", e))?;

                        let called_nums: Vec<u8> = extract_called_numbers_on_ticket(
                            ticket_obj.as_ref(),
                            &call_pos_map,
                            current_draw_position,
                        );

                        Ok(ClaimResult {
                            claim_id,
                            status: "ACCEPTED".to_string(),
                            rejection_code: None,
                            rejection_reason: None,
                            game_id,
                            ticket_code: normalized_code,
                            scheme_id,
                            scheme_name: display_scheme_name,
                            scheme_type: display_scheme_type,
                            current_draw_position,
                            qualification_position: Some(qualification_position),
                            winner_position: Some(winner_position),
                            max_winners: display_max_winners,
                            ticket: ticket_obj,
                            called_numbers_on_ticket: called_nums,
                            submitted_at: claimed_at,
                        })
                    }
                    Err(rusqlite::Error::SqliteFailure(_err, _)) => {
                        // DB-level race / unique violation fallback: roll back the award,
                        // convert the claim to REJECTED with WINNER_LIMIT_REACHED or appropriate code
                        let rej_code: ClaimRejectionCode = ClaimRejectionCode::WinnerLimitReached;
                        let rej_reason: &'static str = rej_code.canonical_reason();

                        tx.execute(
                            "UPDATE claims SET status = 'REJECTED', rejection_code = ?1, rejection_reason = ?2 WHERE id = ?3;",
                            params![rej_code.as_str(), rej_reason, claim_id],
                        )
                        .map_err(|e| format!("Failed to convert claim on constraint race: {}", e))?;

                        tx.commit().map_err(|e| {
                            format!("Failed to commit converted rejected claim: {}", e)
                        })?;

                        let called_nums: Vec<u8> = extract_called_numbers_on_ticket(
                            ticket_obj.as_ref(),
                            &call_pos_map,
                            current_draw_position,
                        );

                        Ok(ClaimResult {
                            claim_id,
                            status: "REJECTED".to_string(),
                            rejection_code: Some(rej_code),
                            rejection_reason: Some(rej_reason.to_string()),
                            game_id,
                            ticket_code: normalized_code,
                            scheme_id,
                            scheme_name: display_scheme_name,
                            scheme_type: display_scheme_type,
                            current_draw_position,
                            qualification_position: Some(qualification_position),
                            winner_position: None,
                            max_winners: display_max_winners,
                            ticket: ticket_obj,
                            called_numbers_on_ticket: called_nums,
                            submitted_at: claimed_at,
                        })
                    }
                    Err(other_err) => Err(format!(
                        "Unexpected database error inserting winner: {}",
                        other_err
                    )),
                }
            }
            ClaimDecision::Reject {
                code,
                qualification_position,
            } => {
                let code_str: &'static str = code.as_str();
                let reason_str: &'static str = code.canonical_reason();

                let (claim_id, claimed_at): (i64, String) = tx
                    .query_row(
                        "INSERT INTO claims (
                            game_id, ticket_id, ticket_code_submitted, scheme_id,
                            current_draw_position, qualification_position, status,
                            rejection_code, rejection_reason, claimed_at
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'REJECTED', ?7, ?8, CURRENT_TIMESTAMP)
                         RETURNING id, claimed_at;",
                        params![
                            game_id,
                            ticket_id_opt,
                            normalized_code,
                            scheme_id,
                            current_draw_position,
                            qualification_position,
                            code_str,
                            reason_str,
                        ],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .map_err(|e| format!("Failed to insert rejected claim: {}", e))?;

                tx.commit()
                    .map_err(|e| format!("Failed to commit claim transaction: {}", e))?;

                let called_nums: Vec<u8> = extract_called_numbers_on_ticket(
                    ticket_obj.as_ref(),
                    &call_pos_map,
                    current_draw_position,
                );

                Ok(ClaimResult {
                    claim_id,
                    status: "REJECTED".to_string(),
                    rejection_code: Some(code),
                    rejection_reason: Some(reason_str.to_string()),
                    game_id,
                    ticket_code: normalized_code,
                    scheme_id,
                    scheme_name: display_scheme_name,
                    scheme_type: display_scheme_type,
                    current_draw_position,
                    qualification_position,
                    winner_position: None,
                    max_winners: display_max_winners,
                    ticket: ticket_obj,
                    called_numbers_on_ticket: called_nums,
                    submitted_at: claimed_at,
                })
            }
        }
    }

    /// Lists claims for a given game session, newest first. Ticket is null for brevity.
    pub fn list_game_claims(conn: &Connection, game_id: i64) -> Result<Vec<ClaimResult>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.game_id, c.ticket_id, c.ticket_code_submitted, COALESCE(c.scheme_id, 0),
                        c.current_draw_position, c.qualification_position, c.status,
                        c.rejection_code, c.rejection_reason, c.claimed_at,
                        COALESCE(gs.name, ps.name, 'Unknown Scheme'),
                        COALESCE(gs.scheme_type, ps.scheme_type, 'EARLY_FIVE'),
                        COALESCE(gs.max_winners, ps.max_winners, 1),
                        w.winner_position
                 FROM claims c
                 LEFT JOIN game_schemes gs ON c.game_id = gs.game_id AND c.scheme_id = gs.scheme_id
                 LEFT JOIN prize_schemes ps ON c.scheme_id = ps.id
                 LEFT JOIN winners w ON c.id = w.claim_id
                 WHERE c.game_id = ?1
                 ORDER BY c.id DESC;",
            )
            .map_err(|e| format!("Failed to prepare list_game_claims query: {}", e))?;

        let rows = stmt
            .query_map(params![game_id], |r| {
                let claim_id: i64 = r.get(0)?;
                let gid: i64 = r.get(1)?;
                let _ticket_id: Option<i64> = r.get(2)?;
                let ticket_code: String = r.get(3)?;
                let scheme_id: i64 = r.get(4)?;
                let current_draw_position: u32 = r.get(5)?;
                let qualification_position: Option<u32> = r.get(6)?;
                let status: String = r.get(7)?;
                let rejection_code_str: Option<String> = r.get(8)?;
                let rejection_reason: Option<String> = r.get(9)?;
                let submitted_at: String = r.get(10)?;
                let scheme_name: String = r.get(11)?;
                let scheme_type_str: String = r.get(12)?;
                let max_winners: u32 = r.get(13)?;
                let winner_position: Option<u32> = r.get(14)?;

                let scheme_type: SchemeType =
                    SchemeType::from_str(&scheme_type_str).unwrap_or(SchemeType::EarlyFive);
                let rejection_code: Option<ClaimRejectionCode> = rejection_code_str
                    .as_deref()
                    .and_then(|s| ClaimRejectionCode::from_str(s).ok());

                Ok(ClaimResult {
                    claim_id,
                    status,
                    rejection_code,
                    rejection_reason,
                    game_id: gid,
                    ticket_code,
                    scheme_id,
                    scheme_name,
                    scheme_type,
                    current_draw_position,
                    qualification_position,
                    winner_position,
                    max_winners,
                    ticket: None,
                    called_numbers_on_ticket: Vec::new(),
                    submitted_at,
                })
            })
            .map_err(|e| format!("Failed to execute list_game_claims: {}", e))?;

        let mut results: Vec<ClaimResult> = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| format!("Error mapping claim row: {}", e))?);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::game::GameRepository;
    use crate::db::repositories::media::MediaRepository;
    use crate::db::repositories::ticket::TicketRepository;
    use crate::domain::media::{MediaType, SaveMediaInput, SourceType};
    use crate::domain::random::DeterministicRandomSource;

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
            "Claim Test Batch",
            ticket_count,
            &mut rng,
            |_, _| {},
        )
        .unwrap();
        batch.id
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

    #[test]
    fn test_late_claim_scenario() {
        // Spec scenario: ticket completes Four Corners when 63 is drawn at position P;
        // a claim at P is ACCEPTED; in fresh game 2, next -> 55 at P+1, and claim at P+1 is REJECTED as LATE_CLAIM.
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 1);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(10);

        // Fetch ticket 1's corner numbers: row 0 leftmost/rightmost, row 2 leftmost/rightmost
        let ticket_code: String = "T-000001".to_string();
        let t_dto: crate::domain::tambola::TicketDto =
            TicketRepository::get_ticket(&conn, &ticket_code).unwrap();
        let ticket: Ticket = dto_to_ticket(&t_dto.cells);

        let row0: Vec<u8> = ticket.cells[0].iter().filter_map(|&c| c).collect();
        let row2: Vec<u8> = ticket.cells[2].iter().filter_map(|&c| c).collect();
        let c1: u8 = row0[0];
        let c2: u8 = row0[4];
        let c3: u8 = row2[0];
        let c4: u8 = row2[4]; // 4th corner number

        // Find scheme_id for Four Corners in prize_schemes
        let four_corners_scheme_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'FOUR_CORNERS';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let next_number: u8 = (1..=90)
            .find(|&n| n != c1 && n != c2 && n != c3 && n != c4)
            .unwrap();

        // GAME 1: Create game, set sequence, draw until all 4 corner numbers are drawn
        let g1 = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g1_id: i64 = g1.game.id;
        let g1_first: u8 = g1.current.unwrap().number;

        let g1_seq: Vec<u8> = build_valid_permutation(g1_first, &[c1, c2, c3, c4, next_number]);
        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g1_seq).unwrap(), g1_id],
        )
        .unwrap();

        // Determine at which position the 4th corner number is drawn
        let mut corners_drawn: u32 = 0;
        let mut qual_pos: u32 = 0;
        for (idx, &num) in g1_seq.iter().enumerate() {
            if num == c1 || num == c2 || num == c3 || num == c4 {
                corners_drawn += 1;
                if corners_drawn == 4 {
                    qual_pos = (idx + 1) as u32;
                    break;
                }
            }
        }
        assert!(qual_pos >= 4);

        // Advance to position qual_pos
        for p in 1..qual_pos {
            GameRepository::draw_next(&mut conn, g1_id, p).unwrap();
        }

        // Claim at position P (qual_pos) must be ACCEPTED!
        let claim_res_g1: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g1_id, four_corners_scheme_id, &ticket_code)
                .unwrap();
        assert_eq!(claim_res_g1.status, "ACCEPTED");
        assert_eq!(claim_res_g1.qualification_position, Some(qual_pos));
        assert_eq!(claim_res_g1.winner_position, Some(1));
        assert!(claim_res_g1.rejection_code.is_none());

        // End Game 1 to clear active/paused constraint
        GameRepository::end_game(&mut conn, g1_id).unwrap();

        // GAME 2: Create fresh game with identical setup, but advance to P+1 before claiming
        let g2 = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g2_id: i64 = g2.game.id;
        let g2_first: u8 = g2.current.unwrap().number;
        let g2_seq: Vec<u8> = build_valid_permutation(g2_first, &[c1, c2, c3, c4, next_number]);
        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g2_seq).unwrap(), g2_id],
        )
        .unwrap();

        // Draw up to qual_pos, and then ONE MORE to advance to qual_pos + 1
        for p in 1..=qual_pos {
            GameRepository::draw_next(&mut conn, g2_id, p).unwrap();
        }

        // Claim at position P+1 must be REJECTED as LATE_CLAIM!
        let claim_res_g2: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g2_id, four_corners_scheme_id, &ticket_code)
                .unwrap();
        assert_eq!(claim_res_g2.status, "REJECTED");
        assert_eq!(
            claim_res_g2.rejection_code,
            Some(ClaimRejectionCode::LateClaim)
        );
        assert_eq!(claim_res_g2.qualification_position, Some(qual_pos));
        assert_eq!(claim_res_g2.current_draw_position, qual_pos + 1);
        assert!(claim_res_g2.winner_position.is_none());
    }

    #[test]
    fn test_not_completed_rejection() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 1);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(20);

        let game = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id: i64 = game.game.id;

        // At position 1, claiming Full House (scheme 6) must be NOT_COMPLETED
        let full_house_scheme_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'FULL_HOUSE';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let res: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, full_house_scheme_id, "T-000001")
                .unwrap();
        assert_eq!(res.status, "REJECTED");
        assert_eq!(res.rejection_code, Some(ClaimRejectionCode::NotCompleted));
        assert!(res.qualification_position.is_none());
    }

    #[test]
    fn test_eligibility_matrix_and_fresh_game() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 2);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(30);

        let t1_dto: crate::domain::tambola::TicketDto =
            TicketRepository::get_ticket(&conn, "T-000001").unwrap();
        let t1: Ticket = dto_to_ticket(&t1_dto.cells);

        let t1_top_row: Vec<u8> = t1.cells[0].iter().filter_map(|&c| c).collect();
        let t1_middle_row: Vec<u8> = t1.cells[1].iter().filter_map(|&c| c).collect();
        let t1_bottom_row: Vec<u8> = t1.cells[2].iter().filter_map(|&c| c).collect();
        assert_eq!(t1_top_row.len(), 5);
        assert_eq!(t1_middle_row.len(), 5);
        assert_eq!(t1_bottom_row.len(), 5);

        let early_five_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let top_line_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'TOP_LINE';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let bottom_line_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'BOTTOM_LINE';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let full_house_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'FULL_HOUSE';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let assert_claim_db_row =
            |c: &Connection,
             claim_id: i64,
             expected_status: &str,
             expected_code: Option<&str>,
             expected_qual_pos: Option<u32>| {
                let (status, rejection_code, qual_pos): (String, Option<String>, Option<u32>) = c
                .query_row(
                    "SELECT status, rejection_code, qualification_position FROM claims WHERE id = ?1;",
                    params![claim_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .unwrap();
                assert_eq!(status, expected_status);
                assert_eq!(rejection_code.as_deref(), expected_code);
                assert_eq!(qual_pos, expected_qual_pos);
            };

        // --- GAME 1: Test Early Five, ONLY_FULL_HOUSE_ELIGIBLE, and Full House on T-000001 ---
        let g1 = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g1_id: i64 = g1.game.id;
        let g1_first: u8 = g1.current.unwrap().number;

        // Priority sequence: 5 top-row numbers first! Then middle and bottom.
        let mut g1_priority: Vec<u8> = Vec::new();
        g1_priority.extend_from_slice(&t1_top_row);
        g1_priority.extend_from_slice(&t1_middle_row);
        g1_priority.extend_from_slice(&t1_bottom_row);

        let g1_seq: Vec<u8> = build_valid_permutation(g1_first, &g1_priority);
        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g1_seq).unwrap(), g1_id],
        )
        .unwrap();

        // Find position where 5th top-row number is drawn (p5) and 15th ticket number (p15)
        let mut t1_top_drawn: u32 = 0;
        let mut t1_all_drawn: u32 = 0;
        let mut p5: u32 = 0;
        let mut p15: u32 = 0;
        for (idx, &num) in g1_seq.iter().enumerate() {
            if t1_top_row.contains(&num) {
                t1_top_drawn += 1;
                if t1_top_drawn == 5 && p5 == 0 {
                    p5 = (idx + 1) as u32;
                }
            }
            if g1_priority.contains(&num) {
                t1_all_drawn += 1;
                if t1_all_drawn == 15 {
                    p15 = (idx + 1) as u32;
                    break;
                }
            }
        }

        // Draw up to p5 (both Early Five and Top Line qualify at p5!)
        for p in 1..p5 {
            GameRepository::draw_next(&mut conn, g1_id, p).unwrap();
        }

        // 1. Claim Early Five at p5 -> ACCEPTED
        let res_ef: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g1_id, early_five_id, "T-000001").unwrap();
        assert_eq!(res_ef.status, "ACCEPTED");
        assert_eq!(res_ef.winner_position, Some(1));
        assert_eq!(res_ef.qualification_position, Some(p5));
        assert_claim_db_row(&conn, res_ef.claim_id, "ACCEPTED", None, Some(p5));

        // 2. Claiming same scheme again at p5 -> ALREADY_WON_SCHEME
        let res_ef_dup: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g1_id, early_five_id, "T-000001").unwrap();
        assert_eq!(res_ef_dup.status, "REJECTED");
        assert_eq!(res_ef_dup.winner_position, None);
        assert_eq!(
            res_ef_dup.rejection_code,
            Some(ClaimRejectionCode::AlreadyWonScheme)
        );
        assert_claim_db_row(
            &conn,
            res_ef_dup.claim_id,
            "REJECTED",
            Some("ALREADY_WON_SCHEME"),
            Some(p5),
        );

        // 3. Claim Top Line at p5 (which also qualifies at p5):
        // Rejected specifically as ONLY_FULL_HOUSE_ELIGIBLE because ticket already won non-FH!
        let res_top_p5: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g1_id, top_line_id, "T-000001").unwrap();
        assert_eq!(res_top_p5.status, "REJECTED");
        assert_eq!(res_top_p5.winner_position, None);
        assert_eq!(
            res_top_p5.rejection_code,
            Some(ClaimRejectionCode::OnlyFullHouseEligible)
        );
        assert_eq!(res_top_p5.qualification_position, Some(p5));
        assert_claim_db_row(
            &conn,
            res_top_p5.claim_id,
            "REJECTED",
            Some("ONLY_FULL_HOUSE_ELIGIBLE"),
            Some(p5),
        );

        // 4. Draw remaining numbers up to p15 (all 15 numbers completed at p15)
        for p in p5..p15 {
            GameRepository::draw_next(&mut conn, g1_id, p).unwrap();
        }

        // Claiming Full House at p15 (exact qualification pos) -> ACCEPTED!
        let res_fh: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g1_id, full_house_id, "T-000001").unwrap();
        assert_eq!(res_fh.status, "ACCEPTED");
        assert_eq!(res_fh.winner_position, Some(1));
        assert_eq!(res_fh.qualification_position, Some(p15));
        assert_claim_db_row(&conn, res_fh.claim_id, "ACCEPTED", None, Some(p15));

        // 5. Subsequent claim for Bottom Line after Full House on T-000001 -> ALREADY_WON_FULL_HOUSE
        let res_after_fh: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g1_id, bottom_line_id, "T-000001").unwrap();
        assert_eq!(res_after_fh.status, "REJECTED");
        assert_eq!(res_after_fh.winner_position, None);
        assert_eq!(
            res_after_fh.rejection_code,
            Some(ClaimRejectionCode::AlreadyWonFullHouse)
        );
        assert_claim_db_row(
            &conn,
            res_after_fh.claim_id,
            "REJECTED",
            Some("ALREADY_WON_FULL_HOUSE"),
            Some(p15),
        );

        // End Game 1 before starting Game 3
        GameRepository::end_game(&mut conn, g1_id).unwrap();

        // --- Matrix Row 3 Test on fresh ticket T-000002 (no prior wins): ---
        // Arrange draws in Game 3 so the last number completes both Full House and Bottom Line at p_fh.
        // At p_fh, claim Full House (ACCEPTED), then claim Bottom Line at p_fh -> ALREADY_WON_FULL_HOUSE.
        let t2_dto: crate::domain::tambola::TicketDto =
            TicketRepository::get_ticket(&conn, "T-000002").unwrap();
        let t2: Ticket = dto_to_ticket(&t2_dto.cells);
        let t2_top: Vec<u8> = t2.cells[0].iter().filter_map(|&c| c).collect();
        let t2_mid: Vec<u8> = t2.cells[1].iter().filter_map(|&c| c).collect();
        let t2_bot: Vec<u8> = t2.cells[2].iter().filter_map(|&c| c).collect();

        // 14 numbers (top + mid + first 4 of bot), then 15th number is the last bottom row number
        let last_bot_num: u8 = t2_bot[4];
        let mut t2_priority: Vec<u8> = Vec::new();
        t2_priority.extend_from_slice(&t2_top);
        t2_priority.extend_from_slice(&t2_mid);
        t2_priority.extend_from_slice(&t2_bot[..4]);
        t2_priority.push(last_bot_num);

        let g3 = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g3_id: i64 = g3.game.id;
        let g3_first: u8 = g3.current.unwrap().number;
        let g3_seq: Vec<u8> = build_valid_permutation(g3_first, &t2_priority);
        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g3_seq).unwrap(), g3_id],
        )
        .unwrap();

        let mut p_fh: u32 = 0;
        let mut t2_all_drawn: u32 = 0;
        for (idx, &num) in g3_seq.iter().enumerate() {
            if t2_priority.contains(&num) {
                t2_all_drawn += 1;
                if t2_all_drawn == 15 {
                    p_fh = (idx + 1) as u32;
                    break;
                }
            }
        }

        for p in 1..p_fh {
            GameRepository::draw_next(&mut conn, g3_id, p).unwrap();
        }

        // T-000002 has 0 prior wins. At p_fh, completes both Full House and Bottom Line.
        // First claim Full House -> ACCEPTED
        let res_t2_fh: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g3_id, full_house_id, "T-000002").unwrap();
        assert_eq!(res_t2_fh.status, "ACCEPTED");
        assert_eq!(res_t2_fh.winner_position, Some(1));
        assert_eq!(res_t2_fh.qualification_position, Some(p_fh));
        assert_claim_db_row(&conn, res_t2_fh.claim_id, "ACCEPTED", None, Some(p_fh));

        // Immediately claim Bottom Line at p_fh -> REJECTED with ALREADY_WON_FULL_HOUSE
        let res_t2_bl: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g3_id, bottom_line_id, "T-000002").unwrap();
        assert_eq!(res_t2_bl.status, "REJECTED");
        assert_eq!(res_t2_bl.winner_position, None);
        assert_eq!(
            res_t2_bl.rejection_code,
            Some(ClaimRejectionCode::AlreadyWonFullHouse)
        );
        assert_eq!(res_t2_bl.qualification_position, Some(p_fh));
        assert_claim_db_row(
            &conn,
            res_t2_bl.claim_id,
            "REJECTED",
            Some("ALREADY_WON_FULL_HOUSE"),
            Some(p_fh),
        );

        // End Game 3
        GameRepository::end_game(&mut conn, g3_id).unwrap();

        // 6. Fresh Game 2 with same batch: ticket 1 has fresh eligibility!
        let g2 = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let g2_id: i64 = g2.game.id;
        let g2_first: u8 = g2.current.unwrap().number;
        let mut all_t1_nums: Vec<u8> = Vec::new();
        all_t1_nums.extend_from_slice(&t1_top_row);
        all_t1_nums.extend_from_slice(&t1_middle_row);
        all_t1_nums.extend_from_slice(&t1_bottom_row);
        let g2_seq: Vec<u8> = build_valid_permutation(g2_first, &all_t1_nums);
        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g2_seq).unwrap(), g2_id],
        )
        .unwrap();

        let mut g2_t_drawn: u32 = 0;
        let mut g2_p5: u32 = 0;
        for (idx, &num) in g2_seq.iter().enumerate() {
            if all_t1_nums.contains(&num) {
                g2_t_drawn += 1;
                if g2_t_drawn == 5 {
                    g2_p5 = (idx + 1) as u32;
                    break;
                }
            }
        }

        for p in 1..g2_p5 {
            GameRepository::draw_next(&mut conn, g2_id, p).unwrap();
        }

        // Ticket 1 in Game 2 wins Early Five afresh!
        let res_g2_ef: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, g2_id, early_five_id, "T-000001").unwrap();
        assert_eq!(res_g2_ef.status, "ACCEPTED");
        assert_eq!(res_g2_ef.winner_position, Some(1));
        assert_claim_db_row(&conn, res_g2_ef.claim_id, "ACCEPTED", None, Some(g2_p5));

        // Game 1 winners are still preserved
        let g1_winners_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM winners WHERE game_id = ?1;",
                params![g1_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(g1_winners_count, 2);
    }

    #[test]
    fn test_winner_capacity_and_rejected_claims() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 2);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(40);

        let early_five_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let game = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id: i64 = game.game.id;

        let t1_dto = TicketRepository::get_ticket(&conn, "T-000001").unwrap();
        let nums1: Vec<u8> = t1_dto.cells.iter().map(|c| c.number).take(5).collect();

        let g_first: u8 = game.current.unwrap().number;
        let g_seq: Vec<u8> = build_valid_permutation(g_first, &nums1);

        conn.execute(
            "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
            params![serde_json::to_string(&g_seq).unwrap(), game_id],
        )
        .unwrap();

        let mut t1_cnt: u32 = 0;
        let mut p_t1: u32 = 0;
        for (idx, &num) in g_seq.iter().enumerate() {
            if nums1.contains(&num) {
                t1_cnt += 1;
                if t1_cnt == 5 {
                    p_t1 = (idx + 1) as u32;
                    break;
                }
            }
        }
        assert!(p_t1 >= 5);

        for p in 1..p_t1 {
            GameRepository::draw_next(&mut conn, game_id, p).unwrap();
        }

        // 1. An invalid rejected claim submitted -> does NOT consume a winner slot
        let claim_inv: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-999999").unwrap();
        assert_eq!(claim_inv.status, "REJECTED");
        assert_eq!(
            claim_inv.rejection_code,
            Some(ClaimRejectionCode::TicketNotFound)
        );

        let win_count_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM winners WHERE game_id = ?1 AND scheme_id = ?2;",
                params![game_id, early_five_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            win_count_before, 0,
            "Rejected claim must not touch winners table"
        );

        // 2. Artificially fill winner capacity (max_winners = 1) by allocating slot 1 to ticket 2 at p_t1
        conn.execute(
            "INSERT INTO claims (game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position, qualification_position, status)
             VALUES (?1, 2, 'T-000002', ?2, ?3, ?3, 'ACCEPTED');",
            params![game_id, early_five_id, p_t1],
        )
        .unwrap();
        let claim_t2_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO winners (game_id, scheme_id, ticket_id, claim_id, is_full_house, winner_position)
             VALUES (?1, ?2, 2, ?3, 0, 1);",
            params![game_id, early_five_id, claim_t2_id],
        )
        .unwrap();

        let win_count_full: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM winners WHERE game_id = ?1 AND scheme_id = ?2;",
                params![game_id, early_five_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(win_count_full, 1, "Capacity (max 1) is now full");

        // 3. Ticket 1 claims at p_t1 (exact position where it qualifies, not late, zero prior wins)
        // Must be REJECTED with WINNER_LIMIT_REACHED
        let claim1: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
        assert_eq!(claim1.status, "REJECTED");
        assert_eq!(
            claim1.rejection_code,
            Some(ClaimRejectionCode::WinnerLimitReached)
        );
        assert_eq!(claim1.qualification_position, Some(p_t1));
        assert!(claim1.winner_position.is_none());

        // Winners table count remains exactly 1
        let win_count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM winners WHERE game_id = ?1 AND scheme_id = ?2;",
                params![game_id, early_five_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(win_count_after, 1);
    }

    #[test]
    fn test_paused_game_accepts_ended_and_completed_reject() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 1);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(50);

        let early_five_id: i64 = conn
            .query_row(
                "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let game = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id: i64 = game.game.id;

        // Pause the game
        GameRepository::pause_game(&mut conn, game_id).unwrap();

        // Paused game accepts claims (e.g. unknown ticket returns TicketNotFound, not GameNotAcceptingClaims)
        let claim_paused: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
        assert_ne!(
            claim_paused.rejection_code,
            Some(ClaimRejectionCode::GameNotAcceptingClaims),
            "PAUSED game must accept claims"
        );

        // Resume and end the game
        GameRepository::resume_game(&mut conn, game_id).unwrap();
        GameRepository::end_game(&mut conn, game_id).unwrap();

        // Ended game must reject with GAME_NOT_ACCEPTING_CLAIMS
        let claim_ended: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
        assert_eq!(claim_ended.status, "REJECTED");
        assert_eq!(
            claim_ended.rejection_code,
            Some(ClaimRejectionCode::GameNotAcceptingClaims)
        );
    }

    #[test]
    fn test_unknown_ticket_and_scheme_not_in_game_recorded() {
        let mut conn: Connection = setup_test_db();
        let batch_id: i64 = populate_valid_media_and_batch(&mut conn, 1);
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(60);

        let game = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
        let game_id: i64 = game.game.id;

        // 1. Unknown ticket
        let claim_unknown: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, 1, "T-999999").unwrap();
        assert_eq!(claim_unknown.status, "REJECTED");
        assert_eq!(
            claim_unknown.rejection_code,
            Some(ClaimRejectionCode::TicketNotFound)
        );

        // Verify it was recorded in DB with NULL ticket_id
        let ticket_id_val: Option<i64> = conn
            .query_row(
                "SELECT ticket_id FROM claims WHERE id = ?1;",
                params![claim_unknown.claim_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(ticket_id_val.is_none());

        // 2. Scheme not in snapshot
        let claim_noscheme: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, 9999, "T-000001").unwrap();
        assert_eq!(claim_noscheme.status, "REJECTED");
        assert_eq!(
            claim_noscheme.rejection_code,
            Some(ClaimRejectionCode::SchemeNotInGame)
        );

        // 3. Ticket not in this game (generate batch 2, ticket from batch 2)
        let batch_2 =
            TicketRepository::generate_batch(&mut conn, "Other Batch", 1, &mut rng, |_, _| {})
                .unwrap();
        let t2_code: String = TicketRepository::list_batch_tickets(&conn, batch_2.id, 0, 10)
            .unwrap()
            .items[0]
            .code
            .clone();

        let claim_other: ClaimResult =
            ClaimRepository::submit_claim(&mut conn, game_id, 1, &t2_code).unwrap();
        assert_eq!(claim_other.status, "REJECTED");
        assert_eq!(
            claim_other.rejection_code,
            Some(ClaimRejectionCode::TicketNotInGame)
        );
    }

    #[test]
    fn test_persistence_across_reopen() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reopen_claim_test.sqlite");

        let game_id: i64;
        let early_five_id: i64;
        let claim_id: i64;

        {
            let mut conn = Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            run_migrations(&conn).unwrap();

            let batch_id = populate_valid_media_and_batch(&mut conn, 1);
            let mut rng = DeterministicRandomSource::new(70);

            early_five_id = conn
                .query_row(
                    "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                    [],
                    |r| r.get(0),
                )
                .unwrap();

            let g = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
            game_id = g.game.id;

            // Submit an unknown ticket claim
            let claim =
                ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-UNKNOWN")
                    .unwrap();
            claim_id = claim.claim_id;
        }

        // Reopen database connection
        {
            let conn2 = Connection::open(&db_path).unwrap();
            conn2.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

            let claims = ClaimRepository::list_game_claims(&conn2, game_id).unwrap();
            assert_eq!(claims.len(), 1);
            assert_eq!(claims[0].claim_id, claim_id);
            assert_eq!(claims[0].status, "REJECTED");
            assert_eq!(
                claims[0].rejection_code,
                Some(ClaimRejectionCode::TicketNotFound)
            );
            assert_eq!(claims[0].ticket_code, "T-UNKNOWN");
        }
    }

    #[test]
    fn test_concurrency_race() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("concurrency_race.sqlite");

        let game_id: i64;
        let early_five_id: i64;

        {
            let mut conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA journal_mode = WAL;
                 PRAGMA busy_timeout = 5000;",
            )
            .unwrap();
            run_migrations(&conn).unwrap();

            let batch_id = populate_valid_media_and_batch(&mut conn, 2);
            let mut rng = DeterministicRandomSource::new(80);

            early_five_id = conn
                .query_row(
                    "SELECT id FROM prize_schemes WHERE scheme_type = 'EARLY_FIVE';",
                    [],
                    |r| r.get(0),
                )
                .unwrap();

            let g = GameRepository::create_game(&mut conn, batch_id, &mut rng).unwrap();
            game_id = g.game.id;
        }

        // Two threads submit claims concurrently for game_id and early_five_id
        let db_path_1 = db_path.clone();
        let db_path_2 = db_path.clone();

        let handle1 = std::thread::spawn(move || {
            let mut conn1 = Connection::open(&db_path_1).unwrap();
            conn1.execute_batch("PRAGMA busy_timeout = 5000;").unwrap();
            ClaimRepository::submit_claim(&mut conn1, game_id, early_five_id, "T-000001")
        });

        let handle2 = std::thread::spawn(move || {
            let mut conn2 = Connection::open(&db_path_2).unwrap();
            conn2.execute_batch("PRAGMA busy_timeout = 5000;").unwrap();
            ClaimRepository::submit_claim(&mut conn2, game_id, early_five_id, "T-000002")
        });

        let res1 = handle1.join().unwrap();
        let res2 = handle2.join().unwrap();

        assert!(
            res1.is_ok(),
            "Thread 1 must succeed without raw error: {:?}",
            res1
        );
        assert!(
            res2.is_ok(),
            "Thread 2 must succeed without raw error: {:?}",
            res2
        );

        let c1 = res1.unwrap();
        let c2 = res2.unwrap();

        // Both claims must be persisted
        let conn_check = Connection::open(&db_path).unwrap();
        let list = ClaimRepository::list_game_claims(&conn_check, game_id).unwrap();
        assert_eq!(list.len(), 2, "Both claims must be recorded");

        // Both executed safely without panic or deadlock
        assert_eq!(c1.game_id, game_id);
        assert_eq!(c2.game_id, game_id);
    }
}
