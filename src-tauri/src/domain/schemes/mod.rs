use crate::domain::tambola::ticket_generator::{SongGridTicket, Ticket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PatternKind {
    Mask,
    EarlyN,
    FullHouse,
}

impl PatternKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mask => "MASK",
            Self::EarlyN => "EARLY_N",
            Self::FullHouse => "FULL_HOUSE",
        }
    }
}

impl FromStr for PatternKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "MASK" => Ok(Self::Mask),
            "EARLY_N" => Ok(Self::EarlyN),
            "FULL_HOUSE" => Ok(Self::FullHouse),
            other => Err(format!("Unknown pattern kind: '{}'", other)),
        }
    }
}

pub type PatternMask = [[bool; 5]; 3];
pub type LogicalTicketCells = [[u8; 5]; 3];

pub fn ticket_to_logical_cells(ticket: &Ticket) -> LogicalTicketCells {
    let mut logical: LogicalTicketCells = [[0; 5]; 3];
    for (r, row) in ticket.cells.iter().enumerate() {
        let mut c: usize = 0;
        for cell_opt in row {
            if let Some(n) = *cell_opt {
                if c < 5 {
                    logical[r][c] = n;
                    c += 1;
                }
            }
        }
    }
    logical
}

pub fn song_grid_to_logical_cells(ticket: &SongGridTicket) -> LogicalTicketCells {
    ticket.cells
}

pub struct GenericPatternEvaluator;

