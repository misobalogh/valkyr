//! Damage-over-time status effects. One effect per damage type; reapplying
//! refreshes it to the stronger values.

use std::collections::BTreeMap;

use crate::damage::{DamageType, Vitals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusEffect {
    pub damage_type: DamageType,
    pub damage_per_tick: i64,
    pub ticks_left: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusSet {
    effects: BTreeMap<DamageType, StatusEffect>,
}

impl StatusSet {
    pub fn get(&self, damage_type: DamageType) -> Option<&StatusEffect> {
        self.effects.get(&damage_type)
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn apply(&mut self, effect: StatusEffect) {
        if effect.ticks_left == 0 {
            return;
        }
        self.effects
            .entry(effect.damage_type)
            .and_modify(|current| {
                current.damage_per_tick = current.damage_per_tick.max(effect.damage_per_tick);
                current.ticks_left = current.ticks_left.max(effect.ticks_left);
            })
            .or_insert(effect);
    }

    /// Deals one tick of every effect through the target's resistances and
    /// returns the total health removed.
    pub fn tick(&mut self, vitals: &mut Vitals) -> i64 {
        let mut total = 0_i64;
        for effect in self.effects.values_mut() {
            total = total
                .saturating_add(vitals.take_damage(effect.damage_type, effect.damage_per_tick));
            effect.ticks_left -= 1;
        }
        self.effects.retain(|_, effect| effect.ticks_left > 0);
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn burn(damage_per_tick: i64, ticks_left: u32) -> StatusEffect {
        StatusEffect {
            damage_type: DamageType::Fire,
            damage_per_tick,
            ticks_left,
        }
    }

    #[test]
    fn effect_deals_damage_each_tick_then_expires() {
        let mut set = StatusSet::default();
        let mut vitals = Vitals::new(100);
        set.apply(burn(5, 3));
        let dealt: Vec<i64> = (0..4).map(|_| set.tick(&mut vitals)).collect();
        assert_eq!(dealt, vec![5, 5, 5, 0]);
        assert!(set.is_empty());
        assert_eq!(vitals.health, 85);
    }

    #[test]
    fn reapplying_keeps_the_stronger_values() {
        let mut set = StatusSet::default();
        set.apply(burn(5, 10));
        set.apply(burn(8, 4));
        assert_eq!(set.get(DamageType::Fire), Some(&burn(8, 10)));
    }

    #[test]
    fn effects_respect_resistances() {
        let mut set = StatusSet::default();
        let mut vitals = Vitals::new(100);
        vitals.resistances.insert(DamageType::Fire, 500);
        set.apply(burn(10, 1));
        assert_eq!(set.tick(&mut vitals), 5);
    }

    #[test]
    fn different_types_stack_independently() {
        let mut set = StatusSet::default();
        let mut vitals = Vitals::new(100);
        set.apply(burn(4, 2));
        set.apply(StatusEffect {
            damage_type: DamageType::Frost,
            damage_per_tick: 3,
            ticks_left: 2,
        });
        assert_eq!(set.tick(&mut vitals), 7);
    }

    #[test]
    fn zero_duration_effects_are_ignored() {
        let mut set = StatusSet::default();
        set.apply(burn(5, 0));
        assert!(set.is_empty());
    }
}
