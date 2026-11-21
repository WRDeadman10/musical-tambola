use media_tambola_lib::db::connection::initialize_database;
use media_tambola_lib::db::repositories::claim::ClaimRepository;
use media_tambola_lib::db::repositories::game::GameRepository;
use media_tambola_lib::db::repositories::history::HistoryRepository;
use media_tambola_lib::db::repositories::lucky_draw::LuckyDrawRepository;
use media_tambola_lib::db::repositories::media::MediaRepository;
use media_tambola_lib::db::repositories::scheme::SchemeRepository;
use media_tambola_lib::db::repositories::ticket::TicketRepository;
use media_tambola_lib::domain::claims::claim_validation::ClaimRejectionCode;
use media_tambola_lib::domain::format::GameFormat;
use media_tambola_lib::domain::game::GameStatus;
use media_tambola_lib::domain::media::{
    MediaAssignment, MediaFolderOrder, MediaType, SaveMediaInput, SourceType, ValidationStatus,
};
use media_tambola_lib::domain::random::DeterministicRandomSource;
use media_tambola_lib::domain::schemes::{PatternKind, PatternMask, SaveSchemeInput, SchemeType};
use media_tambola_lib::domain::tambola::ticket_generator::Ticket;
use media_tambola_lib::services::backup::{
    create_backup_archive_in_dir, restore_backup_archive_in_dir,
};
use media_tambola_lib::services::media_folder::scan_media_folder;
use media_tambola_lib::services::media_validation::validate_slot;
use media_tambola_lib::services::title_import::{
    apply_song_title_import_impl, preview_song_title_import_impl, ApplyTitleImportItemDto,
};
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
        let (status, err, code, params_json) = validate_slot(&slot).await;
        assert_eq!(
            status,
            ValidationStatus::Valid,
            "Slot {} must validate successfully",
            num
        );
        MediaRepository::update_validation_status(
            &conn,
            num,
            status,
            err.as_deref(),
            code.as_deref(),
            params_json.as_deref(),
        )
        .unwrap();
    }

    let all_slots = MediaRepository::list_all(&conn).unwrap();
    let valid_count = all_slots
        .iter()
        .filter(|s| s.validation_status == ValidationStatus::Valid)
        .count();
    assert_eq!(valid_count, 90, "All 90 media items must be validated");

    // 3. Generate 100 tickets and assert unique codes and hashes
    let mut rng = DeterministicRandomSource::new(101);
    let batch = TicketRepository::generate_batch(
        &mut conn,
        "Acceptance Batch",
        100,
        None,
        None,
        &mut rng,
        |_, _| {},
    )
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
    let t1_code: String = tickets_page.items[0].code.clone();
    let t1_dto = TicketRepository::get_ticket(&conn, &t1_code).unwrap();
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
        if ticket.code == t1_code {
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

#[tokio::test]
async fn test_song_grid_v1_0_1_end_to_end_acceptance_workflow() {
    let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    let app_data_dir: PathBuf = temp_dir.path().join("appdata");
    let db_path: PathBuf = app_data_dir.join("data").join("tambola.sqlite");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let mut conn = initialize_database(&db_path).unwrap();

    // 1. Configure active format to SONG_GRID with 45 songs
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('active_game_format', 'SONG_GRID')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('active_song_count', '45')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
        [],
    )
    .unwrap();

    // 2. Folder-assign media from temp directory
    let media_dir: PathBuf = temp_dir.path().join("media_scan");
    fs::create_dir_all(&media_dir).unwrap();
    for num in 1..=45 {
        let fpath: PathBuf = media_dir.join(format!("{:02}_track.jpg", num));
        let mut f = fs::File::create(&fpath).unwrap();
        f.write_all(&[
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        ])
        .unwrap();
    }
    let scan_res = scan_media_folder(&media_dir, MediaFolderOrder::Asc, false, None, 45).unwrap();
    assert_eq!(scan_res.assignments.len(), 45);

    let assignments: Vec<MediaAssignment> = scan_res
        .assignments
        .into_iter()
        .map(|pair| MediaAssignment {
            number: pair.number,
            path: pair.path,
            media_type: MediaType::Image,
            title: None,
        })
        .collect();
    let slots = MediaRepository::apply_assignments(&mut conn, &assignments).unwrap();
    assert_eq!(slots.len(), 90);

    // 3. Apply song title import (CSV)
    let mut csv_data: String = String::from("Number,Title\n");
    for i in 1..=45 {
        csv_data.push_str(&format!("{},Song Title {}\n", i, i));
    }
    let preview = preview_song_title_import_impl(csv_data.as_bytes(), "titles.csv", &conn).unwrap();
    assert_eq!(preview.valid_count, 45);
    assert_eq!(preview.out_of_format_range.len(), 0);

    let apply_items: Vec<ApplyTitleImportItemDto> = preview
        .rows
        .into_iter()
        .filter(|r| r.is_valid)
        .map(|r| ApplyTitleImportItemDto {
            number: r.number,
            title: r.new_title,
        })
        .collect();
    let applied: u32 = apply_song_title_import_impl(&apply_items, &mut conn).unwrap();
    assert_eq!(applied, 45);

    // 4. Generate tickets in SONG_GRID format
    let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(4545);
    let batch = TicketRepository::generate_batch(
        &mut conn,
        "Acceptance Song Grid Batch",
        5,
        Some(GameFormat::SongGrid),
        Some(45),
        &mut rng,
        |_, _| {},
    )
    .unwrap();
    assert_eq!(batch.generated_count, 5);
    let tickets_page = TicketRepository::list_batch_tickets(&conn, batch.id, 0, 5).unwrap();
    assert_eq!(tickets_page.items.len(), 5);

    // 5. Seed / save schemes: Pattern H, Photo Frame, and 2 Full House tiers
    conn.execute("DELETE FROM prize_schemes;", []).unwrap();
    let h_mask: PatternMask = [
        [true, false, false, false, true],
        [true, true, true, true, true],
        [true, false, false, false, true],
    ];
    let photo_mask: PatternMask = [
        [true, true, true, true, true],
        [true, false, false, false, true],
        [true, true, true, true, true],
    ];
    let scheme_h = SchemeRepository::save(
        &conn,
        &SaveSchemeInput {
            id: None,
            name: "Pattern H".to_string(),
            description: "H pattern".to_string(),
            scheme_type: None,
            pattern_kind: Some(PatternKind::Mask),
            mask: Some(h_mask),
            early_n: None,
            is_full_house: Some(false),
            max_winners: 1,
            is_active: true,
        },
    )
    .unwrap();

    let scheme_photo = SchemeRepository::save(
        &conn,
        &SaveSchemeInput {
            id: None,
            name: "Photo Frame".to_string(),
            description: "Photo frame pattern".to_string(),
            scheme_type: None,
            pattern_kind: Some(PatternKind::Mask),
            mask: Some(photo_mask),
            early_n: None,
            is_full_house: Some(false),
            max_winners: 1,
            is_active: true,
        },
    )
    .unwrap();

    let scheme_fh1 = SchemeRepository::save(
        &conn,
        &SaveSchemeInput {
            id: None,
            name: "Full House 1st".to_string(),
            description: "First Full House tier".to_string(),
            scheme_type: Some(SchemeType::FullHouse),
            pattern_kind: Some(PatternKind::FullHouse),
            mask: Some([[true; 5]; 3]),
            early_n: None,
            is_full_house: Some(true),
            max_winners: 1,
            is_active: true,
        },
    )
    .unwrap();

    let scheme_fh2 = SchemeRepository::save(
        &conn,
        &SaveSchemeInput {
            id: None,
            name: "Full House 2nd".to_string(),
            description: "Second Full House tier".to_string(),
            scheme_type: Some(SchemeType::FullHouse),
            pattern_kind: Some(PatternKind::FullHouse),
            mask: Some([[true; 5]; 3]),
            early_n: None,
            is_full_house: Some(true),
            max_winners: 1,
            is_active: true,
        },
    )
    .unwrap();

    // 6. Create game session
    let game = GameRepository::create_game(&mut conn, batch.id, &mut rng).unwrap();
    let game_id: i64 = game.game.id;
    let first_draw: u8 = game.current.unwrap().number;

    // 7. Find two tickets that share at least one Pattern H number
    let mut t1_idx: usize = 0;
    let mut t2_idx: usize = 1;
    let mut shared_h_num: u8 = 0;

    'outer: for (i, item_i) in tickets_page.items.iter().enumerate() {
        let ti = TicketRepository::get_ticket(&conn, &item_i.code).unwrap();
        let ti_h: HashSet<u8> = ti
            .cells
            .iter()
            .filter(|c| (c.row == 1) || ((c.row == 0 || c.row == 2) && (c.col == 0 || c.col == 4)))
            .map(|c| c.number)
            .collect();

        for (j, item_j) in tickets_page.items.iter().enumerate().skip(i + 1) {
            let tj = TicketRepository::get_ticket(&conn, &item_j.code).unwrap();
            let tj_h: HashSet<u8> = tj
                .cells
                .iter()
                .filter(|c| {
                    (c.row == 1) || ((c.row == 0 || c.row == 2) && (c.col == 0 || c.col == 4))
                })
                .map(|c| c.number)
                .collect();

            if let Some(&num) = ti_h.intersection(&tj_h).next() {
                t1_idx = i;
                t2_idx = j;
                shared_h_num = num;
                break 'outer;
            }
        }
    }
    assert!(
        shared_h_num > 0,
        "Must find two tickets sharing a Pattern H number"
    );

    let t1_code: String = tickets_page.items[t1_idx].code.clone();
    let t2_code: String = tickets_page.items[t2_idx].code.clone();
    let t1_dto = TicketRepository::get_ticket(&conn, &t1_code).unwrap();
    let t2_dto = TicketRepository::get_ticket(&conn, &t2_code).unwrap();

    let t1_h_nums: Vec<u8> = t1_dto
        .cells
        .iter()
        .filter(|c| (c.row == 1) || ((c.row == 0 || c.row == 2) && (c.col == 0 || c.col == 4)))
        .map(|c| c.number)
        .collect();
    assert_eq!(t1_h_nums.len(), 9);

    let t2_h_nums: Vec<u8> = t2_dto
        .cells
        .iter()
        .filter(|c| (c.row == 1) || ((c.row == 0 || c.row == 2) && (c.col == 0 || c.col == 4)))
        .map(|c| c.number)
        .collect();
    assert_eq!(t2_h_nums.len(), 9);

    // Photo frame cells on Ticket 1 (12 cells)
    let t1_photo_nums: Vec<u8> = t1_dto
        .cells
        .iter()
        .filter(|c| c.row == 0 || c.row == 2 || c.col == 0 || c.col == 4)
        .map(|c| c.number)
        .collect();
    assert_eq!(t1_photo_nums.len(), 12);

    let t1_all_nums: Vec<u8> = t1_dto.cells.iter().map(|c| c.number).collect();
    assert_eq!(t1_all_nums.len(), 15);

    // Build deterministic sequence
    let mut seq: Vec<u8> = Vec::with_capacity(45);
    let mut used: [bool; 46] = [false; 46];
    seq.push(first_draw);
    used[first_draw as usize] = true;

    // Call all T1 H numbers except shared_h_num
    for &n in &t1_h_nums {
        if n != shared_h_num && !used[n as usize] {
            seq.push(n);
            used[n as usize] = true;
        }
    }

    // Call all T2 H numbers except shared_h_num
    for &n in &t2_h_nums {
        if n != shared_h_num && !used[n as usize] {
            seq.push(n);
            used[n as usize] = true;
        }
    }

    // Now call shared_h_num: completes Pattern H for BOTH T1 and T2 at this exact position!
    if !used[shared_h_num as usize] {
        seq.push(shared_h_num);
        used[shared_h_num as usize] = true;
    }
    let h_complete_pos: u32 = seq.len() as u32;

    // Call remaining Photo Frame numbers of Ticket 1
    for &n in &t1_photo_nums {
        if !used[n as usize] {
            seq.push(n);
            used[n as usize] = true;
        }
    }
    let photo_complete_pos: u32 = seq.len() as u32;

    // Call remaining numbers of Ticket 1 for Full House
    for &n in &t1_all_nums {
        if !used[n as usize] {
            seq.push(n);
            used[n as usize] = true;
        }
    }
    let fh_complete_pos: u32 = seq.len() as u32;

    // Fill the rest up to 45
    for n in 1..=45 {
        if !used[n as usize] {
            seq.push(n);
            used[n as usize] = true;
        }
    }
    assert_eq!(seq.len(), 45);

    conn.execute(
        "UPDATE games SET draw_sequence_json = ?1 WHERE id = ?2;",
        params![serde_json::to_string(&seq).unwrap(), game_id],
    )
    .unwrap();

    // Advance draws to h_complete_pos
    for p in 1..h_complete_pos {
        GameRepository::draw_next(&mut conn, game_id, p).unwrap();
    }

    // 8. Claims testing:
    // (a) Ticket 1 claims Pattern H at h_complete_pos -> ACCEPTED
    let claim_h = ClaimRepository::submit_claim(&mut conn, game_id, scheme_h.id, &t1_code).unwrap();
    assert_eq!(claim_h.status, "ACCEPTED");
    assert_eq!(claim_h.winner_position, Some(1));
    assert_eq!(claim_h.qualification_position, Some(h_complete_pos));

    // (b) Winner limit reached: Ticket 2 also qualifies at h_complete_pos, but max_winners 1 reached -> REJECTED (WinnerLimitReached)
    let claim_h_limit =
        ClaimRepository::submit_claim(&mut conn, game_id, scheme_h.id, &t2_code).unwrap();
    assert_eq!(claim_h_limit.status, "REJECTED");
    assert_eq!(
        claim_h_limit.rejection_code,
        Some(ClaimRejectionCode::WinnerLimitReached)
    );

    // (c) Not completed: Ticket 1 claims Full House before completion -> REJECTED (NotCompleted)
    let claim_fh_early =
        ClaimRepository::submit_claim(&mut conn, game_id, scheme_fh1.id, &t1_code).unwrap();
    assert_eq!(claim_fh_early.status, "REJECTED");
    assert_eq!(
        claim_fh_early.rejection_code,
        Some(ClaimRejectionCode::NotCompleted)
    );

    // Now advance ONE MORE draw past h_complete_pos
    GameRepository::draw_next(&mut conn, game_id, h_complete_pos).unwrap();

    // (d) Late claim: Ticket 2 attempts claim at h_complete_pos + 1 (qualified at h_complete_pos) -> REJECTED (LateClaim)
    let claim_late =
        ClaimRepository::submit_claim(&mut conn, game_id, scheme_h.id, &t2_code).unwrap();
    assert_eq!(claim_late.status, "REJECTED");
    assert_eq!(
        claim_late.rejection_code,
        Some(ClaimRejectionCode::LateClaim)
    );

    // Advance draws to photo_complete_pos where Ticket 1 completes Photo Frame
    for p in (h_complete_pos + 1)..photo_complete_pos {
        GameRepository::draw_next(&mut conn, game_id, p).unwrap();
    }

    // (e) OnlyFullHouseEligible: Ticket 1 completed Photo Frame, but already won Pattern H -> REJECTED (OnlyFullHouseEligible)
    let claim_photo_only_fh =
        ClaimRepository::submit_claim(&mut conn, game_id, scheme_photo.id, &t1_code).unwrap();
    assert_eq!(claim_photo_only_fh.status, "REJECTED");
    assert_eq!(
        claim_photo_only_fh.rejection_code,
        Some(ClaimRejectionCode::OnlyFullHouseEligible)
    );

    // Advance draws to fh_complete_pos
    for p in photo_complete_pos..fh_complete_pos {
        GameRepository::draw_next(&mut conn, game_id, p).unwrap();
    }

    // (f) Full House 1st tier: Ticket 1 claims at fh_complete_pos -> ACCEPTED
    let claim_fh =
        ClaimRepository::submit_claim(&mut conn, game_id, scheme_fh1.id, &t1_code).unwrap();
    assert_eq!(claim_fh.status, "ACCEPTED");
    assert_eq!(claim_fh.winner_position, Some(1));
    assert_eq!(claim_fh.qualification_position, Some(fh_complete_pos));

    // (g) AlreadyWonFullHouse: Ticket 1 tries to claim Full House 2nd tier -> REJECTED (AlreadyWonFullHouse)
    let claim_fh2 =
        ClaimRepository::submit_claim(&mut conn, game_id, scheme_fh2.id, &t1_code).unwrap();
    assert_eq!(claim_fh2.status, "REJECTED");
    assert_eq!(
        claim_fh2.rejection_code,
        Some(ClaimRejectionCode::AlreadyWonFullHouse)
    );

    // 9. Lucky Draw execution
    let candidate_names: Vec<String> = vec!["Guest A".to_string(), "Guest B".to_string()];
    let ld = LuckyDrawRepository::create_draw(
        &mut conn,
        Some("Song Grid Lucky Draw"),
        "NAMES",
        &candidate_names,
        None,
        false,
    )
    .unwrap();
    let mut ld_rng: DeterministicRandomSource = DeterministicRandomSource::new(888);
    let ld_winner = LuckyDrawRepository::draw_winner(&mut conn, ld.id, &mut ld_rng).unwrap();
    assert!(["Guest A", "Guest B"].contains(&ld_winner.value.as_str()));

    // 10. Crash simulation / database reopen
    drop(conn);
    let mut conn2 = initialize_database(&db_path).unwrap();
    let active_game = GameRepository::get_active_game(&conn2)
        .unwrap()
        .expect("Game must recover after reopen");
    assert_eq!(active_game.game.id, game_id);
    assert_eq!(active_game.game.status, GameStatus::Active);

    // 11. Backup -> Staged Restore -> Apply -> Verification
    let backup_zip_path: PathBuf = temp_dir.path().join("backup.zip");
    create_backup_archive_in_dir(
        Some(backup_zip_path.to_string_lossy().to_string()),
        &app_data_dir,
        &conn2,
    )
    .unwrap();
    assert!(backup_zip_path.exists());

    // End active game before restore
    GameRepository::end_game(&mut conn2, game_id).unwrap();

    let restore_dto =
        restore_backup_archive_in_dir(&backup_zip_path.to_string_lossy(), &app_data_dir, &conn2)
            .unwrap();
    assert!(restore_dto.restart_required);

    drop(conn2);

    // Startup applies staged restore!
    let conn3 = initialize_database(&db_path).unwrap();
    let games = GameRepository::list_games(&conn3, 10).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].id, game_id);
    let claims = ClaimRepository::list_game_claims(&conn3, game_id).unwrap();
    assert_eq!(claims.len(), 7);
}