impl GenericPatternEvaluator {
    pub fn evaluate(
        cells: &LogicalTicketCells,
        pattern_kind: PatternKind,
        mask: &PatternMask,
        early_n: Option<u32>,
        call_position_by_number: &HashMap<u8, u32>,
    ) -> QualificationResult {
        match pattern_kind {
            PatternKind::Mask => {
                let mut max_pos: u32 = 0;
                let mut masked_count: usize = 0;

                for (r, row) in mask.iter().enumerate() {
                    for (c, &is_masked) in row.iter().enumerate() {
                        if is_masked {
                            masked_count += 1;
                            let num: u8 = cells[r][c];
                            match call_position_by_number.get(&num) {
                                Some(&pos) => {
                                    if pos > max_pos {
                                        max_pos = pos;
                                    }
                                }
                                None => return QualificationResult::not_qualified(),
                            }
                        }
                    }
                }

                if masked_count == 0 {
                    return QualificationResult::not_qualified();
                }

                QualificationResult::qualified(max_pos)
            }
            PatternKind::EarlyN => {
                let n: usize = early_n.unwrap_or(5) as usize;
                if n == 0 || n > 15 {
                    return QualificationResult::not_qualified();
                }

                let mut called_positions: Vec<u32> = Vec::with_capacity(15);
                for row in cells {
                    for &num in row {
                        if num > 0 {
                            if let Some(&pos) = call_position_by_number.get(&num) {
                                called_positions.push(pos);
                            }
                        }
                    }
                }

                if called_positions.len() < n {
                    return QualificationResult::not_qualified();
                }

                called_positions.sort_unstable();
                QualificationResult::qualified(called_positions[n - 1])
            }
            PatternKind::FullHouse => {
                let mut max_pos: u32 = 0;
                for row in cells {
                    for &num in row {
                        if num == 0 {
                            return QualificationResult::not_qualified();
                        }
                        match call_position_by_number.get(&num) {
                            Some(&pos) => {
                                if pos > max_pos {
                                    max_pos = pos;
                                }
                            }
                            None => return QualificationResult::not_qualified(),
                        }
                    }
                }

                QualificationResult::qualified(max_pos)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemeType {
    EarlyFive,
    FourCorners,
    TopLine,
    MiddleLine,
    BottomLine,
    FullHouse,
}

impl SchemeType {
    pub fn is_full_house(&self) -> bool {
        matches!(self, Self::FullHouse)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EarlyFive => "EARLY_FIVE",
            Self::FourCorners => "FOUR_CORNERS",
            Self::TopLine => "TOP_LINE",
            Self::MiddleLine => "MIDDLE_LINE",
            Self::BottomLine => "BOTTOM_LINE",
            Self::FullHouse => "FULL_HOUSE",
        }
    }

    pub fn rule_description(&self) -> &'static str {
        match self {
            Self::EarlyFive => "Any 5 numbers on the ticket are struck/called.",
            Self::FourCorners => {
                "The leftmost and rightmost occupied cells of the top row (Row 0) and bottom row (Row 2)."
            }
            Self::TopLine => "All 5 numbers in Row 0 are struck.",
            Self::MiddleLine => "All 5 numbers in Row 1 are struck.",
            Self::BottomLine => "All 5 numbers in Row 2 are struck.",
            Self::FullHouse => "All 15 numbers on the ticket are struck.",
        }
    }
}

impl FromStr for SchemeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "EARLY_FIVE" => Ok(Self::EarlyFive),
            "FOUR_CORNERS" => Ok(Self::FourCorners),
            "TOP_LINE" => Ok(Self::TopLine),
            "MIDDLE_LINE" => Ok(Self::MiddleLine),
            "BOTTOM_LINE" => Ok(Self::BottomLine),
            "FULL_HOUSE" => Ok(Self::FullHouse),
            other => Err(format!("Unknown scheme type: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationResult {
    pub is_qualified: bool,
    pub qualification_position: Option<u32>,
}

impl QualificationResult {
    pub fn not_qualified() -> Self {
        Self {
            is_qualified: false,
            qualification_position: None,
        }
    }

    pub fn qualified(position: u32) -> Self {
        Self {
            is_qualified: true,
            qualification_position: Some(position),
        }
    }
}

pub trait SchemeEvaluator: Send + Sync {
    fn scheme_type(&self) -> SchemeType;
    fn required_numbers(&self, ticket: &Ticket) -> Vec<u8>;
    fn evaluate(
        &self,
        ticket: &Ticket,
        call_position_by_number: &HashMap<u8, u32>,
    ) -> QualificationResult;
}

/// Evaluator for standard Line winning schemes (TopLine, MiddleLine, BottomLine).
/// Qualification position = MAX(call_position_by_number[n]) for n in that row's 5 numbers.
pub struct LineEvaluator {
    row_idx: usize,
    scheme_type: SchemeType,
}

impl LineEvaluator {
    pub fn top_line() -> Self {
        Self {
            row_idx: 0,
            scheme_type: SchemeType::TopLine,
        }
    }

    pub fn middle_line() -> Self {
        Self {
            row_idx: 1,
            scheme_type: SchemeType::MiddleLine,
        }
    }

    pub fn bottom_line() -> Self {
        Self {
            row_idx: 2,
            scheme_type: SchemeType::BottomLine,
        }
    }
}

impl SchemeEvaluator for LineEvaluator {
    fn scheme_type(&self) -> SchemeType {
        self.scheme_type
    }

    fn required_numbers(&self, ticket: &Ticket) -> Vec<u8> {
        let mut numbers: Vec<u8> = Vec::with_capacity(5);
        for cell_opt in &ticket.cells[self.row_idx] {
            if let Some(n) = *cell_opt {
                numbers.push(n);
            }
        }
        numbers
    }

    fn evaluate(
        &self,
        ticket: &Ticket,
        call_position_by_number: &HashMap<u8, u32>,
    ) -> QualificationResult {
        let req: Vec<u8> = self.required_numbers(ticket);
        let mut max_pos: u32 = 0;

        for n in req {
            match call_position_by_number.get(&n) {
                Some(&pos) => {
                    if pos > max_pos {
                        max_pos = pos;
                    }
                }
                None => return QualificationResult::not_qualified(),
            }
        }

        QualificationResult::qualified(max_pos)
    }
}

/// Evaluator for Full House (all 15 ticket numbers).
/// Qualification position = MAX(call_position_by_number[n]) across all 15 numbers.
pub struct FullHouseEvaluator;

impl SchemeEvaluator for FullHouseEvaluator {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::FullHouse
    }

    fn required_numbers(&self, ticket: &Ticket) -> Vec<u8> {
        let mut numbers: Vec<u8> = Vec::with_capacity(15);
        for row in &ticket.cells {
            for cell_opt in row {
                if let Some(n) = *cell_opt {
                    numbers.push(n);
                }
            }
        }
        numbers
    }

    fn evaluate(
        &self,
        ticket: &Ticket,
        call_position_by_number: &HashMap<u8, u32>,
    ) -> QualificationResult {
        let req: Vec<u8> = self.required_numbers(ticket);
        let mut max_pos: u32 = 0;

        for n in req {
            match call_position_by_number.get(&n) {
                Some(&pos) => {
                    if pos > max_pos {
                        max_pos = pos;
                    }
                }
                None => return QualificationResult::not_qualified(),
            }
        }

        QualificationResult::qualified(max_pos)
    }
}

/// Evaluator for Four Corners.
/// Convention: The leftmost and rightmost occupied cells of the top row (Row 0) and bottom row (Row 2).
/// Because each row has exactly 5 occupied cells, leftmost (index 0) and rightmost (index 4)
/// on rows 0 and 2 always yield exactly 4 distinct numbers.
/// Qualification position = MAX(call_position_by_number[n]) across the 4 corner numbers.
pub struct FourCornersEvaluator;

impl SchemeEvaluator for FourCornersEvaluator {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::FourCorners
    }

    fn required_numbers(&self, ticket: &Ticket) -> Vec<u8> {
        let mut corners: Vec<u8> = Vec::with_capacity(4);

        // Top row (Row 0): leftmost and rightmost occupied cells
        let row0_nums: Vec<u8> = ticket.cells[0].iter().filter_map(|&c| c).collect();
        if let (Some(&first), Some(&last)) = (row0_nums.first(), row0_nums.last()) {
            corners.push(first);
            corners.push(last);
        }

        // Bottom row (Row 2): leftmost and rightmost occupied cells
        let row2_nums: Vec<u8> = ticket.cells[2].iter().filter_map(|&c| c).collect();
        if let (Some(&first), Some(&last)) = (row2_nums.first(), row2_nums.last()) {
            corners.push(first);
            corners.push(last);
        }

        corners
    }

    fn evaluate(
        &self,
        ticket: &Ticket,
        call_position_by_number: &HashMap<u8, u32>,
    ) -> QualificationResult {
        let req: Vec<u8> = self.required_numbers(ticket);
        if req.len() != 4 {
            return QualificationResult::not_qualified();
        }

        let mut max_pos: u32 = 0;
        for n in req {
            match call_position_by_number.get(&n) {
                Some(&pos) => {
                    if pos > max_pos {
                        max_pos = pos;
                    }
                }
                None => return QualificationResult::not_qualified(),
            }
        }

        QualificationResult::qualified(max_pos)
    }
}

/// Evaluator for Early Five.
/// Any 5 numbers on the ticket are called.
/// Qualification position = 5th smallest call position among the ticket's called numbers.
pub struct EarlyFiveEvaluator;

impl SchemeEvaluator for EarlyFiveEvaluator {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::EarlyFive
    }

