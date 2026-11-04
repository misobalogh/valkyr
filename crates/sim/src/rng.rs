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

    /// Uniform index in `0..len`, or `None` when `len` is zero.
    pub fn next_index(&mut self, len: usize) -> Option<usize> {
        let bound = NonZeroU64::new(u64::try_from(len).ok()?)?;
        usize::try_from(self.next_below(bound)).ok()
    }

    /// Uniform value in `min..=max`, or `None` when `min > max` or the range
    /// is wider than `u64::MAX`.
    pub fn next_in_range(&mut self, min: i64, max: i64) -> Option<i64> {
        let span = i128::from(max) - i128::from(min) + 1;
        let bound = NonZeroU64::new(u64::try_from(span).ok()?)?;
        let offset = self.next_below(bound);
        i64::try_from(i128::from(min) + i128::from(offset)).ok()
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

    #[test]
    fn next_index_is_none_for_empty() {
        assert_eq!(Rng::new(1).next_index(0), None);
    }

    #[test]
    fn next_index_stays_in_bounds() {
        let mut rng = Rng::new(1);
        for _ in 0..500 {
            assert!(rng.next_index(7).is_some_and(|i| i < 7));
        }
    }

    #[test]
    fn next_in_range_includes_both_ends() {
        let mut rng = Rng::new(2);
        let values: Vec<i64> = (0..200).filter_map(|_| rng.next_in_range(-1, 1)).collect();
        assert!(values.contains(&-1) && values.contains(&0) && values.contains(&1));
        assert!(values.iter().all(|v| (-1..=1).contains(v)));
    }

    #[test]
    fn next_in_range_rejects_inverted_and_huge_ranges() {
        let mut rng = Rng::new(2);
        assert_eq!(rng.next_in_range(5, 4), None);
        assert_eq!(rng.next_in_range(i64::MIN, i64::MAX), None);
    }
}
