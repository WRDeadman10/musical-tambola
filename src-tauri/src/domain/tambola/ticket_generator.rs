use crate::domain::random::RandomSource;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    pub cells: [[Option<u8>; 9]; 3],
}

impl Ticket {
    pub fn empty() -> Self {
        Self {
            cells: [[None; 9]; 3],
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TicketError {
    #[error("Ticket must have exactly 15 numbers, found {0}")]
    InvalidTotalNumbers(usize),
    #[error("Row {row} must have exactly 5 numbers, found {count}")]
    InvalidRowNumberCount { row: usize, count: usize },
    #[error("Column {col} must have between 1 and 3 numbers, found {count}")]
    InvalidColumnNumberCount { col: usize, count: usize },
    #[error("Number {number} in column {col} is out of bounds [{min}, {max}]")]
    NumberOutOfRange {
        col: usize,
        number: u8,
        min: u8,
        max: u8,
    },
    #[error(
        "Column {col} is not sorted: upper number {upper_number} >= lower number {lower_number}"
    )]
    ColumnNotSorted {
        col: usize,
        upper_number: u8,
        lower_number: u8,
    },
    #[error("Duplicate number {0} found on ticket")]
    DuplicateNumber(u8),
    #[error("Failed to generate a valid ticket layout via backtracking")]
    LayoutGenerationFailed,
}

pub fn col_min(col: usize) -> u8 {
    match col {
        0 => 1,
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        6 => 60,
        7 => 70,
        8 => 80,
        _ => 1,
    }
}

pub fn col_max(col: usize) -> u8 {
    match col {
        0 => 9,
        1 => 19,
        2 => 29,
        3 => 39,
        4 => 49,
        5 => 59,
        6 => 69,
        7 => 79,
        8 => 90,
        _ => 90,
    }
}

pub fn validate_ticket(ticket: &Ticket) -> Result<(), TicketError> {
    let mut total_numbers: usize = 0;
    let mut col_counts: [usize; 9] = [0usize; 9];
    let mut seen: [bool; 91] = [false; 91];

    for (r, row) in ticket.cells.iter().enumerate() {
        let mut row_count: usize = 0;
        for (c, cell_opt) in row.iter().enumerate() {
            if let Some(n) = *cell_opt {
                row_count += 1;
                total_numbers += 1;
                col_counts[c] += 1;

                let min: u8 = col_min(c);
                let max: u8 = col_max(c);
                if n < min || n > max {
                    return Err(TicketError::NumberOutOfRange {
                        col: c,
                        number: n,
                        min,
                        max,
                    });
                }

                if seen[n as usize] {
                    return Err(TicketError::DuplicateNumber(n));
                }
                seen[n as usize] = true;
            }
        }

        if row_count != 5 {
            return Err(TicketError::InvalidRowNumberCount {
                row: r,
                count: row_count,
            });
        }
    }

    if total_numbers != 15 {
        return Err(TicketError::InvalidTotalNumbers(total_numbers));
    }

    for c in 0..9 {
        let count: usize = col_counts[c];
        if !(1..=3).contains(&count) {
            return Err(TicketError::InvalidColumnNumberCount { col: c, count });
        }

        let mut prev_num: Option<u8> = None;
        for row in &ticket.cells {
            if let Some(curr) = row[c] {
                if let Some(prev) = prev_num {
                    if curr <= prev {
                        return Err(TicketError::ColumnNotSorted {
                            col: c,
                            upper_number: prev,
                            lower_number: curr,
                        });
                    }
                }
                prev_num = Some(curr);
            }
        }
    }

    Ok(())
}

pub fn canonical_layout(ticket: &Ticket) -> String {
    let mut rows: Vec<String> = Vec::with_capacity(3);
    for row in &ticket.cells {
        let mut row_str: String = String::with_capacity(27);
        for (c, cell_opt) in row.iter().enumerate() {
            if c > 0 {
                row_str.push('|');
            }
            match cell_opt {
                Some(n) => row_str.push_str(&n.to_string()),
                None => row_str.push('_'),
            }
        }
        rows.push(row_str);
    }
    rows.join("\n")
}

pub fn layout_hash(ticket: &Ticket) -> String {
    let canonical: String = canonical_layout(ticket);
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(canonical.as_bytes());
    let result: sha2::digest::Output<Sha256> = hasher.finalize();
    format!("{:x}", result)
}

fn backtrack_layout<R: RandomSource>(
    col: usize,
    col_counts: &[usize; 9],
    row_counts: &mut [usize; 3],
    grid: &mut [[bool; 9]; 3],
    rng: &mut R,
) -> bool {
    if col == 9 {
        return row_counts[0] == 5 && row_counts[1] == 5 && row_counts[2] == 5;
    }

    let remaining_cols: usize = 9 - col;
    for &count in row_counts.iter() {
        if count > 5 || count + remaining_cols < 5 {
            return false;
        }
    }

    let k: usize = col_counts[col];
    let mut candidate_subsets: Vec<Vec<usize>> = match k {
        3 => vec![vec![0, 1, 2]],
        2 => vec![vec![0, 1], vec![0, 2], vec![1, 2]],
        1 => vec![vec![0], vec![1], vec![2]],
        _ => return false,
    };

    // Shuffle candidate row combinations to guarantee unbiased assignment across all rows
    for i in (1..candidate_subsets.len()).rev() {
        let j: usize = rng.next_index(i + 1);
        candidate_subsets.swap(i, j);
    }

    for subset in candidate_subsets {
        let can_add: bool = subset.iter().all(|&r| row_counts[r] < 5);
        if can_add {
            for &r in &subset {
                row_counts[r] += 1;
                grid[r][col] = true;
            }

            if backtrack_layout(col + 1, col_counts, row_counts, grid, rng) {
                return true;
            }

            for &r in &subset {
                row_counts[r] -= 1;
                grid[r][col] = false;
            }
        }
    }

    false
}

fn generate_valid_layout<R: RandomSource>(rng: &mut R) -> Result<[[bool; 9]; 3], TicketError> {
    // Try up to 50 randomized partition attempts with backtracking
    for _ in 0..50 {
        // Step 1: Uniformly partition 15 numbers into 9 column counts in {1, 2, 3}
        let mut col_counts: [usize; 9] = [1; 9];
        let mut extra: usize = 6;
        let mut attempts: usize = 0;
        while extra > 0 && attempts < 100 {
            attempts += 1;
            let col: usize = rng.next_index(9);
            if col_counts[col] < 3 {
                col_counts[col] += 1;
                extra -= 1;
            }
        }
        if extra > 0 {
            for count in &mut col_counts {
                while extra > 0 && *count < 3 {
                    *count += 1;
                    extra -= 1;
                }
            }
        }

        // Step 2: Randomized backtracking assignment to rows 0, 1, 2 (each row exactly 5)
        let mut row_counts: [usize; 3] = [0, 0, 0];
        let mut grid: [[bool; 9]; 3] = [[false; 9]; 3];

        if backtrack_layout(0, &col_counts, &mut row_counts, &mut grid, rng) {
            return Ok(grid);
        }
    }

    Err(TicketError::LayoutGenerationFailed)
}

pub fn generate_ticket<R: RandomSource>(rng: &mut R) -> Result<Ticket, TicketError> {
    let layout: [[bool; 9]; 3] = generate_valid_layout(rng)?;
    let mut ticket: Ticket = Ticket::empty();

    for c in 0..9 {
        let occupied_rows: Vec<usize> = (0..3).filter(|&r| layout[r][c]).collect();
        let k: usize = occupied_rows.len();

        let min: u8 = col_min(c);
        let max: u8 = col_max(c);
        let mut pool: Vec<u8> = (min..=max).collect();
        let mut picked: Vec<u8> = Vec::with_capacity(k);

        for _ in 0..k {
            let idx: usize = rng.next_index(pool.len());
            picked.push(pool.swap_remove(idx));
        }

        picked.sort_unstable();

        for (i, &r) in occupied_rows.iter().enumerate() {
            ticket.cells[r][c] = Some(picked[i]);
        }
    }

    validate_ticket(&ticket)?;
    Ok(ticket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::random::{DeterministicRandomSource, OsRngSource};

    #[test]
    fn test_generate_10000_tickets_all_valid() {
        let mut rng: OsRngSource = OsRngSource::new();
        for _ in 0..10_000 {
            let ticket: Ticket = generate_ticket(&mut rng).expect("generated ticket must be valid");
            assert!(validate_ticket(&ticket).is_ok());
        }
    }

    #[test]
    fn test_statistical_distribution_unbiased() {
        let mut rng: OsRngSource = OsRngSource::new();
        // Track per-column count occurrences (1, 2, 3) and bottom-row singleton occurrences
        let mut count_occurrences: [[usize; 4]; 9] = [[0; 4]; 9]; // count_occurrences[col][1..=3]
        let mut singleton_in_row2: [usize; 9] = [0; 9];

        for _ in 0..20_000 {
            let ticket: Ticket = generate_ticket(&mut rng).expect("generated ticket must be valid");
            for c in 0..9 {
                let occupied_rows: Vec<usize> =
                    (0..3).filter(|&r| ticket.cells[r][c].is_some()).collect();
                let count: usize = occupied_rows.len();
                count_occurrences[c][count] += 1;

                if count == 1 && occupied_rows[0] == 2 {
                    singleton_in_row2[c] += 1;
                }
            }
        }

        for c in 0..9 {
            // (a) Every column must have the "count = 1 and in row 2" case occur (> 0)
            assert!(
                singleton_in_row2[c] > 0,
                "Column {} never had count = 1 located in row 2 across 20,000 samples",
                c
            );

            // (b) Every column count value 1, 2, and 3 must occur for every column (> 0)
            assert!(
                count_occurrences[c][1] > 0,
                "Column {} never had count = 1",
                c
            );
            assert!(
                count_occurrences[c][2] > 0,
                "Column {} never had count = 2",
                c
            );
            assert!(
                count_occurrences[c][3] > 0,
                "Column {} never had count = 3",
                c
            );
        }
    }

    #[test]
    fn test_deterministic_reproducibility() {
        let mut rng1: DeterministicRandomSource = DeterministicRandomSource::new(42);
        let mut rng2: DeterministicRandomSource = DeterministicRandomSource::new(42);

        let t1: Ticket = generate_ticket(&mut rng1).unwrap();
        let t2: Ticket = generate_ticket(&mut rng2).unwrap();

        assert_eq!(t1, t2);
        assert_eq!(canonical_layout(&t1), canonical_layout(&t2));
        assert_eq!(layout_hash(&t1), layout_hash(&t2));
    }

    #[test]
    fn test_validation_rejects_invalid_row_counts() {
        let mut ticket: Ticket = Ticket::empty();
        for (c, cell) in ticket.cells[0].iter_mut().enumerate().take(6) {
            *cell = Some(col_min(c));
        }
        let res: Result<(), TicketError> = validate_ticket(&ticket);
        assert!(res.is_err());
    }

    #[test]
    fn test_validation_rejects_unsorted_column() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(12345);
        let mut ticket: Ticket = generate_ticket(&mut rng).unwrap();

        for c in 0..9 {
            let rows: Vec<usize> = (0..3).filter(|&r| ticket.cells[r][c].is_some()).collect();
            if rows.len() >= 2 {
                let n0: Option<u8> = ticket.cells[rows[0]][c];
                let n1: Option<u8> = ticket.cells[rows[1]][c];
                ticket.cells[rows[0]][c] = n1;
                ticket.cells[rows[1]][c] = n0;
                break;
            }
        }

        let res: Result<(), TicketError> = validate_ticket(&ticket);
        assert!(matches!(res, Err(TicketError::ColumnNotSorted { .. })));
    }
}
