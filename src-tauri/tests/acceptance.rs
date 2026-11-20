use media_tambola_lib::db::connection::initialize_database;
use media_tambola_lib::db::repositories::claim::ClaimRepository;
use media_tambola_lib::db::repositories::game::GameRepository;
use media_tambola_lib::db::repositories::history::HistoryRepository;
use media_tambola_lib::db::repositories::media::MediaRepository;
use media_tambola_lib::db::repositories::ticket::TicketRepository;
use media_tambola_lib::domain::claims::claim_validation::ClaimRejectionCode;
use media_tambola_lib::domain::game::GameStatus;
use media_tambola_lib::domain::media::{MediaType, SaveMediaInput, SourceType, ValidationStatus};
use media_tambola_lib::domain::random::DeterministicRandomSource;
use media_tambola_lib::domain::tambola::ticket_generator::Ticket;
use media_tambola_lib::services::media_validation::validate_slot;
use rusqlite::params;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

fn spawn_mock_media_server() -> (String, mpsc::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port: u16 = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel::<()>();
    listener.set_nonblocking(true).unwrap();

    thread::spawn(move || loop {
        if rx.try_recv().is_ok() {
            break;
        }
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let jpeg_bytes: [u8; 12] = [
                0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
            ];
            let response: String = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    jpeg_bytes.len()
                );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(&jpeg_bytes);
            let _ = stream.flush();
        }
        thread::sleep(std::time::Duration::from_millis(10));
    });

    (format!("http://127.0.0.1:{}/remote.jpg", port), tx)
}

