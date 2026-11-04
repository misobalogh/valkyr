//! Affixes: random modifiers rolled onto procedural loot.

use std::fmt;

use crate::rng::Rng;
use crate::stats::{Modifier, ModifierKind, StatId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffixDef {
    pub stat: StatId,
    pub kind: ModifierKind,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AffixError {
    PoolTooSmall { requested: usize, available: usize },
    InvalidRange { stat: StatId, min: i64, max: i64 },
}

impl fmt::Display for AffixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "affix error: {self:?}")
    }
}

impl std::error::Error for AffixError {}

/// Picks `count` distinct affixes from the pool and rolls a value for each.
pub fn roll_affixes(
    rng: &mut Rng,
    pool: &[AffixDef],
    count: usize,
) -> Result<Vec<Modifier>, AffixError> {
    if count > pool.len() {
        return Err(AffixError::PoolTooSmall {
            requested: count,
            available: pool.len(),
        });
    }
    let mut remaining: Vec<&AffixDef> = pool.iter().collect();
    let mut rolled = Vec::with_capacity(count);
    for _ in 0..count {
        let Some(index) = rng.next_index(remaining.len()) else {
            break;
        };
        let def = remaining.remove(index);
        let value = rng
            .next_in_range(def.min, def.max)
            .ok_or(AffixError::InvalidRange {
                stat: def.stat,
                min: def.min,
                max: def.max,
            })?;
        rolled.push(Modifier {
            stat: def.stat,
            kind: def.kind,
            value,
        });
    }
    Ok(rolled)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> Vec<AffixDef> {
        (1..=5)
            .map(|i| AffixDef {
                stat: StatId(i),
                kind: ModifierKind::Flat,
                min: 10,
                max: 20,
            })
            .collect()
    }

    #[test]
    fn rolls_distinct_affixes_within_range() {
        let mut rng = Rng::new(3);
        for _ in 0..50 {
            let rolled = roll_affixes(&mut rng, &pool(), 3).unwrap();
            let mut stats: Vec<StatId> = rolled.iter().map(|m| m.stat).collect();
            stats.sort();
            stats.dedup();
            assert_eq!(stats.len(), 3);
            assert!(rolled.iter().all(|m| (10..=20).contains(&m.value)));
        }
    }

    #[test]
    fn same_seed_rolls_the_same_affixes() {
        let a = roll_affixes(&mut Rng::new(8), &pool(), 4);
        let b = roll_affixes(&mut Rng::new(8), &pool(), 4);
        assert_eq!(a, b);
    }

    #[test]
    fn zero_count_rolls_nothing_and_keeps_rng() {
        let mut rng = Rng::new(8);
        assert_eq!(roll_affixes(&mut rng, &pool(), 0), Ok(Vec::new()));
        assert_eq!(rng, Rng::new(8));
    }

    #[test]
    fn asking_for_more_than_the_pool_is_an_error() {
        let result = roll_affixes(&mut Rng::new(1), &pool(), 6);
        assert_eq!(
            result,
            Err(AffixError::PoolTooSmall {
                requested: 6,
                available: 5
            })
        );
    }

    #[test]
    fn inverted_range_is_an_error() {
        let bad = [AffixDef {
            stat: StatId(1),
            kind: ModifierKind::Flat,
            min: 5,
            max: 1,
        }];
        let result = roll_affixes(&mut Rng::new(1), &bad, 1);
        assert_eq!(
            result,
            Err(AffixError::InvalidRange {
                stat: StatId(1),
                min: 5,
                max: 1
            })
        );
    }
}