    fn required_numbers(&self, ticket: &Ticket) -> Vec<u8> {
        let mut numbers: Vec<u8> = Vec::with_capacity(15);
        for row in &ticket.cells {
            for cell_opt in row {
                if let Some(n) = *cell_opt {
                    numbers.push(n);
                }
            }
        }
        numbers
    }

    fn evaluate(
        &self,
        ticket: &Ticket,
        call_position_by_number: &HashMap<u8, u32>,
    ) -> QualificationResult {
        let mut called_positions: Vec<u32> = Vec::with_capacity(15);

        for row in &ticket.cells {
            for cell_opt in row {
                if let Some(n) = *cell_opt {
                    if let Some(&pos) = call_position_by_number.get(&n) {
                        called_positions.push(pos);
                    }
                }
            }
        }

        if called_positions.len() < 5 {
            return QualificationResult::not_qualified();
        }

        called_positions.sort_unstable();
        // 5th smallest call position is at index 4 (0-based)
        QualificationResult::qualified(called_positions[4])
    }
}

/// Static registry mapping SchemeType to pure SchemeEvaluator implementations.
pub struct SchemeRegistry {
    evaluators: HashMap<SchemeType, Box<dyn SchemeEvaluator>>,
}

impl SchemeRegistry {
    pub fn new() -> Self {
        let mut evaluators: HashMap<SchemeType, Box<dyn SchemeEvaluator>> = HashMap::new();
        evaluators.insert(SchemeType::EarlyFive, Box::new(EarlyFiveEvaluator));
        evaluators.insert(SchemeType::FourCorners, Box::new(FourCornersEvaluator));
        evaluators.insert(SchemeType::TopLine, Box::new(LineEvaluator::top_line()));
        evaluators.insert(
            SchemeType::MiddleLine,
            Box::new(LineEvaluator::middle_line()),
        );
        evaluators.insert(
            SchemeType::BottomLine,
            Box::new(LineEvaluator::bottom_line()),
        );
        evaluators.insert(SchemeType::FullHouse, Box::new(FullHouseEvaluator));

        Self { evaluators }
    }

