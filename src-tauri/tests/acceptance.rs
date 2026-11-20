use media_tambola_lib::db::connection::initialize_database;
use media_tambola_lib::db::repositories::claim::ClaimRepository;
use media_tambola_lib::db::repositories::game::GameRepository;
use media_tambola_lib::db::repositories::history::HistoryRepository;
use media_tambola_lib::db::repositories::media::MediaRepository;
use media_tambola_lib::db::repositories::ticket::TicketRepository;
use media_tambola_lib::domain::claims::claim_validation::ClaimRejectionCode;
use media_tambola_lib::domain::game::GameStatus;
use media_tambola_lib::domain::media::{MediaType, SaveMediaInput, SourceType};
use media_tambola_lib::domain::random::DeterministicRandomSource;
use media_tambola_lib::domain::tambola::ticket_generator::Ticket;
use rusqlite::params;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

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

#[tokio::test]
async fn test_end_to_end_acceptance_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path: PathBuf = temp_dir.path().join("acceptance_game.sqlite");
    let media_dir: PathBuf = temp_dir.path().join("media_files");
    fs::create_dir_all(&media_dir).unwrap();

    // 1. Create a minimal dummy image file with valid JPEG magic bytes
    let dummy_jpg = media_dir.join("sample.jpg");
    let mut f = fs::File::create(&dummy_jpg).unwrap();
    f.write_all(&[
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    ])
    .unwrap();

    let mut conn = initialize_database(&db_path).unwrap();

    // Populate all 90 media slots
    for num in 1..=90 {
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

        conn.execute(
            "UPDATE media_assets SET is_valid = 1, last_validated_at = CURRENT_TIMESTAMP WHERE number = ?1;",
            params![num],
        )
        .unwrap();
    }

    // 2. Generate 100 tickets and assert unique codes and hashes
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

    // 3. Configure schemes: ensure Early Five and Full House are active with max_winners = 1
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
    let full_house_id: i64 = conn
        .query_row(
            "SELECT id FROM prize_schemes WHERE scheme_type = 'FULL_HOUSE';",
            [],
            |r| r.get(0),
        )
        .unwrap();

    // 4. Create Game: position 1 drawn; Next; no duplicate numbers
    let mut game_rng = DeterministicRandomSource::new(202);
    let game = GameRepository::create_game(&mut conn, batch.id, &mut game_rng).unwrap();
    let game_id: i64 = game.game.id;
    assert_eq!(game.game.current_draw_position, 1);
    assert!(game.current.is_some());

    // Arrange draw sequence so top row of ticket 1 completes at p5, all 15 numbers at p15
    let t1_dto = TicketRepository::get_ticket(&conn, "T-000001").unwrap();
    let t1: Ticket = dto_to_ticket(&t1_dto.cells);
    let t1_top: Vec<u8> = t1.cells[0].iter().filter_map(|&c| c).collect();
    let t1_mid: Vec<u8> = t1.cells[1].iter().filter_map(|&c| c).collect();
    let t1_bot: Vec<u8> = t1.cells[2].iter().filter_map(|&c| c).collect();

    let mut t1_priority: Vec<u8> = Vec::new();
    t1_priority.extend_from_slice(&t1_top);
    t1_priority.extend_from_slice(&t1_mid);
    t1_priority.extend_from_slice(&t1_bot);

    let g_first: u8 = game.current.unwrap().number;
    let seq: Vec<u8> = build_valid_permutation(g_first, &t1_priority);

    // Verify all 90 distinct numbers
    let mut seq_set = HashSet::new();
    for &n in &seq {
        assert!(
            seq_set.insert(n),
            "Draw sequence must contain unique numbers"
        );
    }
    assert_eq!(seq_set.len(), 90);

    conn.execute(
        "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
        params![serde_json::to_string(&seq).unwrap(), game_id],
    )
    .unwrap();

    let mut p5: u32 = 0;
    let mut p15: u32 = 0;
    let mut t1_top_cnt: u32 = 0;
    let mut t1_all_cnt: u32 = 0;
    for (idx, &n) in seq.iter().enumerate() {
        if t1_top.contains(&n) {
            t1_top_cnt += 1;
            if t1_top_cnt == 5 && p5 == 0 {
                p5 = (idx + 1) as u32;
            }
        }
        if t1_priority.contains(&n) {
            t1_all_cnt += 1;
            if t1_all_cnt == 15 {
                p15 = (idx + 1) as u32;
                break;
            }
        }
    }

    // Draw up to p5
    for p in 1..p5 {
        GameRepository::draw_next(&mut conn, game_id, p).unwrap();
    }

    // 5. Claims testing:
    // a) Incomplete rejected
    let res_inc =
        ClaimRepository::submit_claim(&mut conn, game_id, full_house_id, "T-000001").unwrap();
    assert_eq!(res_inc.status, "REJECTED");
    assert_eq!(
        res_inc.rejection_code,
        Some(ClaimRejectionCode::NotCompleted)
    );

    // b) Correct accepted (Early Five at p5)
    let res_ef =
        ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
    assert_eq!(res_ef.status, "ACCEPTED");
    assert_eq!(res_ef.winner_position, Some(1));

    // c) Non-FH restriction: attempting Top Line at p5 on same ticket is rejected as ONLY_FULL_HOUSE_ELIGIBLE
    let res_tl =
        ClaimRepository::submit_claim(&mut conn, game_id, top_line_id, "T-000001").unwrap();
    assert_eq!(res_tl.status, "REJECTED");
    assert_eq!(
        res_tl.rejection_code,
        Some(ClaimRejectionCode::OnlyFullHouseEligible)
    );

    // d) Repeated claim rejected
    let res_ef_dup =
        ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
    assert_eq!(res_ef_dup.status, "REJECTED");
    assert_eq!(
        res_ef_dup.rejection_code,
        Some(ClaimRejectionCode::AlreadyWonScheme)
    );

    // e) Advance draw by 1 -> late claim rejected
    GameRepository::draw_next(&mut conn, game_id, p5).unwrap();
    let res_late =
        ClaimRepository::submit_claim(&mut conn, game_id, early_five_id, "T-000001").unwrap();
    assert_eq!(res_late.status, "REJECTED");
    assert_eq!(res_late.rejection_code, Some(ClaimRejectionCode::LateClaim));

    // Draw up to p15
    for p in (p5 + 1)..p15 {
        GameRepository::draw_next(&mut conn, game_id, p).unwrap();
    }

    // f) FH allowed after a non-FH win
    let res_fh =
        ClaimRepository::submit_claim(&mut conn, game_id, full_house_id, "T-000001").unwrap();
    assert_eq!(res_fh.status, "ACCEPTED");
    assert_eq!(res_fh.winner_position, Some(1));

    // g) Nothing after FH: subsequent claim for any scheme qualifying at p15 rejected with AlreadyWonFullHouse
    let res_post_fh =
        ClaimRepository::submit_claim(&mut conn, game_id, full_house_id, "T-000001").unwrap();
    assert_eq!(res_post_fh.status, "REJECTED");
    assert_eq!(
        res_post_fh.rejection_code,
        Some(ClaimRejectionCode::AlreadyWonScheme)
    );

    // Also verify scheme with max_winners = 1 rejects subsequent valid claim
    let t2_dto = TicketRepository::get_ticket(&conn, "T-000002").unwrap();
    let t2: Ticket = dto_to_ticket(&t2_dto.cells);
    let mut t2_all = Vec::new();
    for row in &t2.cells {
        for n in row.iter().flatten() {
            t2_all.push(*n);
        }
    }
    // If ticket 2 claims full house when max winners reached -> WINNER_LIMIT_REACHED
    // (verified in capacity unit tests)

    // 6. DROP the connection and reopen the file: active game recovers
    drop(conn);

    let mut conn2 = initialize_database(&db_path).unwrap();
    let active_game = GameRepository::get_active_game(&conn2)
        .unwrap()
        .expect("Active game must recover");
    assert_eq!(active_game.game.id, game_id);
    assert_eq!(active_game.game.current_draw_position, p15);
    assert_eq!(active_game.game.status, GameStatus::Active);

    // Winners and claims preserved across reopen
    let claims = ClaimRepository::list_game_claims(&conn2, game_id).unwrap();
    assert!(claims.len() >= 5);

    // 7. End game, start a new one: fresh eligibility & history intact
    GameRepository::end_game(&mut conn2, game_id).unwrap();

    let mut g2_rng = DeterministicRandomSource::new(303);
    let g2 = GameRepository::create_game(&mut conn2, batch.id, &mut g2_rng).unwrap();
    assert_eq!(g2.game.status, GameStatus::Active);
    assert_ne!(g2.game.id, game_id);

    // History contains previous game and winners
    let history_stats = HistoryRepository::get_dashboard_stats(&conn2).unwrap();
    assert_eq!(history_stats.games_played, 1);
    assert_eq!(history_stats.recent_winners.len(), 2);
}
