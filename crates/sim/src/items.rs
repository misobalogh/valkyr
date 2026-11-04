//! Item, set and race definitions. They are data: the catalog is filled by the
//! content loader, never by code.

use std::collections::BTreeMap;

use crate::stats::Modifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SetId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RaceId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EquipSlot {
    Head,
    Chest,
    Legs,
    Hands,
    Feet,
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemDef {
    pub id: ItemId,
    /// Weight in grams.
    pub weight: u32,
    /// `None` for items that cannot be equipped, such as raw materials.
    pub slot: Option<EquipSlot>,
    pub set: Option<SetId>,
    pub modifiers: Vec<Modifier>,
    /// Upgrade track: level `n` applies the first `n` steps.
    pub upgrades: Vec<Vec<Modifier>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetBonus {
    /// Number of equipped set pieces needed for the bonus.
    pub pieces: u8,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetDef {
    pub id: SetId,
    pub bonuses: Vec<SetBonus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RaceDef {
    pub id: RaceId,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Catalog {
    pub items: BTreeMap<ItemId, ItemDef>,
    pub sets: BTreeMap<SetId, SetDef>,
    pub races: BTreeMap<RaceId, RaceDef>,
}

/// One concrete item a player owns.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemInstance {
    pub item: ItemId,
    pub upgrade_level: u8,
    /// Rolled modifiers, see `affix`.
    pub affixes: Vec<Modifier>,
}
