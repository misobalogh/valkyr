//! What a character wears. Slots are validated against the catalog.

use std::collections::BTreeMap;
use std::fmt;

use crate::items::{Catalog, EquipSlot, ItemId, ItemInstance, RaceId, SetId};
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
    UnknownSet(SetId),
    UpgradeBeyondTrack {
        item: ItemId,
        level: u8,
        max: usize,
    },
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
        let mut set_pieces: BTreeMap<SetId, u8> = BTreeMap::new();
        for instance in self.slots.values() {
            let def = catalog
                .items
                .get(&instance.item)
                .ok_or(LoadoutError::UnknownItem(instance.item))?;
            modifiers.extend_from_slice(&def.modifiers);
            if let Some(set) = def.set {
                let count = set_pieces.entry(set).or_insert(0);
                *count = count.saturating_add(1);
            }
            let steps = def
                .upgrades
                .get(..usize::from(instance.upgrade_level))
                .ok_or(LoadoutError::UpgradeBeyondTrack {
                    item: instance.item,
                    level: instance.upgrade_level,
                    max: def.upgrades.len(),
                })?;
            for step in steps {
                modifiers.extend_from_slice(step);
            }
            modifiers.extend_from_slice(&instance.affixes);
        }
        for (set_id, count) in set_pieces {
            let set = catalog
                .sets
                .get(&set_id)
                .ok_or(LoadoutError::UnknownSet(set_id))?;
            for bonus in set.bonuses.iter().filter(|b| b.pieces <= count) {
                modifiers.extend_from_slice(&bonus.modifiers);
            }
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
    use crate::items::{ItemDef, RaceDef, SetBonus, SetDef};
    use crate::stats::{ModifierKind, StatId};

    const HELM: ItemId = ItemId(1);
    const ORE: ItemId = ItemId(2);
    const CHEST: ItemId = ItemId(3);
    const DWARF: RaceId = RaceId(1);
    const SET: SetId = SetId(1);
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
                set: Some(SET),
                modifiers: vec![flat(10)],
                upgrades: vec![vec![flat(1)], vec![flat(2)]],
                ..ItemDef::default()
            },
        );
        catalog.items.insert(
            CHEST,
            ItemDef {
                id: CHEST,
                slot: Some(EquipSlot::Chest),
                set: Some(SET),
                ..ItemDef::default()
            },
        );
        catalog.sets.insert(
            SET,
            SetDef {
                id: SET,
                bonuses: vec![SetBonus {
                    pieces: 2,
                    modifiers: vec![flat(50)],
                }],
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
            upgrade_level: 0,
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
            ..ItemInstance::default()
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

    fn upgraded(level: u8) -> ItemInstance {
        ItemInstance {
            item: HELM,
            upgrade_level: level,
            ..ItemInstance::default()
        }
    }

    fn hp_with_helm_level(level: u8) -> Result<i64, LoadoutError> {
        let mut loadout = Loadout::default();
        let catalog = catalog();
        loadout.equip(&catalog, EquipSlot::Head, upgraded(level))?;
        let stats = loadout.stats(&catalog, DWARF, &StatSheet::new())?;
        Ok(stats[&HP])
    }

    #[test]
    fn upgrade_levels_apply_the_first_n_steps() {
        assert_eq!(hp_with_helm_level(0), Ok(110));
        assert_eq!(hp_with_helm_level(1), Ok(111));
        assert_eq!(hp_with_helm_level(2), Ok(113));
    }

    #[test]
    fn upgrade_beyond_the_track_is_an_error() {
        assert_eq!(
            hp_with_helm_level(3),
            Err(LoadoutError::UpgradeBeyondTrack {
                item: HELM,
                level: 3,
                max: 2,
            })
        );
    }

    #[test]
    fn set_bonus_needs_enough_pieces() {
        let mut loadout = Loadout::default();
        let catalog = catalog();
        loadout
            .equip(&catalog, EquipSlot::Head, upgraded(0))
            .unwrap();
        let one_piece = loadout.stats(&catalog, DWARF, &StatSheet::new()).unwrap();
        loadout
            .equip(&catalog, EquipSlot::Chest, instance(CHEST))
            .unwrap();
        let two_pieces = loadout.stats(&catalog, DWARF, &StatSheet::new()).unwrap();
        assert_eq!(one_piece[&HP], 110);
        assert_eq!(two_pieces[&HP], 160);
    }

    #[test]
    fn item_with_unknown_set_is_an_error() {
        let mut catalog = catalog();
        catalog.sets.clear();
        let mut loadout = Loadout::default();
        loadout
            .equip(&catalog, EquipSlot::Head, upgraded(0))
            .unwrap();
        assert_eq!(
            loadout.modifiers(&catalog, DWARF),
            Err(LoadoutError::UnknownSet(SET))
        );
    }
}
