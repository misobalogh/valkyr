//! Inventory with a weight limit. Stacks are keyed by item, so iteration order
//! is deterministic.

use std::collections::BTreeMap;
use std::fmt;

use crate::items::{Catalog, ItemId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryError {
    UnknownItem(ItemId),
    OverWeight {
        item: ItemId,
        would_weigh: u64,
        limit: u64,
    },
    NotEnough {
        item: ItemId,
        have: u32,
        want: u32,
    },
    CountOverflow(ItemId),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "inventory error: {self:?}")
    }
}

impl std::error::Error for InventoryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    stacks: BTreeMap<ItemId, u32>,
    /// In grams.
    weight_limit: u64,
}

impl Inventory {
    pub fn new(weight_limit: u64) -> Inventory {
        Inventory {
            stacks: BTreeMap::new(),
            weight_limit,
        }
    }

    pub fn count(&self, item: ItemId) -> u32 {
        self.stacks.get(&item).copied().unwrap_or(0)
    }

    pub fn weight(&self, catalog: &Catalog) -> Result<u64, InventoryError> {
        self.stacks.iter().try_fold(0_u64, |total, (item, count)| {
            let def = catalog
                .items
                .get(item)
                .ok_or(InventoryError::UnknownItem(*item))?;
            Ok(total.saturating_add(u64::from(def.weight) * u64::from(*count)))
        })
    }

    pub fn add(
        &mut self,
        catalog: &Catalog,
        item: ItemId,
        count: u32,
    ) -> Result<(), InventoryError> {
        let def = catalog
            .items
            .get(&item)
            .ok_or(InventoryError::UnknownItem(item))?;
        let total = self
            .count(item)
            .checked_add(count)
            .ok_or(InventoryError::CountOverflow(item))?;
        let added = u64::from(def.weight) * u64::from(count);
        let would_weigh = self.weight(catalog)?.saturating_add(added);
        if would_weigh > self.weight_limit {
            return Err(InventoryError::OverWeight {
                item,
                would_weigh,
                limit: self.weight_limit,
            });
        }
        if count > 0 {
            self.stacks.insert(item, total);
        }
        Ok(())
    }

    pub fn remove(&mut self, item: ItemId, count: u32) -> Result<(), InventoryError> {
        let have = self.count(item);
        let left = have.checked_sub(count).ok_or(InventoryError::NotEnough {
            item,
            have,
            want: count,
        })?;
        if left == 0 {
            self.stacks.remove(&item);
        } else {
            self.stacks.insert(item, left);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::ItemDef;

    const LOG: ItemId = ItemId(1);

    fn catalog() -> Catalog {
        let mut catalog = Catalog::default();
        catalog.items.insert(
            LOG,
            ItemDef {
                id: LOG,
                weight: 500,
                ..ItemDef::default()
            },
        );
        catalog
    }

    #[test]
    fn add_tracks_count_and_weight() {
        let mut inventory = Inventory::new(10_000);
        let catalog = catalog();
        inventory.add(&catalog, LOG, 4).unwrap();
        assert_eq!(inventory.count(LOG), 4);
        assert_eq!(inventory.weight(&catalog), Ok(2_000));
    }

    #[test]
    fn add_over_the_limit_changes_nothing() {
        let mut inventory = Inventory::new(1_000);
        let catalog = catalog();
        inventory.add(&catalog, LOG, 2).unwrap();
        assert_eq!(
            inventory.add(&catalog, LOG, 1),
            Err(InventoryError::OverWeight {
                item: LOG,
                would_weigh: 1_500,
                limit: 1_000
            })
        );
        assert_eq!(inventory.count(LOG), 2);
    }

    #[test]
    fn exactly_at_the_limit_is_allowed() {
        let mut inventory = Inventory::new(1_000);
        assert_eq!(inventory.add(&catalog(), LOG, 2), Ok(()));
    }

    #[test]
    fn remove_more_than_owned_fails() {
        let mut inventory = Inventory::new(10_000);
        inventory.add(&catalog(), LOG, 1).unwrap();
        assert_eq!(
            inventory.remove(LOG, 2),
            Err(InventoryError::NotEnough {
                item: LOG,
                have: 1,
                want: 2
            })
        );
    }

    #[test]
    fn removing_everything_clears_the_stack() {
        let mut inventory = Inventory::new(10_000);
        let catalog = catalog();
        inventory.add(&catalog, LOG, 3).unwrap();
        inventory.remove(LOG, 3).unwrap();
        assert_eq!(inventory, Inventory::new(10_000));
    }

    #[test]
    fn unknown_items_are_rejected() {
        let mut inventory = Inventory::new(10_000);
        assert_eq!(
            inventory.add(&catalog(), ItemId(9), 1),
            Err(InventoryError::UnknownItem(ItemId(9)))
        );
    }

    #[test]
    fn count_overflow_is_an_error() {
        let mut inventory = Inventory::new(u64::MAX);
        let catalog = catalog();
        inventory.add(&catalog, LOG, u32::MAX).unwrap();
        assert_eq!(
            inventory.add(&catalog, LOG, 1),
            Err(InventoryError::CountOverflow(LOG))
        );
    }
}
