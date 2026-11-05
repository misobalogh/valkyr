//! Loot tables: weighted item picks plus a rarity tier that decides how many
//! affixes the drop gets. All randomness comes from the passed-in `Rng`.

use std::fmt;
use std::num::NonZeroU64;

use crate::affix::{AffixDef, AffixError, roll_affixes};
use crate::items::ItemId;
use crate::rng::Rng;
use crate::stats::Modifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootEntry {
    pub item: ItemId,
    pub weight: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RarityTier {
    pub rarity: Rarity,
    pub weight: u32,
    pub affix_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootTable {
    pub entries: Vec<LootEntry>,
    pub tiers: Vec<RarityTier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drop {
    pub item: ItemId,
    pub rarity: Rarity,
    pub affixes: Vec<Modifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootError {
    NoEntries,
    NoTiers,
    ZeroTotalWeight,
    Affix(AffixError),
}

impl From<AffixError> for LootError {
    fn from(error: AffixError) -> LootError {
        LootError::Affix(error)
    }
}

impl fmt::Display for LootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "loot error: {self:?}")
    }
}

impl std::error::Error for LootError {}

impl LootTable {
    pub fn roll(&self, rng: &mut Rng, affix_pool: &[AffixDef]) -> Result<Drop, LootError> {
        if self.entries.is_empty() {
            return Err(LootError::NoEntries);
        }
        if self.tiers.is_empty() {
            return Err(LootError::NoTiers);
        }
        let entry =
            pick_weighted(rng, &self.entries, |e| e.weight).ok_or(LootError::ZeroTotalWeight)?;
        let tier =
            pick_weighted(rng, &self.tiers, |t| t.weight).ok_or(LootError::ZeroTotalWeight)?;
        let affixes = roll_affixes(rng, affix_pool, tier.affix_count)?;
        Ok(Drop {
            item: entry.item,
            rarity: tier.rarity,
            affixes,
        })
    }
}

fn pick_weighted<'a, T>(
    rng: &mut Rng,
    items: &'a [T],
    weight: impl Fn(&T) -> u32,
) -> Option<&'a T> {
    let total: u64 = items.iter().map(|item| u64::from(weight(item))).sum();
    let mut roll = rng.next_below(NonZeroU64::new(total)?);
    for item in items {
        let w = u64::from(weight(item));
        if roll < w {
            return Some(item);
        }
        roll -= w;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::{ModifierKind, StatId};

    fn table() -> LootTable {
        LootTable {
            entries: vec![
                LootEntry {
                    item: ItemId(1),
                    weight: 3,
                },
                LootEntry {
                    item: ItemId(2),
                    weight: 1,
                },
                LootEntry {
                    item: ItemId(3),
                    weight: 0,
                },
            ],
            tiers: vec![
                RarityTier {
                    rarity: Rarity::Common,
                    weight: 9,
                    affix_count: 0,
                },
                RarityTier {
                    rarity: Rarity::Rare,
                    weight: 1,
                    affix_count: 2,
                },
            ],
        }
    }

    fn pool() -> Vec<AffixDef> {
        (1..=3)
            .map(|i| AffixDef {
                stat: StatId(i),
                kind: ModifierKind::Flat,
                min: 1,
                max: 9,
            })
            .collect()
    }

    #[test]
    fn same_seed_gives_same_drops() {
        let roll_all = |seed| {
            let mut rng = Rng::new(seed);
            (0..50)
                .map(|_| table().roll(&mut rng, &pool()))
                .collect::<Vec<_>>()
        };
        assert_eq!(roll_all(4), roll_all(4));
    }

    #[test]
    fn zero_weight_entries_never_drop() {
        let mut rng = Rng::new(1);
        for _ in 0..2_000 {
            let drop = table().roll(&mut rng, &pool()).unwrap();
            assert_ne!(drop.item, ItemId(3));
        }
    }

    #[test]
    fn weights_shape_the_distribution() {
        let mut rng = Rng::new(1);
        let drops: Vec<Drop> = (0..4_000)
            .map(|_| table().roll(&mut rng, &pool()).unwrap())
            .collect();
        let heavy = drops.iter().filter(|d| d.item == ItemId(1)).count();
        let rare = drops.iter().filter(|d| d.rarity == Rarity::Rare).count();
        assert!((2_800..3_200).contains(&heavy), "heavy = {heavy}");
        assert!((250..550).contains(&rare), "rare = {rare}");
    }

    #[test]
    fn tier_decides_affix_count() {
        let mut rng = Rng::new(9);
        for _ in 0..300 {
            let drop = table().roll(&mut rng, &pool()).unwrap();
            let expected = if drop.rarity == Rarity::Rare { 2 } else { 0 };
            assert_eq!(drop.affixes.len(), expected);
        }
    }

    #[test]
    fn broken_tables_are_errors() {
        let mut rng = Rng::new(1);
        let mut empty = table();
        empty.entries.clear();
        assert_eq!(empty.roll(&mut rng, &pool()), Err(LootError::NoEntries));
        let mut no_tiers = table();
        no_tiers.tiers.clear();
        assert_eq!(no_tiers.roll(&mut rng, &pool()), Err(LootError::NoTiers));
        let mut zero = table();
        zero.entries.iter_mut().for_each(|e| e.weight = 0);
        assert_eq!(
            zero.roll(&mut rng, &pool()),
            Err(LootError::ZeroTotalWeight)
        );
    }

    #[test]
    fn affix_pool_errors_are_propagated() {
        let mut rng = Rng::new(1);
        let only_rare = LootTable {
            entries: table().entries,
            tiers: vec![RarityTier {
                rarity: Rarity::Rare,
                weight: 1,
                affix_count: 5,
            }],
        };
        assert!(matches!(
            only_rare.roll(&mut rng, &pool()),
            Err(LootError::Affix(AffixError::PoolTooSmall { .. }))
        ));
    }
}
