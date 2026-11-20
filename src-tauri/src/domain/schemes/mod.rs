use crate::domain::tambola::ticket_generator::Ticket;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

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
    pub scheme_type: SchemeType,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::random::DeterministicRandomSource;
    use crate::domain::tambola::ticket_generator::generate_ticket;

    #[test]
    fn test_line_evaluators_partial_vs_complete() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(999);
        let ticket: Ticket = generate_ticket(&mut rng).unwrap();

        let top_eval = LineEvaluator::top_line();
        let req_top: Vec<u8> = top_eval.required_numbers(&ticket);
        assert_eq!(req_top.len(), 5);

        // Map only 4 of the 5 numbers: not qualified
        let mut call_pos: HashMap<u8, u32> = HashMap::new();
        call_pos.insert(req_top[0], 10);
        call_pos.insert(req_top[1], 5);
        call_pos.insert(req_top[2], 20);
        call_pos.insert(req_top[3], 15);

        let res_partial: QualificationResult = top_eval.evaluate(&ticket, &call_pos);
        assert!(!res_partial.is_qualified);
        assert_eq!(res_partial.qualification_position, None);

        // Add 5th number at pos 12: max is 20
        call_pos.insert(req_top[4], 12);
        let res_complete: QualificationResult = top_eval.evaluate(&ticket, &call_pos);
        assert!(res_complete.is_qualified);
        assert_eq!(res_complete.qualification_position, Some(20));
    }

    #[test]
    fn test_four_corners_mandatory_spec_scenario() {
        // Spec scenario:
        // Draws [12, 63, 55], ticket whose corners are completed by 63 has position == 2
        let mut ticket: Ticket = Ticket::empty();
        // Construct ticket with known corners
        ticket.cells[0][0] = Some(4);
        ticket.cells[0][1] = Some(12);
        ticket.cells[0][2] = Some(22);
        ticket.cells[0][3] = Some(31);
        ticket.cells[0][4] = Some(45); // top right corner

        ticket.cells[1][0] = Some(5);
        ticket.cells[1][1] = Some(15);
        ticket.cells[1][2] = Some(25);
        ticket.cells[1][3] = Some(35);
        ticket.cells[1][4] = Some(46);

        ticket.cells[2][0] = Some(9); // bottom left corner
        ticket.cells[2][1] = Some(19);
        ticket.cells[2][2] = Some(29);
        ticket.cells[2][3] = Some(39);
        ticket.cells[2][7] = Some(72);
        ticket.cells[2][8] = Some(63); // bottom right corner

        let corners_eval = FourCornersEvaluator;
        let corners: Vec<u8> = corners_eval.required_numbers(&ticket);
        assert_eq!(corners, vec![4, 45, 9, 63]);

        // Draws: 4 at pos 0 (before test start), 9 at pos 1 (call 12 not a corner), 45 at pos 1, 63 at pos 2, 55 at pos 3
        let mut call_pos: HashMap<u8, u32> = HashMap::new();
        call_pos.insert(4, 1);
        call_pos.insert(9, 1);
        call_pos.insert(45, 1);
        call_pos.insert(12, 1); // 12 drawn at pos 1
        call_pos.insert(63, 2); // 63 drawn at pos 2 (completes corners)
        call_pos.insert(55, 3); // 55 drawn at pos 3 (after corners completed)

        let res: QualificationResult = corners_eval.evaluate(&ticket, &call_pos);
        assert!(res.is_qualified);
        // Qualification position must be exactly 2 (MAX of corner call positions), NOT comparing number magnitudes!
        assert_eq!(res.qualification_position, Some(2));
    }

    #[test]
    fn test_early_five_4_vs_7_called() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(777);
        let ticket: Ticket = generate_ticket(&mut rng).unwrap();
        let early_five = EarlyFiveEvaluator;

        // Flatten all 15 numbers on ticket
        let all_nums: Vec<u8> = early_five.required_numbers(&ticket);
        assert_eq!(all_nums.len(), 15);

        // 1. Exactly 4 called: not qualified
        let mut call_pos: HashMap<u8, u32> = HashMap::new();
        call_pos.insert(all_nums[0], 10);
        call_pos.insert(all_nums[1], 2);
        call_pos.insert(all_nums[2], 18);
        call_pos.insert(all_nums[3], 7);

        let res4: QualificationResult = early_five.evaluate(&ticket, &call_pos);
        assert!(!res4.is_qualified);
        assert_eq!(res4.qualification_position, None);

        // 2. Add 3 more called (total 7 called) with positions: [10, 2, 18, 7, 14, 5, 25]
        // Sorted call positions: [2, 5, 7, 10, 14, 18, 25]
        // 5th smallest is 14 (index 4)
        call_pos.insert(all_nums[4], 14);
        call_pos.insert(all_nums[5], 5);
        call_pos.insert(all_nums[6], 25);

        let res7: QualificationResult = early_five.evaluate(&ticket, &call_pos);
        assert!(res7.is_qualified);
        assert_eq!(res7.qualification_position, Some(14));
    }

    #[test]
    fn test_full_house_complete_all_15() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(888);
        let ticket: Ticket = generate_ticket(&mut rng).unwrap();
        let fh_eval = FullHouseEvaluator;
        let all_nums: Vec<u8> = fh_eval.required_numbers(&ticket);

        let mut call_pos: HashMap<u8, u32> = HashMap::new();
        for (i, &n) in all_nums.iter().enumerate() {
            call_pos.insert(n, (i as u32) + 1);
        }

        let res: QualificationResult = fh_eval.evaluate(&ticket, &call_pos);
        assert!(res.is_qualified);
        assert_eq!(res.qualification_position, Some(15));
    }
}
