//! Poise and stagger. Poise damage that empties the bar staggers the target.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Poise {
    pub current: i64,
    pub max: i64,
    /// Regained per tick while not staggered.
    pub regen: i64,
    pub stagger_ticks_left: u32,
}

impl Poise {
    pub fn new(max: i64, regen: i64) -> Poise {
        Poise {
            current: max,
            max,
            regen,
            stagger_ticks_left: 0,
        }
    }

    pub fn is_staggered(&self) -> bool {
        self.stagger_ticks_left > 0
    }

    /// Returns true when this hit started a stagger. Hits on a staggered
    /// target do nothing, so a stagger cannot be chained forever.
    pub fn absorb(&mut self, poise_damage: i64, stagger_ticks: u32) -> bool {
        if self.is_staggered() {
            return false;
        }
        self.current = self.current.saturating_sub(poise_damage.max(0));
        if self.current > 0 {
            return false;
        }
        self.current = self.max;
        self.stagger_ticks_left = stagger_ticks;
        stagger_ticks > 0
    }

    pub fn tick(&mut self) {
        if self.is_staggered() {
            self.stagger_ticks_left -= 1;
        } else {
            self.current = self.current.saturating_add(self.regen).min(self.max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_hits_do_not_stagger() {
        let mut poise = Poise::new(100, 1);
        assert!(!poise.absorb(40, 30));
        assert_eq!(poise.current, 60);
    }

    #[test]
    fn emptying_the_bar_staggers_and_refills_it() {
        let mut poise = Poise::new(100, 1);
        poise.absorb(60, 30);
        assert!(poise.absorb(40, 30));
        assert!(poise.is_staggered());
        assert_eq!(poise.current, 100);
    }

    #[test]
    fn staggered_targets_ignore_further_poise_damage() {
        let mut poise = Poise::new(100, 1);
        poise.absorb(100, 5);
        assert!(!poise.absorb(100, 5));
        assert_eq!(poise.current, 100);
        assert_eq!(poise.stagger_ticks_left, 5);
    }

    #[test]
    fn stagger_wears_off_tick_by_tick() {
        let mut poise = Poise::new(100, 1);
        poise.absorb(100, 3);
        for _ in 0..3 {
            assert!(poise.is_staggered());
            poise.tick();
        }
        assert!(!poise.is_staggered());
    }

    #[test]
    fn poise_regenerates_up_to_max_when_not_staggered() {
        let mut poise = Poise::new(100, 4);
        poise.absorb(10, 30);
        poise.tick();
        poise.tick();
        poise.tick();
        assert_eq!(poise.current, 100);
    }

    #[test]
    fn zero_length_stagger_never_staggers() {
        let mut poise = Poise::new(10, 1);
        assert!(!poise.absorb(10, 0));
        assert!(!poise.is_staggered());
    }
}
