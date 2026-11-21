use media_tambola_lib::domain::claims::claim_validation::{
    evaluate_claim, ClaimDecision, GameSchemeSnapshot,
};
use media_tambola_lib::domain::game::GameStatus;
use media_tambola_lib::domain::schemes::{
    LogicalTicketCells, PatternKind, PatternMask, SchemeType,
};
use media_tambola_lib::domain::win_forecast::{
    calculate_game_win_forecast, GameWinForecastDto, TicketForecastCandidate,
};
use std::collections::HashMap;

#[test]
fn test_win_forecast_crosscheck_with_claim_validation() {
    let ticket_1_cells: LogicalTicketCells =
        [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];

    let ticket_2_cells: LogicalTicketCells = [
        [16, 17, 18, 19, 20],
        [21, 22, 23, 24, 25],
        [26, 27, 28, 29, 30],
    ];

    let tickets: Vec<TicketForecastCandidate> = vec![
        TicketForecastCandidate {
            ticket_id: 1,
            ticket_code: "1".to_string(),
            cells: ticket_1_cells,
        },
        TicketForecastCandidate {
            ticket_id: 2,
            ticket_code: "2".to_string(),
            cells: ticket_2_cells,
        },
    ];

    let top_line_mask: PatternMask = [
        [true, true, true, true, true],
        [false, false, false, false, false],
        [false, false, false, false, false],
    ];

    let scheme_top_line: GameSchemeSnapshot = GameSchemeSnapshot {
        scheme_id: 1,
        scheme_name: "Top Line".to_string(),
        scheme_type: SchemeType::TopLine,
        pattern_kind: PatternKind::Mask,
        mask: top_line_mask,
        early_n: None,
        is_full_house: false,
        max_winners: 1,
    };

    let full_house_mask: PatternMask = [
        [true, true, true, true, true],
        [true, true, true, true, true],
        [true, true, true, true, true],
    ];

    let scheme_full_house: GameSchemeSnapshot = GameSchemeSnapshot {
        scheme_id: 2,
        scheme_name: "Full House".to_string(),
        scheme_type: SchemeType::FullHouse,
        pattern_kind: PatternKind::Mask,
        mask: full_house_mask,
        early_n: None,
        is_full_house: true,
        max_winners: 1,
    };

    let schemes: Vec<GameSchemeSnapshot> = vec![scheme_top_line.clone(), scheme_full_house.clone()];

    // Draw sequence:
    // First 5 numbers: 1, 2, 3, 4, 5 -> ticket 1 completes Top Line at draw 5.
    // Next 10 numbers: 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 -> ticket 1 completes Full House at draw 15.
    // Ticket 2 never completes anything.
    let draw_sequence: Vec<u8> = vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30,
    ];

    let mut song_titles: HashMap<u8, String> = HashMap::new();
    song_titles.insert(5, "Song 5".to_string());
    song_titles.insert(15, "Song 15".to_string());

    let forecast: GameWinForecastDto = calculate_game_win_forecast(
        100,
        "G-TEST-001",
        "SONG_GRID",
        &draw_sequence,
        &schemes,
        &tickets,
        &song_titles,
    );

    assert_eq!(forecast.schemes.len(), 2);

    // Verify Top Line
    let top_line_forecast = &forecast.schemes[0];
    assert_eq!(top_line_forecast.scheme_name, "Top Line");
    assert_eq!(top_line_forecast.winners.len(), 1);
    let top_winner = &top_line_forecast.winners[0];
    assert_eq!(top_winner.ticket_code, "1");
    assert_eq!(top_winner.draw_position, 5);
    assert_eq!(top_winner.drawn_number, 5);
    assert_eq!(top_winner.song_title, Some("Song 5".to_string()));

    // Verify Full House
    let fh_forecast = &forecast.schemes[1];
    assert_eq!(fh_forecast.scheme_name, "Full House");
    assert_eq!(fh_forecast.winners.len(), 1);
    let fh_winner = &fh_forecast.winners[0];
    assert_eq!(fh_winner.ticket_code, "1");
    assert_eq!(fh_winner.draw_position, 15);
    assert_eq!(fh_winner.drawn_number, 15);
    assert_eq!(fh_winner.song_title, Some("Song 15".to_string()));

    // CROSS-CHECK with evaluate_claim (submit_claim logic)
    // 1. At draw position 5:
    let mut call_map: HashMap<u8, u32> = HashMap::new();
    for (idx, &num) in draw_sequence[0..5].iter().enumerate() {
        call_map.insert(num, (idx + 1) as u32);
    }

    let decision_top = evaluate_claim(
        Some(GameStatus::Active),
        5,
        Some(&ticket_1_cells),
        true,
        Some(&scheme_top_line),
        &call_map,
        &[],
        0,
    );
    match decision_top {
        ClaimDecision::Accept {
            qualification_position,
            winner_position,
        } => {
            assert_eq!(qualification_position, top_winner.draw_position);
            assert_eq!(winner_position, top_winner.winner_position);
        }
        _ => panic!("Expected top line claim to be accepted"),
    }

    // 2. At draw position 15:
    for (idx, &num) in draw_sequence[5..15].iter().enumerate() {
        call_map.insert(num, (idx + 6) as u32);
    }

    let decision_fh = evaluate_claim(
        Some(GameStatus::Active),
        15,
        Some(&ticket_1_cells),
        true,
        Some(&scheme_full_house),
        &call_map,
        &[],
        0,
    );
    match decision_fh {
        ClaimDecision::Accept {
            qualification_position,
            winner_position,
        } => {
            assert_eq!(qualification_position, fh_winner.draw_position);
            assert_eq!(winner_position, fh_winner.winner_position);
        }
        _ => panic!("Expected full house claim to be accepted"),
    }
}