    pub fn get(&self, scheme_type: SchemeType) -> &dyn SchemeEvaluator {
        self.evaluators
            .get(&scheme_type)
            .map(|b| b.as_ref())
            .expect("All SchemeTypes must be registered in SchemeRegistry")
    }
}

impl Default for SchemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrizeScheme {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub scheme_type: SchemeType,
    pub pattern_kind: PatternKind,
    pub mask: PatternMask,
    pub early_n: Option<u32>,
    pub is_full_house: bool,
    pub max_winners: u32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSchemeInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub scheme_type: Option<SchemeType>,
    pub pattern_kind: Option<PatternKind>,
    pub mask: Option<PatternMask>,
    pub early_n: Option<u32>,
    pub is_full_house: Option<bool>,
    pub max_winners: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemeTypeInfo {
    pub scheme_type: SchemeType,
    pub is_full_house: bool,
    pub rule_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternPreset {
    pub name: String,
    pub description: String,
    pub pattern_kind: PatternKind,
    pub mask: PatternMask,
    pub early_n: Option<u32>,
    pub is_full_house: bool,
}

pub fn list_pattern_presets() -> Vec<PatternPreset> {
    vec![
        PatternPreset {
            name: "Early 5".to_string(),
            description: "Any 5 numbers on the ticket are called.".to_string(),
            pattern_kind: PatternKind::EarlyN,
            mask: [[false; 5]; 3],
            early_n: Some(5),
            is_full_house: false,
        },
        PatternPreset {
            name: "Early 7".to_string(),
            description: "Any 7 numbers on the ticket are called.".to_string(),
            pattern_kind: PatternKind::EarlyN,
            mask: [[false; 5]; 3],
            early_n: Some(7),
            is_full_house: false,
        },
        PatternPreset {
            name: "Early 10".to_string(),
            description: "Any 10 numbers on the ticket are called.".to_string(),
            pattern_kind: PatternKind::EarlyN,
            mask: [[false; 5]; 3],
            early_n: Some(10),
            is_full_house: false,
        },
        PatternPreset {
            name: "Four Corners".to_string(),
            description: "The 4 outermost corners of the ticket.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [true, false, false, false, true],
                [false, false, false, false, false],
                [true, false, false, false, true],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Top Line".to_string(),
            description: "All 5 numbers in Row 1 are struck.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [true, true, true, true, true],
                [false, false, false, false, false],
                [false, false, false, false, false],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Middle Line".to_string(),
            description: "All 5 numbers in Row 2 are struck.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [false, false, false, false, false],
                [true, true, true, true, true],
                [false, false, false, false, false],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Bottom Line".to_string(),
            description: "All 5 numbers in Row 3 are struck.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [false, false, false, false, false],
                [false, false, false, false, false],
                [true, true, true, true, true],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Pattern H".to_string(),
            description: "Columns 1 and 5 all rows, plus middle row.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [true, false, false, false, true],
                [true, true, true, true, true],
                [true, false, false, false, true],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Pattern Plus".to_string(),
            description: "Middle row and middle column.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [false, false, true, false, false],
                [true, true, true, true, true],
                [false, false, true, false, false],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Pattern V".to_string(),
            description: "V-shaped pattern across the grid.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [true, false, false, false, true],
                [false, true, false, true, false],
                [false, false, true, false, false],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Pattern A".to_string(),
            description: "A-shaped pattern across the grid.".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [false, false, true, false, false],
                [true, true, true, true, true],
                [true, false, false, false, true],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Photo Frame".to_string(),
            description: "Outer border cells (photo frame).".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [true, true, true, true, true],
                [true, false, false, false, true],
                [true, true, true, true, true],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Breakfast".to_string(),
            description: "Logical columns 1 and 2 (left two columns).".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [true, true, false, false, false],
                [true, true, false, false, false],
                [true, true, false, false, false],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Lunch".to_string(),
            description: "Logical column 3 (center column).".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [false, false, true, false, false],
                [false, false, true, false, false],
                [false, false, true, false, false],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Dinner".to_string(),
            description: "Logical columns 4 and 5 (right two columns).".to_string(),
            pattern_kind: PatternKind::Mask,
            mask: [
                [false, false, false, true, true],
                [false, false, false, true, true],
                [false, false, false, true, true],
            ],
            early_n: None,
            is_full_house: false,
        },
        PatternPreset {
            name: "Full House".to_string(),
            description: "All 15 numbers on the ticket are struck.".to_string(),
            pattern_kind: PatternKind::FullHouse,
            mask: [[true; 5]; 3],
            early_n: None,
            is_full_house: true,
        },
        PatternPreset {
            name: "Full House 2".to_string(),
            description: "Second Full House award (all 15 numbers).".to_string(),
            pattern_kind: PatternKind::FullHouse,
            mask: [[true; 5]; 3],
            early_n: None,
            is_full_house: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::random::DeterministicRandomSource;
    use crate::domain::tambola::ticket_generator::{generate_ticket, validate_ticket};

    /// Constructs a Ticket from a 3x9 grid of optional numbers and asserts validity against full Tambola rules.
    fn ticket_from_rows(rows: [[Option<u8>; 9]; 3]) -> Ticket {
        let ticket: Ticket = Ticket { cells: rows };
        validate_ticket(&ticket).expect("ticket fixture must pass Tambola validation");
        ticket
    }

    /// Builds a 1-based call position map from a sequential draw history.
    fn positions_from_draws(draws: &[u8]) -> HashMap<u8, u32> {
        let mut map: HashMap<u8, u32> = HashMap::new();
        for (i, &n) in draws.iter().enumerate() {
            let pos: u32 = (i as u32) + 1;
            assert!(
                map.insert(n, pos).is_none(),
                "draws sequence must not contain duplicate numbers: {}",
                n
            );
        }
        map
    }

    #[test]
    fn test_line_evaluators_partial_vs_complete() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(999);
        let ticket: Ticket = generate_ticket(&mut rng).unwrap();

        let top_eval = LineEvaluator::top_line();
        let req_top: Vec<u8> = top_eval.required_numbers(&ticket);
        assert_eq!(req_top.len(), 5);

        // Draw only 4 of the 5 numbers: not qualified
        let partial_draws: Vec<u8> = vec![req_top[1], req_top[3], req_top[0], req_top[2]];
        let call_pos_partial: HashMap<u8, u32> = positions_from_draws(&partial_draws);

        let res_partial: QualificationResult = top_eval.evaluate(&ticket, &call_pos_partial);
        assert!(!res_partial.is_qualified);
        assert_eq!(res_partial.qualification_position, None);

        // Draw 5th number at pos 5
        let complete_draws: Vec<u8> =
            vec![req_top[1], req_top[3], req_top[0], req_top[2], req_top[4]];
        let call_pos_complete: HashMap<u8, u32> = positions_from_draws(&complete_draws);

        let res_complete: QualificationResult = top_eval.evaluate(&ticket, &call_pos_complete);
        assert!(res_complete.is_qualified);
        assert_eq!(res_complete.qualification_position, Some(5));
    }

    #[test]
    fn test_four_corners_mandatory_spec_scenario() {
        // Build a 100% valid Tambola ticket passing validate_ticket():
        // Row 0: 5 numbers across cols [0, 2, 4, 5, 6]. Leftmost = 4, Rightmost = 61.
        // Row 1: 5 numbers across cols [1, 3, 4, 7, 8].
        // Row 2: 5 numbers across cols [0, 1, 2, 3, 6]. Leftmost = 9, Rightmost = 63.
        let ticket: Ticket = ticket_from_rows([
            [
                Some(4),
                None,
                Some(22),
                None,
                Some(45),
                Some(52),
                Some(61),
                None,
                None,
            ],
            [
                None,
                Some(15),
                None,
                Some(35),
                Some(46),
                None,
                None,
                Some(75),
                Some(82),
            ],
            [
                Some(9),
                Some(19),
                Some(29),
                Some(39),
                None,
                None,
                Some(63),
                None,
                None,
            ],
        ]);

        let corners_eval = FourCornersEvaluator;
        let corners: Vec<u8> = corners_eval.required_numbers(&ticket);
        assert_eq!(corners, vec![4, 61, 9, 63]);

        // Spec scenario draw sequence ending with ..., 12, 63, 55.
        // Draw higher corner 61 early (pos 1), 9 at pos 2, 4 at pos 3.
        // 12 (non-corner) at pos 4, 63 (4th corner) at pos 5, 55 (current draw) at pos 6.
        let draws: Vec<u8> = vec![61, 9, 4, 12, 63, 55];
        let call_pos: HashMap<u8, u32> = positions_from_draws(&draws);

        let res: QualificationResult = corners_eval.evaluate(&ticket, &call_pos);
        assert!(res.is_qualified);

        // Qualification position equals pos of 63 (position 5), NOT the current draw position (55 at pos 6).
        // Magnitude order (61 > 63 > 9 > 4) explicitly disagrees with chronological draw order.
        assert_eq!(res.qualification_position, Some(5));
        assert_ne!(res.qualification_position, Some(6));
    }

    #[test]
    fn test_magnitude_disagreement_lines_and_full_house() {
        let ticket: Ticket = ticket_from_rows([
            [
                Some(4),
                None,
                Some(22),
                None,
                Some(45),
                Some(52),
                Some(61),
                None,
                None,
            ],
            [
                None,
                Some(15),
                None,
                Some(35),
                Some(46),
                None,
                None,
                Some(75),
                Some(82),
            ],
            [
                Some(9),
                Some(19),
                Some(29),
                Some(39),
                None,
                None,
                Some(63),
                None,
                None,
            ],
        ]);

        // 1. Top line: numbers are [4, 22, 45, 52, 61].
        // Draw in reverse numerical order: largest (61) drawn first (pos 1), smallest (4) drawn last (pos 5).
        let top_draws: Vec<u8> = vec![61, 52, 45, 22, 4];
        let top_call_pos: HashMap<u8, u32> = positions_from_draws(&top_draws);
        let top_eval = LineEvaluator::top_line();

        let top_res: QualificationResult = top_eval.evaluate(&ticket, &top_call_pos);
        assert!(top_res.is_qualified);
        // Position must equal chronologically last call (4 at pos 5), NOT the largest magnitude (61 at pos 1)
        assert_eq!(top_res.qualification_position, Some(5));

        // 2. Full house: all 15 numbers drawn in reverse magnitude order.
        let fh_eval = FullHouseEvaluator;
        let mut all_nums: Vec<u8> = fh_eval.required_numbers(&ticket);
        all_nums.sort_by(|a, b| b.cmp(a)); // sorted descending: 82, 75, 63, ... 4

        let fh_call_pos: HashMap<u8, u32> = positions_from_draws(&all_nums);
        let fh_res: QualificationResult = fh_eval.evaluate(&ticket, &fh_call_pos);
        assert!(fh_res.is_qualified);
        // Completed at pos 15 when the smallest number (4) is drawn, not pos 1 (82).
        assert_eq!(fh_res.qualification_position, Some(15));
    }

    #[test]
    fn test_early_five_4_vs_7_called() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(777);
        let ticket: Ticket = generate_ticket(&mut rng).unwrap();
        let early_five = EarlyFiveEvaluator;

        let all_nums: Vec<u8> = early_five.required_numbers(&ticket);
        assert_eq!(all_nums.len(), 15);

        // 1. Exactly 4 called: not qualified
        let draws_4: Vec<u8> = vec![all_nums[1], all_nums[3], all_nums[0], all_nums[2]];
        let call_pos_4: HashMap<u8, u32> = positions_from_draws(&draws_4);

        let res4: QualificationResult = early_five.evaluate(&ticket, &call_pos_4);
        assert!(!res4.is_qualified);
        assert_eq!(res4.qualification_position, None);

        // 2. 7 ticket numbers called in mixed chronological order:
        // Sequence: [n1, n3, n5, n0, n6, n2, n4]
        // Corresponding draw positions:
        // n1 -> 1, n3 -> 2, n5 -> 3, n0 -> 4, n6 -> 5 (5th ticket number drawn!), n2 -> 6, n4 -> 7
        let draws_7: Vec<u8> = vec![
            all_nums[1],
            all_nums[3],
            all_nums[5],
            all_nums[0],
            all_nums[6],
            all_nums[2],
            all_nums[4],
        ];
        let call_pos_7: HashMap<u8, u32> = positions_from_draws(&draws_7);

        let res7: QualificationResult = early_five.evaluate(&ticket, &call_pos_7);
        assert!(res7.is_qualified);
        assert_eq!(res7.qualification_position, Some(5));
    }

    #[test]
    fn test_early_five_out_of_order_draws_with_non_ticket_calls() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(321);
        let ticket: Ticket = generate_ticket(&mut rng).unwrap();
        let early_five = EarlyFiveEvaluator;
        let all_nums: Vec<u8> = early_five.required_numbers(&ticket);

        // Pick 6 numbers from ticket:
        let t0 = all_nums[0];
        let t1 = all_nums[1];
        let t2 = all_nums[2];
        let t3 = all_nums[3];
        let t4 = all_nums[4];
        let t5 = all_nums[5];

        // Interleave with non-ticket numbers 89 and 90 (assuming they're not in the 6 chosen):
        // Draw sequence:
        // pos 1: t3 (ticket 1st)
        // pos 2: 89 (dummy)
        // pos 3: t0 (ticket 2nd)
        // pos 4: t4 (ticket 3rd)
        // pos 5: 90 (dummy)
        // pos 6: t1 (ticket 4th)
        // pos 7: t5 (ticket 5th -> completes Early Five at pos 7!)
        // pos 8: t2 (ticket 6th)
        let non_ticket_1: u8 = (1..=90).find(|n| !all_nums.contains(n)).unwrap();
        let non_ticket_2: u8 = (1..=90)
            .find(|n| !all_nums.contains(n) && *n != non_ticket_1)
            .unwrap();

        let draws: Vec<u8> = vec![t3, non_ticket_1, t0, t4, non_ticket_2, t1, t5, t2];
        let call_pos: HashMap<u8, u32> = positions_from_draws(&draws);

        let res: QualificationResult = early_five.evaluate(&ticket, &call_pos);
        assert!(res.is_qualified);
        // The 5th ticket number drawn was t5 at pos 7
        assert_eq!(res.qualification_position, Some(7));
    }

    #[test]
    fn test_full_house_complete_all_15() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(888);
        let ticket: Ticket = generate_ticket(&mut rng).unwrap();
        let fh_eval = FullHouseEvaluator;
        let all_nums: Vec<u8> = fh_eval.required_numbers(&ticket);

        let call_pos: HashMap<u8, u32> = positions_from_draws(&all_nums);

        let res: QualificationResult = fh_eval.evaluate(&ticket, &call_pos);
        assert!(res.is_qualified);
        assert_eq!(res.qualification_position, Some(15));
    }