fn dto_to_ticket(cells_vec: &[media_tambola_lib::domain::tambola::TicketCell]) -> Ticket {
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

#[tokio::test]
async fn test_end_to_end_acceptance_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path: PathBuf = temp_dir.path().join("acceptance_game.sqlite");
    let media_dir: PathBuf = temp_dir.path().join("media_files");
    fs::create_dir_all(&media_dir).unwrap();

    // 1. Local mock media server for remote validation + local dummy JPEG file
    let (remote_url, server_shutdown) = spawn_mock_media_server();

    let dummy_jpg = media_dir.join("sample.jpg");
    let mut f = fs::File::create(&dummy_jpg).unwrap();
    f.write_all(&[
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    ])
    .unwrap();

    let mut conn = initialize_database(&db_path).unwrap();

    // 2. Populate 90 media slots (slots 1..89 local file, slot 90 remote URL)
    for num in 1..=89 {
        MediaRepository::upsert(
            &conn,
            &SaveMediaInput {
                number: num,
                media_type: MediaType::Image,
                source_type: SourceType::Local,
                source: dummy_jpg.to_string_lossy().to_string(),
                thumbnail_source: None,
                title: format!("વસ્તુ {}", num),
            },
        )
        .unwrap();
    }

    MediaRepository::upsert(
        &conn,
        &SaveMediaInput {
            number: 90,
            media_type: MediaType::Image,
            source_type: SourceType::Remote,
            source: remote_url.clone(),
            thumbnail_source: None,
            title: "રિમોટ વસ્તુ 90".to_string(),
        },
    )
    .unwrap();

    // Run validate_all over all 90 slots and assert 90 valid
    for num in 1..=90 {
        let slot = MediaRepository::get_by_number(&conn, num).unwrap();
        let (status, err) = validate_slot(&slot).await;
        assert_eq!(
            status,
            ValidationStatus::Valid,
            "Slot {} must validate successfully",
            num
        );
        MediaRepository::update_validation_status(&conn, num, status, err.as_deref()).unwrap();
    }

    let all_slots = MediaRepository::list_all(&conn).unwrap();
    let valid_count = all_slots
        .iter()
        .filter(|s| s.validation_status == ValidationStatus::Valid)
        .count();
    assert_eq!(valid_count, 90, "All 90 media items must be validated");

    // 3. Generate 100 tickets and assert unique codes and hashes
    let mut rng = DeterministicRandomSource::new(101);
    let batch =
        TicketRepository::generate_batch(&mut conn, "Acceptance Batch", 100, &mut rng, |_, _| {})
            .unwrap();
    assert_eq!(batch.generated_count, 100);

    let tickets_page = TicketRepository::list_batch_tickets(&conn, batch.id, 0, 100).unwrap();
    assert_eq!(tickets_page.total, 100);
    assert_eq!(tickets_page.items.len(), 100);

    let mut code_set = HashSet::new();
    for t in &tickets_page.items {
        assert!(
            code_set.insert(t.code.clone()),
            "Ticket code must be unique"
        );
    }

    // 4. Configure schemes: Early Five (max_winners=1), Top Line, Full House (max_winners=1)
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

    // 5. Construct draw sequence for simultaneous qualification & winner limit assertion
    let t1_dto = TicketRepository::get_ticket(&conn, "T-000001").unwrap();
    let t1: Ticket = dto_to_ticket(&t1_dto.cells);
    let mut t1_numbers: Vec<u8> = Vec::new();
    for row in &t1.cells {
        for n in row.iter().flatten() {
            t1_numbers.push(*n);
        }
    }
    let t1_set: HashSet<u8> = t1_numbers.iter().cloned().collect();
    let t1_top_set: HashSet<u8> = t1.cells[0].iter().filter_map(|&c| c).collect();
    assert_eq!(t1_top_set.len(), 5);

    // Find a second ticket in the batch sharing at least 1 number with Ticket 1's top row
    let mut second_ticket_code = String::new();
    let mut shared_num = 0u8;
    let mut tk_numbers: Vec<u8> = Vec::new();
    for ticket in &tickets_page.items {
        if ticket.code == "T-000001" {
            continue;
        }
        let tk_ticket = dto_to_ticket(&ticket.cells);
        let mut cur_nums = Vec::new();
        for row in &tk_ticket.cells {
            for n in row.iter().flatten() {
                cur_nums.push(*n);
            }
        }
        let tk_set: HashSet<u8> = cur_nums.iter().cloned().collect();
        let overlap: Vec<u8> = t1_top_set.intersection(&tk_set).cloned().collect();
        if !overlap.is_empty() {
            second_ticket_code = ticket.code.clone();
            shared_num = overlap[0];
            tk_numbers = cur_nums;
            break;
        }
    }
    assert!(!second_ticket_code.is_empty());
    assert!(shared_num > 0);

    // The other 4 numbers of T1's top row
    let t1_unique_4: Vec<u8> = t1_top_set
        .iter()
        .cloned()
        .filter(|&n| n != shared_num)
        .collect();
    assert_eq!(t1_unique_4.len(), 4);

    // Pick 4 unique numbers from Tk (not in T1, not shared)
    let tk_unique_4: Vec<u8> = tk_numbers
        .iter()
        .cloned()
        .filter(|&n| n != shared_num && !t1_set.contains(&n))
        .take(4)
        .collect();
    assert_eq!(tk_unique_4.len(), 4);

    // Remaining 10 numbers of Ticket 1
    let drawn_so_far: HashSet<u8> = t1_unique_4
        .iter()
        .chain(tk_unique_4.iter())
        .chain(std::iter::once(&shared_num))
        .cloned()
        .collect();

    let t1_mid: Vec<u8> = t1.cells[1].iter().filter_map(|&c| c).collect();
    let t1_bot: Vec<u8> = t1.cells[2].iter().filter_map(|&c| c).collect();

    let mut t1_remaining: Vec<u8> = Vec::new();
    for n in t1_mid {
        if !drawn_so_far.contains(&n) {
            t1_remaining.push(n);
        }
    }
    for n in t1_bot {
        if !drawn_so_far.contains(&n) {
            t1_remaining.push(n);
        }
    }
    assert_eq!(t1_remaining.len(), 10);

    // Assemble deterministic 90-number draw sequence:
    // Positions 1..4: t1_unique_4 (4 numbers)
    // Positions 5..8: tk_unique_4 (4 numbers)
    // Position 9: shared_num (T1 reaches 5 at pos 9, Tk reaches 5 at pos 9)
    // Positions 10..19: t1_remaining (T1 reaches 15 at pos 19)
    // Positions 20..90: all remaining numbers 1..90
    let mut planned_seq: Vec<u8> = Vec::with_capacity(90);
    planned_seq.extend_from_slice(&t1_unique_4);
    planned_seq.extend_from_slice(&tk_unique_4);
    planned_seq.push(shared_num);
    planned_seq.extend_from_slice(&t1_remaining);

    let mut used_in_planned: HashSet<u8> = planned_seq.iter().cloned().collect();
    for n in 1..=90 {
        if !used_in_planned.contains(&n) {
            planned_seq.push(n);
            used_in_planned.insert(n);
        }
    }
    assert_eq!(planned_seq.len(), 90);

    // 6. Create Game: inject planned draw sequence
    let mut game_rng = DeterministicRandomSource::new(202);
    let game = GameRepository::create_game(&mut conn, batch.id, &mut game_rng).unwrap();
    let game_id: i64 = game.game.id;

    conn.execute(
        "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
        params![serde_json::to_string(&planned_seq).unwrap(), game_id],
    )
    .unwrap();
    conn.execute(
        "UPDATE game_calls SET number = ?1 WHERE game_id = ?2 AND draw_position = 1;",
        params![planned_seq[0], game_id],
    )
    .unwrap();

    // 7. Advance draws to position 9 (both T1 and Tk qualify for Early Five at pos 9)
    for p in 1..9 {
        GameRepository::draw_next(&mut conn, game_id, p).unwrap();
    }

    // a) Incomplete claim rejected
    let res_inc =
        ClaimRepository::submit_claim(&mut conn, game_id, full_house_id, "T-000001").unwrap();
    assert_eq!(res_inc.status, "REJECTED");
    assert_eq!(
        res_inc.rejection_code,
        Some(ClaimRejectionCode::NotCompleted)
    );

    // b) First Early Five claim accepted (T-000001)
    let res_ef1 =
        ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
    assert_eq!(res_ef1.status, "ACCEPTED");
    assert_eq!(res_ef1.winner_position, Some(1));

    // c) Second Early Five claim at same draw position rejected: WinnerLimitReached (max_winners = 1)
    let res_ef2 =
        ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, &second_ticket_code)
            .unwrap();
    assert_eq!(res_ef2.status, "REJECTED");
    assert_eq!(
        res_ef2.rejection_code,
        Some(ClaimRejectionCode::WinnerLimitReached),
        "Second ticket at same draw position must receive WinnerLimitReached"
    );

    // d) Non-FH restriction: attempting Top Line on T-000001 is rejected as ONLY_FULL_HOUSE_ELIGIBLE
    let res_tl =
        ClaimRepository::submit_claim(&mut conn, game_id, top_line_id, "T-000001").unwrap();
    assert_eq!(res_tl.status, "REJECTED");
    assert_eq!(
        res_tl.rejection_code,
        Some(ClaimRejectionCode::OnlyFullHouseEligible)
    );

    // 8. Advance draw by 1 -> position 10: late claim rejected
    GameRepository::draw_next(&mut conn, game_id, 9).unwrap();
    let res_late =
        ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
    assert_eq!(res_late.status, "REJECTED");
    assert_eq!(res_late.rejection_code, Some(ClaimRejectionCode::LateClaim));

    // 9. Draw up to position 19: Ticket 1 completes Full House
    for p in 10..19 {
        GameRepository::draw_next(&mut conn, game_id, p).unwrap();
    }

    // e) Full House accepted after non-FH win
    let res_fh =
        ClaimRepository::submit_claim(&mut conn, game_id, full_house_id, "T-000001").unwrap();
    assert_eq!(res_fh.status, "ACCEPTED");
    assert_eq!(res_fh.winner_position, Some(1));

    // f) Post-Full-House guard: claiming a DIFFERENT scheme (Bottom Line, qualifying at pos 19)
    // after winning Full House returns ALREADY_WON_FULL_HOUSE (Step 7 of evaluate_claim)
    let res_post_fh =
        ClaimRepository::submit_claim(&mut conn, game_id, bottom_line_id, "T-000001").unwrap();
    assert_eq!(res_post_fh.status, "REJECTED");
    assert_eq!(
        res_post_fh.rejection_code,
        Some(ClaimRejectionCode::AlreadyWonFullHouse),
        "Claiming another scheme after winning Full House must be rejected with AlreadyWonFullHouse"
    );

    // 10. Crash simulation: drop database connection and reopen
    drop(conn);

    let mut conn2 = initialize_database(&db_path).unwrap();
    let active_game = GameRepository::get_active_game(&conn2)
        .unwrap()
        .expect("Active game must recover after database drop/reopen");
    assert_eq!(active_game.game.id, game_id);
    assert_eq!(active_game.game.current_draw_position, 19);
    assert_eq!(active_game.game.status, GameStatus::Active);

    // Assert that persisted claims rows (both accepted and rejected) are completely intact
    let claims = ClaimRepository::list_game_claims(&conn2, game_id).unwrap();
    assert_eq!(claims.len(), 7, "All 7 claim submissions must be preserved");

    let accepted_claims: Vec<_> = claims.iter().filter(|c| c.status == "ACCEPTED").collect();
    assert_eq!(accepted_claims.len(), 2);
    assert_eq!(accepted_claims[0].winner_position, Some(1));
    assert_eq!(accepted_claims[1].winner_position, Some(1));

    let rejected_claims: Vec<_> = claims.iter().filter(|c| c.status == "REJECTED").collect();
    assert_eq!(rejected_claims.len(), 5);

    let rejected_codes: HashSet<_> = rejected_claims
        .iter()
        .filter_map(|c| c.rejection_code)
        .collect();
    assert!(rejected_codes.contains(&ClaimRejectionCode::NotCompleted));
    assert!(rejected_codes.contains(&ClaimRejectionCode::WinnerLimitReached));
    assert!(rejected_codes.contains(&ClaimRejectionCode::OnlyFullHouseEligible));
    assert!(rejected_codes.contains(&ClaimRejectionCode::LateClaim));
    assert!(rejected_codes.contains(&ClaimRejectionCode::AlreadyWonFullHouse));

    // 11. End game, start a fresh game: fresh state & historical records preserved
    GameRepository::end_game(&mut conn2, game_id).unwrap();

    let mut g2_rng = DeterministicRandomSource::new(303);
    let g2 = GameRepository::create_game(&mut conn2, batch.id, &mut g2_rng).unwrap();
    assert_eq!(g2.game.status, GameStatus::Active);
    assert_ne!(g2.game.id, game_id);

    let history_stats = HistoryRepository::get_dashboard_stats(&conn2).unwrap();
    assert_eq!(history_stats.games_played, 1);
    assert_eq!(history_stats.recent_winners.len(), 2);

    let _ = server_shutdown.send(());
}
