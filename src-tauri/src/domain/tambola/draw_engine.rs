use crate::domain::random::RandomSource;

/// Generates a complete 90-ball Tambola draw sequence using Fisher-Yates shuffle.
/// Each number 1..=90 appears exactly once with zero modulo bias.
pub fn generate_draw_sequence(rng: &mut impl RandomSource) -> [u8; 90] {
    let mut seq: [u8; 90] = [0u8; 90];
    for (i, val) in (1u8..=90u8).enumerate() {
        seq[i] = val;
    }

    // Fisher-Yates shuffle from 89 down to 1
    for i in (1..90).rev() {
        let j: usize = rng.next_index(i + 1);
        seq.swap(i, j);
    }

    seq
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::random::DeterministicRandomSource;
    use std::collections::HashSet;

    #[test]
    fn test_draw_sequence_all_numbers_1_to_90_exactly_once() {
        let mut rng: DeterministicRandomSource = DeterministicRandomSource::new(42);
        let seq: [u8; 90] = generate_draw_sequence(&mut rng);

        assert_eq!(seq.len(), 90);
        let mut seen: HashSet<u8> = HashSet::new();
        for &num in seq.iter() {
            assert!((1..=90).contains(&num), "Number out of range: {}", num);
            assert!(
                seen.insert(num),
                "Duplicate number in draw sequence: {}",
                num
            );
        }
        assert_eq!(seen.len(), 90);
    }

    #[test]
    fn test_draw_sequence_deterministic_with_seeded_source() {
        let mut rng1: DeterministicRandomSource = DeterministicRandomSource::new(12345);
        let seq1: [u8; 90] = generate_draw_sequence(&mut rng1);

        let mut rng2: DeterministicRandomSource = DeterministicRandomSource::new(12345);
        let seq2: [u8; 90] = generate_draw_sequence(&mut rng2);

        assert_eq!(seq1, seq2, "Same seed must produce identical draw sequence");
    }

    #[test]
    fn test_draw_sequence_distinct_across_different_seeds() {
        let mut rng1: DeterministicRandomSource = DeterministicRandomSource::new(101);
        let seq1: [u8; 90] = generate_draw_sequence(&mut rng1);

        let mut rng2: DeterministicRandomSource = DeterministicRandomSource::new(202);
        let seq2: [u8; 90] = generate_draw_sequence(&mut rng2);

        assert_ne!(
            seq1, seq2,
            "Different seeds should produce different draw sequences"
        );
    }
}
