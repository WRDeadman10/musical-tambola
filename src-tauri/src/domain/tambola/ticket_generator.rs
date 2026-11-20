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
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn generate_valid_layout<R: RandomSource>(rng: &mut R) -> [[bool; 9]; 3] {
    for _ in 0..500 {
        let mut grid: [[bool; 9]; 3] = [[false; 9]; 3];
        let mut col_counts: [u8; 9] = [0u8; 9];

        // Row 0: Pick 5 distinct columns at random
        let mut available: Vec<usize> = (0..9).collect();
        for _ in 0..5 {
            let idx: usize = rng.next_index(available.len());
            let col: usize = available.swap_remove(idx);
            grid[0][col] = true;
            col_counts[col] += 1;
        }

        // Row 1: Prioritize columns with count 0 (at most 4)
        let mut zero_cols: Vec<usize> = (0..9).filter(|&c| col_counts[c] == 0).collect();
        let mut other_cols: Vec<usize> = (0..9).filter(|&c| col_counts[c] > 0).collect();

        let mut row1_cols: Vec<usize> = Vec::with_capacity(5);
        while !zero_cols.is_empty() && row1_cols.len() < 5 {
            let idx: usize = rng.next_index(zero_cols.len());
            let col: usize = zero_cols.swap_remove(idx);
            row1_cols.push(col);
        }
        while row1_cols.len() < 5 && !other_cols.is_empty() {
            let idx: usize = rng.next_index(other_cols.len());
            let col: usize = other_cols.swap_remove(idx);
            row1_cols.push(col);
        }
        for &col in &row1_cols {
            grid[1][col] = true;
            col_counts[col] += 1;
        }

        // Row 2: Any remaining zero columns must be included
        let remaining_zero_cols: Vec<usize> = (0..9).filter(|&c| col_counts[c] == 0).collect();
        if remaining_zero_cols.len() > 5 {
            continue;
        }
        let mut remaining_available: Vec<usize> = (0..9)
            .filter(|&c| col_counts[c] > 0 && col_counts[c] < 3)
            .collect();

        let mut row2_cols: Vec<usize> = remaining_zero_cols;
        while row2_cols.len() < 5 && !remaining_available.is_empty() {
            let idx: usize = rng.next_index(remaining_available.len());
            let col: usize = remaining_available.swap_remove(idx);
            row2_cols.push(col);
        }

        if row2_cols.len() == 5 {
            for &col in &row2_cols {
                grid[2][col] = true;
                col_counts[col] += 1;
            }
            if col_counts.iter().all(|&c| (1..=3).contains(&c)) {
                return grid;
            }
        }
    }

    // Static fallback pattern (guaranteed valid: 5-5-5 rows, 1-2 per col, sum 15)
    [
        [true, true, true, true, true, false, false, false, false],
        [false, false, false, true, true, true, true, true, false],
        [true, false, false, false, false, true, true, true, true],
    ]
}

pub fn generate_ticket<R: RandomSource>(rng: &mut R) -> Result<Ticket, TicketError> {
    let layout: [[bool; 9]; 3] = generate_valid_layout(rng);
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
        // 6 numbers in row 0
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

        // Swap numbers in any column that has at least 2 numbers
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
