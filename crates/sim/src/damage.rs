//! Damage types, resistances and health.

use std::collections::BTreeMap;

const PERMILLE: i64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DamageType {
    Physical,
    Fire,
    Frost,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vitals {
    pub health: i64,
    pub max_health: i64,
    /// Permille per damage type. 1000 is immune, negative is vulnerable.
    pub resistances: BTreeMap<DamageType, i64>,
}

impl Vitals {
    pub fn new(max_health: i64) -> Vitals {
        Vitals {
            health: max_health,
            max_health,
            resistances: BTreeMap::new(),
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }

    /// Applies resisted damage and returns how much health was removed.
    /// Negative amounts do nothing; overkill is not counted.
    pub fn take_damage(&mut self, kind: DamageType, amount: i64) -> i64 {
        let resist = self
            .resistances
            .get(&kind)
            .copied()
            .unwrap_or(0)
            .min(PERMILLE);
        let scaled =
            i128::from(amount.max(0)) * i128::from(PERMILLE - resist) / i128::from(PERMILLE);
        let wanted = i64::try_from(scaled).unwrap_or(i64::MAX);
        let dealt = wanted.min(self.health.max(0));
        self.health -= dealt;
        dealt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresisted_damage_is_applied_in_full() {
        let mut vitals = Vitals::new(100);
        assert_eq!(vitals.take_damage(DamageType::Physical, 30), 30);
        assert_eq!(vitals.health, 70);
    }

    #[test]
    fn resistance_scales_damage_per_type() {
        let mut vitals = Vitals::new(1_000);
        vitals.resistances.insert(DamageType::Frost, 750);
        assert_eq!(vitals.take_damage(DamageType::Frost, 100), 25);
        assert_eq!(vitals.take_damage(DamageType::Fire, 100), 100);
    }

    #[test]
    fn vulnerability_increases_damage() {
        let mut vitals = Vitals::new(1_000);
        vitals.resistances.insert(DamageType::Fire, -500);
        assert_eq!(vitals.take_damage(DamageType::Fire, 100), 150);
    }

    #[test]
    fn resistance_above_immune_is_capped() {
        let mut vitals = Vitals::new(100);
        vitals.resistances.insert(DamageType::Dark, 5_000);
        assert_eq!(vitals.take_damage(DamageType::Dark, 100), 0);
        assert_eq!(vitals.health, 100);
    }

    #[test]
    fn damage_stops_at_zero_health() {
        let mut vitals = Vitals::new(50);
        assert_eq!(vitals.take_damage(DamageType::Physical, 80), 50);
        assert!(vitals.is_dead());
        assert_eq!(vitals.take_damage(DamageType::Physical, 80), 0);
        assert_eq!(vitals.health, 0);
    }

    #[test]
    fn negative_damage_does_not_heal() {
        let mut vitals = Vitals::new(50);
        vitals.health = 10;
        assert_eq!(vitals.take_damage(DamageType::Physical, -40), 0);
        assert_eq!(vitals.health, 10);
    }
}
