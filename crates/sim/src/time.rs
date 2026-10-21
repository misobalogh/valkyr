//! Simulation time. Counted in ticks only, never in wall-clock time.

pub const TICKS_PER_SECOND: u32 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tick(pub u64);

impl Tick {
    pub const ZERO: Tick = Tick(0);

    pub fn next(self) -> Tick {
        Tick(self.0.saturating_add(1))
    }

    pub fn from_seconds(seconds: u32) -> Tick {
        Tick(u64::from(seconds) * u64::from(TICKS_PER_SECOND))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_advances_by_one() {
        assert_eq!(Tick::ZERO.next(), Tick(1));
    }

    #[test]
    fn next_saturates_at_max() {
        assert_eq!(Tick(u64::MAX).next(), Tick(u64::MAX));
    }

    #[test]
    fn from_seconds_uses_tick_rate() {
        assert_eq!(Tick::from_seconds(2), Tick(2 * u64::from(TICKS_PER_SECOND)));
    }
}
