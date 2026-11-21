use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DrawOrderError {
    #[error("Item count {0} is out of bounds (must be between 15 and 90)")]
    InvalidCount(u32),
}

/// Generates a deterministic, seeded permutation of numbers `1..=count` using
/// Fisher-Yates shuffle with a 64-bit linear congruential generator.
/// Guarantees:
/// - Exactly `count` numbers.
/// - Every number from 1 to `count` appears exactly once (no repeats, no omissions).
/// - Deterministic output for any given seed.
pub fn generate_seeded_draw_order(count: u32, seed: u64) -> Result<Vec<u8>, DrawOrderError> {
    if !(15..=90).contains(&count) {
        return Err(DrawOrderError::InvalidCount(count));
    }

    let mut pool: Vec<u8> = (1..=(count as u8)).collect();
    let mut rng_state: u64 = seed;

    for i in (1..pool.len()).rev() {
        // Knuth 64-bit LCG
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j: usize = ((rng_state >> 32) as usize) % (i + 1);
        pool.swap(i, j);
    }

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_order_bounds_rejection() {
        assert_eq!(generate_seeded_draw_order(14, 42), Err(DrawOrderError::InvalidCount(14)));
        assert_eq!(generate_seeded_draw_order(91, 42), Err(DrawOrderError::InvalidCount(91)));
        assert_eq!(generate_seeded_draw_order(0, 42), Err(DrawOrderError::InvalidCount(0)));
    }

    #[test]
    fn test_draw_order_properties_all_counts_15_to_90() {
        for count in 15..=90 {
            let order = generate_seeded_draw_order(count, 12345).unwrap();
            assert_eq!(order.len(), count as usize);

            let mut sorted = order.clone();
            sorted.sort_unstable();

            let expected: Vec<u8> = (1..=(count as u8)).collect();
            assert_eq!(sorted, expected, "Permutation mismatch for count {}", count);
        }
    }

    #[test]
    fn test_draw_order_determinism() {
        let order1 = generate_seeded_draw_order(45, 9999).unwrap();
        let order2 = generate_seeded_draw_order(45, 9999).unwrap();
        assert_eq!(order1, order2);

        let order3 = generate_seeded_draw_order(45, 8888).unwrap();
        assert_ne!(order1, order3);
    }
}
