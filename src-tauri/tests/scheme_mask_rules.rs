use media_tambola_lib::domain::schemes::{
    GenericPatternEvaluator, LogicalTicketCells, PatternKind, PatternMask,
};
use std::collections::HashMap;

#[test]
fn test_scheme_mask_evaluation_rules() {
    // 3x5 logical ticket cells
    let ticket_cells: LogicalTicketCells =
        [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];

    // Scenario 1: Top Line Mask
    let top_line_mask: PatternMask = [
        [true, true, true, true, true],
        [false, false, false, false, false],
        [false, false, false, false, false],
    ];

    // Numbers 1, 2, 3, 4 drawn, 5 NOT drawn
    let mut call_map: HashMap<u8, u32> = HashMap::new();
    call_map.insert(1, 1);
    call_map.insert(2, 2);
    call_map.insert(3, 3);
    call_map.insert(4, 4);

    let res_incomplete = GenericPatternEvaluator::evaluate(
        &ticket_cells,
        PatternKind::Mask,
        &top_line_mask,
        None,
        &call_map,
    );
    assert!(
        !res_incomplete.is_qualified,
        "Top line should not be qualified with 4 of 5 numbers"
    );

    // Draw 5 at position 5
    call_map.insert(5, 5);
    let res_complete = GenericPatternEvaluator::evaluate(
        &ticket_cells,
        PatternKind::Mask,
        &top_line_mask,
        None,
        &call_map,
    );
    assert!(res_complete.is_qualified, "Top line must be qualified");
    assert_eq!(res_complete.qualification_position, Some(5));

    // Scenario 2: Early 5
    let empty_mask: PatternMask = [[false; 5]; 3];
    let mut call_map_early: HashMap<u8, u32> = HashMap::new();
    call_map_early.insert(1, 1);
    call_map_early.insert(6, 2);
    call_map_early.insert(11, 3);
    call_map_early.insert(7, 4);

    let res_early4 = GenericPatternEvaluator::evaluate(
        &ticket_cells,
        PatternKind::EarlyN,
        &empty_mask,
        Some(5),
        &call_map_early,
    );
    assert!(
        !res_early4.is_qualified,
        "Early 5 must reject 4 matched numbers"
    );

    call_map_early.insert(15, 9);
    let res_early5 = GenericPatternEvaluator::evaluate(
        &ticket_cells,
        PatternKind::EarlyN,
        &empty_mask,
        Some(5),
        &call_map_early,
    );
    assert!(
        res_early5.is_qualified,
        "Early 5 must accept 5 matched numbers"
    );
    assert_eq!(res_early5.qualification_position, Some(9));

    // Scenario 3: Four Corners Mask
    let four_corners_mask: PatternMask = [
        [true, false, false, false, true],
        [false, false, false, false, false],
        [true, false, false, false, true],
    ];

    let mut call_map_corners: HashMap<u8, u32> = HashMap::new();
    call_map_corners.insert(1, 10);
    call_map_corners.insert(5, 12);
    call_map_corners.insert(11, 14);
    call_map_corners.insert(15, 20);

    let res_corners = GenericPatternEvaluator::evaluate(
        &ticket_cells,
        PatternKind::Mask,
        &four_corners_mask,
        None,
        &call_map_corners,
    );
    assert!(
        res_corners.is_qualified,
        "Four corners must qualify when cells (0,0), (0,4), (2,0), (2,4) are drawn"
    );
    assert_eq!(res_corners.qualification_position, Some(20));
}
