//! What a character wears. Slots are validated against the catalog.

use std::collections::BTreeMap;
use std::fmt;

use crate::items::{Catalog, EquipSlot, ItemId, ItemInstance, RaceId};
use crate::stats::{Modifier, StatSheet, evaluate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadoutError {
    UnknownItem(ItemId),
    NotEquippable(ItemId),
    WrongSlot {
        item: ItemId,
        item_slot: EquipSlot,
        target: EquipSlot,
    },
    UnknownRace(RaceId),
}

impl fmt::Display for LoadoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "loadout error: {self:?}")
    }
}

impl std::error::Error for LoadoutError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Loadout {
    slots: BTreeMap<EquipSlot, ItemInstance>,
}

impl Loadout {
    pub fn get(&self, slot: EquipSlot) -> Option<&ItemInstance> {
        self.slots.get(&slot)
    }

    /// Equips the item and returns whatever it replaced.
    pub fn equip(
        &mut self,
        catalog: &Catalog,
        slot: EquipSlot,
        instance: ItemInstance,
    ) -> Result<Option<ItemInstance>, LoadoutError> {
        let item = instance.item;
        let def = catalog
            .items
            .get(&item)
            .ok_or(LoadoutError::UnknownItem(item))?;
        let item_slot = def.slot.ok_or(LoadoutError::NotEquippable(item))?;
        if item_slot != slot {
            return Err(LoadoutError::WrongSlot {
                item,
                item_slot,
                target: slot,
            });
        }
        Ok(self.slots.insert(slot, instance))
    }

    pub fn unequip(&mut self, slot: EquipSlot) -> Option<ItemInstance> {
        self.slots.remove(&slot)
    }

    /// Every modifier that applies to the character, in no particular order.
    pub fn modifiers(
        &self,
        catalog: &Catalog,
        race: RaceId,
    ) -> Result<Vec<Modifier>, LoadoutError> {
        let race_def = catalog
            .races
            .get(&race)
            .ok_or(LoadoutError::UnknownRace(race))?;
        let mut modifiers = race_def.modifiers.clone();
        for instance in self.slots.values() {
            let def = catalog
                .items
                .get(&instance.item)
                .ok_or(LoadoutError::UnknownItem(instance.item))?;
            modifiers.extend_from_slice(&def.modifiers);
            modifiers.extend_from_slice(&instance.affixes);
        }
        Ok(modifiers)
    }

    pub fn stats(
        &self,
        catalog: &Catalog,
        race: RaceId,
        base: &StatSheet,
    ) -> Result<StatSheet, LoadoutError> {
        Ok(evaluate(base, &self.modifiers(catalog, race)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{ItemDef, RaceDef};
    use crate::stats::{ModifierKind, StatId};

    const HELM: ItemId = ItemId(1);
    const ORE: ItemId = ItemId(2);
    const DWARF: RaceId = RaceId(1);
    const HP: StatId = StatId(1);

    fn flat(value: i64) -> Modifier {
        Modifier {
            stat: HP,
            kind: ModifierKind::Flat,
            value,
        }
    }

    fn catalog() -> Catalog {
        let mut catalog = Catalog::default();
        catalog.items.insert(
            HELM,
            ItemDef {
                id: HELM,
                slot: Some(EquipSlot::Head),
                modifiers: vec![flat(10)],
                ..ItemDef::default()
            },
        );
        catalog.races.insert(
            DWARF,
            RaceDef {
                id: DWARF,
                modifiers: vec![flat(100)],
            },
        );
        catalog.items.insert(
            ORE,
            ItemDef {
                id: ORE,
                ..ItemDef::default()
            },
        );
        catalog
    }

    fn instance(item: ItemId) -> ItemInstance {
        ItemInstance {
            item,
            ..ItemInstance::default()
        }
    }

    #[test]
    fn equip_fills_the_slot() {
        let mut loadout = Loadout::default();
        let replaced = loadout.equip(&catalog(), EquipSlot::Head, instance(HELM));
        assert_eq!(replaced, Ok(None));
        assert_eq!(loadout.get(EquipSlot::Head), Some(&instance(HELM)));
    }

    #[test]
    fn equip_returns_the_replaced_item() {
        let mut loadout = Loadout::default();
        let catalog = catalog();
        let first = ItemInstance {
            item: HELM,
            affixes: Vec::new(),
        };
        loadout
            .equip(&catalog, EquipSlot::Head, first.clone())
            .unwrap();
        let replaced = loadout.equip(&catalog, EquipSlot::Head, instance(HELM));
        assert_eq!(replaced, Ok(Some(first)));
    }

    #[test]
    fn wrong_slot_is_rejected() {
        let mut loadout = Loadout::default();
        let result = loadout.equip(&catalog(), EquipSlot::Feet, instance(HELM));
        assert_eq!(
            result,
            Err(LoadoutError::WrongSlot {
                item: HELM,
                item_slot: EquipSlot::Head,
                target: EquipSlot::Feet,
            })
        );
        assert_eq!(loadout.get(EquipSlot::Feet), None);
    }

    #[test]
    fn materials_and_unknown_items_are_rejected() {
        let mut loadout = Loadout::default();
        let catalog = catalog();
        assert_eq!(
            loadout.equip(&catalog, EquipSlot::Head, instance(ORE)),
            Err(LoadoutError::NotEquippable(ORE))
        );
        assert_eq!(
            loadout.equip(&catalog, EquipSlot::Head, instance(ItemId(9))),
            Err(LoadoutError::UnknownItem(ItemId(9)))
        );
    }

    #[test]
    fn unequip_empties_the_slot() {
        let mut loadout = Loadout::default();
        loadout
            .equip(&catalog(), EquipSlot::Head, instance(HELM))
            .unwrap();
        assert_eq!(loadout.unequip(EquipSlot::Head), Some(instance(HELM)));
        assert_eq!(loadout.get(EquipSlot::Head), None);
    }

    #[test]
    fn stats_combine_race_item_and_affixes() {
        let mut loadout = Loadout::default();
        let catalog = catalog();
        let rolled = ItemInstance {
            item: HELM,
            affixes: vec![flat(3)],
        };
        loadout.equip(&catalog, EquipSlot::Head, rolled).unwrap();
        let base = StatSheet::from([(HP, 1)]);
        let stats = loadout.stats(&catalog, DWARF, &base).unwrap();
        assert_eq!(stats[&HP], 1 + 100 + 10 + 3);
    }

    #[test]
    fn empty_loadout_still_applies_race() {
        let stats = Loadout::default()
            .stats(&catalog(), DWARF, &StatSheet::new())
            .unwrap();
        assert_eq!(stats[&HP], 100);
    }

    #[test]
    fn unknown_race_is_an_error() {
        let result = Loadout::default().modifiers(&catalog(), RaceId(7));
        assert_eq!(result, Err(LoadoutError::UnknownRace(RaceId(7))));
    }
}
