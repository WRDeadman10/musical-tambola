use media_tambola_lib::domain::random::RandomSource;
use media_tambola_lib::domain::tambola::ticket_generator::{
    col_max, col_min, generate_ticket, validate_ticket, Ticket,
};

struct LinearCongruentialRng {
    state: u64,
}

impl LinearCongruentialRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl RandomSource for LinearCongruentialRng {
    fn next_index(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            return 0;
        }
        // Knuth's 64-bit linear congruential generator constants
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as usize) % bound
    }
}

#[test]
fn test_classic_ticket_properties_over_1000_seeds() {
    for seed in 1..=1000 {
        let mut rng: LinearCongruentialRng = LinearCongruentialRng::new(seed as u64);
        let ticket: Ticket = generate_ticket(&mut rng).expect("Failed to generate ticket");

        // 1. Must pass full domain validator
        validate_ticket(&ticket).expect("Generated ticket failed validation");

        // 2. Exactly 15 numbers total
        let mut total_numbers: usize = 0;
        let mut numbers_seen: Vec<u8> = Vec::with_capacity(15);

        for row in 0..3 {
            let mut row_count: usize = 0;
            for col in 0..9 {
                if let Some(num) = ticket.cells[row][col] {
                    total_numbers += 1;
                    row_count += 1;
                    numbers_seen.push(num);

                    // 3. Number must strictly lie within column range
                    assert!(
                        num >= col_min(col) && num <= col_max(col),
                        "Number {} out of range [{}, {}] in col {} on seed {}",
                        num,
                        col_min(col),
                        col_max(col),
                        col,
                        seed
                    );
                }
            }
            // 4. Exactly 5 numbers per row
            assert_eq!(
                row_count, 5,
                "Row {} has {} numbers (expected 5) on seed {}",
                row, row_count, seed
            );
        }

        assert_eq!(
            total_numbers, 15,
            "Total numbers is {} (expected 15) on seed {}",
            total_numbers, seed
        );

        // 5. Each column has between 1 and 3 numbers and must be sorted strictly ascending
        for col in 0..9 {
            let mut col_nums: Vec<u8> = Vec::new();
            for row in 0..3 {
                if let Some(num) = ticket.cells[row][col] {
                    col_nums.push(num);
                }
            }
            assert!(
                !col_nums.is_empty() && col_nums.len() <= 3,
                "Col {} has {} numbers (expected 1..=3) on seed {}",
                col,
                col_nums.len(),
                seed
            );

            for i in 0..(col_nums.len().saturating_sub(1)) {
                assert!(
                    col_nums[i] < col_nums[i + 1],
                    "Col {} not sorted ascending: {:?} on seed {}",
                    col,
                    col_nums,
                    seed
                );
            }
        }

        // 6. No duplicates anywhere on the ticket
        numbers_seen.sort_unstable();
        let original_len: usize = numbers_seen.len();
        numbers_seen.dedup();
        assert_eq!(
            numbers_seen.len(),
            original_len,
            "Duplicate number found on ticket on seed {}",
            seed
        );
    }
}