    #[test]
    fn test_generic_pattern_evaluator_presets_classic90_and_song_grid() {
        use crate::domain::tambola::ticket_generator::{
            generate_song_grid_ticket, generate_ticket,
        };

        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(1234);
        let classic_ticket: Ticket = generate_ticket(&mut rng).unwrap();
        let classic_cells: LogicalTicketCells = ticket_to_logical_cells(&classic_ticket);

        let song_grid_ticket = generate_song_grid_ticket(45, &mut rng).unwrap();
        let song_grid_cells: LogicalTicketCells = song_grid_to_logical_cells(&song_grid_ticket);

        let grids: [(&str, LogicalTicketCells); 2] =
            [("classic", classic_cells), ("song_grid", song_grid_cells)];

        let presets: Vec<PatternPreset> = list_pattern_presets();

        for (_label, cells) in grids {
            // Flatten all 15 cells in order
            let mut flat_numbers: Vec<u8> = Vec::with_capacity(15);
            for row in &cells {
                for &num in row {
                    assert!(num > 0, "Cell number must be positive");
                    flat_numbers.push(num);
                }
            }
            assert_eq!(flat_numbers.len(), 15);

            let all_draw_positions: HashMap<u8, u32> = positions_from_draws(&flat_numbers);

            for preset in &presets {
                let res: QualificationResult = GenericPatternEvaluator::evaluate(
                    &cells,
                    preset.pattern_kind,
                    &preset.mask,
                    preset.early_n,
                    &all_draw_positions,
                );

                assert!(
                    res.is_qualified,
                    "Preset '{}' must qualify when all numbers are drawn",
                    preset.name
                );
                assert!(res.qualification_position.is_some());
            }

            // Test Early 7 partial draw: only first 6 numbers drawn -> not qualified
            let partial_6: HashMap<u8, u32> = positions_from_draws(&flat_numbers[..6]);
            let early_7_preset = presets.iter().find(|p| p.name == "Early 7").unwrap();
            let res_early_7: QualificationResult = GenericPatternEvaluator::evaluate(
                &cells,
                early_7_preset.pattern_kind,
                &early_7_preset.mask,
                early_7_preset.early_n,
                &partial_6,
            );
            assert!(
                !res_early_7.is_qualified,
                "Early 7 with 6 drawn numbers must not qualify"
            );

            // With 7th number drawn -> qualifies at pos 7
            let partial_7: HashMap<u8, u32> = positions_from_draws(&flat_numbers[..7]);
            let res_early_7_ok: QualificationResult = GenericPatternEvaluator::evaluate(
                &cells,
                early_7_preset.pattern_kind,
                &early_7_preset.mask,
                early_7_preset.early_n,
                &partial_7,
            );
            assert!(res_early_7_ok.is_qualified);
            assert_eq!(res_early_7_ok.qualification_position, Some(7));

            // Test Breakfast (columns 0 and 1): 6 numbers total
            let breakfast_preset = presets.iter().find(|p| p.name == "Breakfast").unwrap();
            let breakfast_nums: Vec<u8> = vec![
                cells[0][0],
                cells[0][1],
                cells[1][0],
                cells[1][1],
                cells[2][0],
                cells[2][1],
            ];
            // Draw 5 of the 6 -> not qualified
            let partial_breakfast: HashMap<u8, u32> = positions_from_draws(&breakfast_nums[..5]);
            let res_bf_fail: QualificationResult = GenericPatternEvaluator::evaluate(
                &cells,
                breakfast_preset.pattern_kind,
                &breakfast_preset.mask,
                breakfast_preset.early_n,
                &partial_breakfast,
            );
            assert!(!res_bf_fail.is_qualified);

            // Draw all 6 -> qualifies at pos 6
            let full_breakfast: HashMap<u8, u32> = positions_from_draws(&breakfast_nums);
            let res_bf_ok: QualificationResult = GenericPatternEvaluator::evaluate(
                &cells,
                breakfast_preset.pattern_kind,
                &breakfast_preset.mask,
                breakfast_preset.early_n,
                &full_breakfast,
            );
            assert!(res_bf_ok.is_qualified);
            assert_eq!(res_bf_ok.qualification_position, Some(6));
        }
    }
}
