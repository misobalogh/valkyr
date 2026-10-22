//! Seeded random numbers (SplitMix64). The only source of randomness in sim.

use std::num::NonZeroU64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    pub fn state(&self) -> u64 {
        self.state
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform value in `0..bound`, without modulo bias.
    pub fn next_below(&mut self, bound: NonZeroU64) -> u64 {
        let bound = bound.get();
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return value % bound;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_reference_splitmix64_for_seed_zero() {
        let mut rng = Rng::new(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn same_seed_gives_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn next_below_stays_in_range() {
        let mut rng = Rng::new(7);
        let bound = NonZeroU64::new(10).unwrap();
        for _ in 0..1000 {
            assert!(rng.next_below(bound) < 10);
        }
    }

    #[test]
    fn next_below_one_is_always_zero() {
        let mut rng = Rng::new(7);
        let bound = NonZeroU64::new(1).unwrap();
        assert_eq!(rng.next_below(bound), 0);
    }
}
