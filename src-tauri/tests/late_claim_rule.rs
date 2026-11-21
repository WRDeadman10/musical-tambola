use media_tambola_lib::domain::claims::claim_validation::{
    evaluate_claim, ClaimDecision, ClaimRejectionCode, GameSchemeSnapshot,
};
use media_tambola_lib::domain::game::GameStatus;
use media_tambola_lib::domain::schemes::{
    LogicalTicketCells, PatternKind, PatternMask, SchemeType,
};
use std::collections::HashMap;

#[test]
fn test_late_claim_is_strictly_rejected() {
    let cells: LogicalTicketCells = [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];

    let top_line_mask: PatternMask = [
        [true, true, true, true, true],
        [false, false, false, false, false],
        [false, false, false, false, false],
    ];

    let scheme_snapshot: GameSchemeSnapshot = GameSchemeSnapshot {
        scheme_id: 101,
        scheme_name: "Top Line".to_string(),
        scheme_type: SchemeType::TopLine,
        pattern_kind: PatternKind::Mask,
        mask: top_line_mask,
        early_n: None,
        is_full_house: false,
        max_winners: 1,
    };

    // Numbers 1, 2, 3, 4, 5 were drawn at positions 1, 2, 3, 4, 5
    let mut call_map: HashMap<u8, u32> = HashMap::new();
    call_map.insert(1, 1);
    call_map.insert(2, 2);
    call_map.insert(3, 3);
    call_map.insert(4, 4);
    call_map.insert(5, 5);

    // Current draw position is 5 (exact match -> should ACCEPT)
    let decision_timely = evaluate_claim(
        Some(GameStatus::Active),
        5,
        Some(&cells),
        true,
        Some(&scheme_snapshot),
        &call_map,
        &[],
        0,
    );
    match decision_timely {
        ClaimDecision::Accept {
            qualification_position,
            winner_position,
        } => {
            assert_eq!(qualification_position, 5);
            assert_eq!(winner_position, 1);
        }
        ClaimDecision::Reject { code, .. } => {
            panic!(
                "Expected timely claim to be accepted, rejected with {:?}",
                code
            );
        }
    }

    // Now advance current draw position to 6 (subsequent number drawn)
    let decision_late = evaluate_claim(
        Some(GameStatus::Active),
        6,
        Some(&cells),
        true,
        Some(&scheme_snapshot),
        &call_map,
        &[],
        0,
    );
    match decision_late {
        ClaimDecision::Reject {
            code,
            qualification_position,
        } => {
            assert_eq!(code, ClaimRejectionCode::LateClaim);
            assert_eq!(qualification_position, Some(5));
        }
        ClaimDecision::Accept { .. } => {
            panic!("Expected late claim at position 6 to be rejected");
        }
    }
}
