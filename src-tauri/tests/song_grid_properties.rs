use media_tambola_lib::domain::random::RandomSource;
use media_tambola_lib::domain::tambola::ticket_generator::{
    generate_song_grid_ticket, validate_song_grid_ticket, SongGridTicket,
};

struct StepRng {
    val: usize,
}

impl StepRng {
    fn new(start: usize) -> Self {
        Self { val: start }
    }
}

impl RandomSource for StepRng {
    fn next_index(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            return 0;
        }
        self.val = self.val.wrapping_mul(1103515245).wrapping_add(12345);
        self.val % bound
    }
}

#[test]
fn test_song_grid_properties_across_all_counts_15_to_90() {
    for song_count in 15..=90 {
        for trial in 0..5 {
            let mut rng: StepRng = StepRng::new((song_count as usize) * 100 + trial + 1);
            let ticket: SongGridTicket = generate_song_grid_ticket(song_count, &mut rng)
                .expect("Failed to generate song grid ticket");

            // 1. Must validate against domain rules
            validate_song_grid_ticket(&ticket, song_count)
                .expect("Validation failed for song grid ticket");

            // 2. Exactly 15 cells (3x5), all > 0 and <= song_count
            let mut seen_numbers: Vec<u8> = Vec::with_capacity(15);
            for row in 0..3 {
                for col in 0..5 {
                    let num: u8 = ticket.cells[row][col];
                    assert!(
                        num >= 1 && (num as u32) <= song_count,
                        "Number {} out of range [1, {}] in row {}, col {}",
                        num,
                        song_count,
                        row,
                        col
                    );
                    seen_numbers.push(num);
                }
            }

            assert_eq!(seen_numbers.len(), 15, "Expected 15 numbers in 3x5 grid");

            // 3. Zero duplicates
            seen_numbers.sort_unstable();
            let original_len: usize = seen_numbers.len();
            seen_numbers.dedup();
            assert_eq!(
                seen_numbers.len(),
                original_len,
                "Duplicate number found in song grid ticket for count {}",
                song_count
            );
        }
    }
}
