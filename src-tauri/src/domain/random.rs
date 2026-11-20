use rand::rngs::OsRng;
use rand::RngCore;

pub trait RandomSource: Send {
    /// Returns a random index in the range [0, upper).
    /// Precondition: upper > 0.
    fn next_index(&mut self, upper: usize) -> usize;
}

pub struct OsRngSource;

impl OsRngSource {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OsRngSource {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSource for OsRngSource {
    fn next_index(&mut self, upper: usize) -> usize {
        assert!(upper > 0, "upper bound must be greater than 0");
        let mut bytes: [u8; 8] = [0u8; 8];
        OsRng.fill_bytes(&mut bytes);
        let val: u64 = u64::from_le_bytes(bytes);
        (val % (upper as u64)) as usize
    }
}

/// A deterministic pseudo-random source for tests and reproducible sequences.
pub struct DeterministicRandomSource {
    state: u64,
}

impl DeterministicRandomSource {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }
}

impl RandomSource for DeterministicRandomSource {
    fn next_index(&mut self, upper: usize) -> usize {
        assert!(upper > 0, "upper bound must be greater than 0");
        let mut x: u64 = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x % (upper as u64)) as usize
    }
}

/// A mock random source that returns constant indices, useful for testing collision handling and retry limits.
pub struct ConstantRandomSource {
    value: usize,
}

impl ConstantRandomSource {
    pub fn new(value: usize) -> Self {
        Self { value }
    }
}

impl RandomSource for ConstantRandomSource {
    fn next_index(&mut self, upper: usize) -> usize {
        assert!(upper > 0, "upper bound must be greater than 0");
        self.value % upper
    }
}
