//! Modifier pipeline. Evaluation order is fixed and independent of the order
//! in which modifiers are collected:
//!
//! 1. base value plus all `Flat` modifiers
//! 2. times (1 + sum of all `PercentAdd`)
//! 3. times (1 + each `PercentMult`), applied in sorted order
//!
//! Percentages are in permille (1000 = 100%). Results truncate toward zero and
//! saturate instead of overflowing.

use std::collections::{BTreeMap, BTreeSet};

const PERMILLE: i64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatId(pub u16);

/// Declaration order is evaluation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModifierKind {
    Flat,
    PercentAdd,
    PercentMult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Modifier {
    pub stat: StatId,
    pub kind: ModifierKind,
    pub value: i64,
}

pub type StatSheet = BTreeMap<StatId, i64>;

pub fn evaluate(base: &StatSheet, modifiers: &[Modifier]) -> StatSheet {
    let mut sorted = modifiers.to_vec();
    sorted.sort();
    let stats: BTreeSet<StatId> = base
        .keys()
        .copied()
        .chain(sorted.iter().map(|m| m.stat))
        .collect();
    stats
        .into_iter()
        .map(|stat| {
            let of_stat: Vec<Modifier> =
                sorted.iter().copied().filter(|m| m.stat == stat).collect();
            let start = base.get(&stat).copied().unwrap_or(0);
            (stat, evaluate_stat(start, &of_stat))
        })
        .collect()
}

fn evaluate_stat(base: i64, sorted: &[Modifier]) -> i64 {
    let of_kind = |kind| sorted.iter().filter(move |m| m.kind == kind);
    let flat = of_kind(ModifierKind::Flat).fold(base, |acc, m| acc.saturating_add(m.value));
    let percent_add =
        of_kind(ModifierKind::PercentAdd).fold(0, |acc: i64, m| acc.saturating_add(m.value));
    let mut value = scale(flat, percent_add);
    for modifier in of_kind(ModifierKind::PercentMult) {
        value = scale(value, modifier.value);
    }
    value
}

fn scale(value: i64, percent: i64) -> i64 {
    let factor = PERMILLE.saturating_add(percent).max(0);
    let scaled = i128::from(value) * i128::from(factor) / i128::from(PERMILLE);
    i64::try_from(scaled).unwrap_or(if scaled < 0 { i64::MIN } else { i64::MAX })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HP: StatId = StatId(1);
    const SPEED: StatId = StatId(2);

    fn modifier(stat: StatId, kind: ModifierKind, value: i64) -> Modifier {
        Modifier { stat, kind, value }
    }

    fn sheet(stat: StatId, value: i64) -> StatSheet {
        StatSheet::from([(stat, value)])
    }

    #[test]
    fn flat_then_percent_add_then_percent_mult() {
        let mods = [
            modifier(HP, ModifierKind::Flat, 20),
            modifier(HP, ModifierKind::PercentAdd, 500),
            modifier(HP, ModifierKind::PercentMult, 100),
        ];
        // (100 + 20) * 1.5 = 180, then * 1.1 = 198
        assert_eq!(evaluate(&sheet(HP, 100), &mods)[&HP], 198);
    }

    #[test]
    fn collection_order_does_not_matter() {
        let mods = [
            modifier(HP, ModifierKind::PercentMult, 100),
            modifier(HP, ModifierKind::PercentMult, 300),
            modifier(HP, ModifierKind::Flat, 7),
            modifier(HP, ModifierKind::PercentAdd, 250),
            modifier(HP, ModifierKind::PercentAdd, -50),
        ];
        let mut reversed = mods;
        reversed.reverse();
        let base = sheet(HP, 101);
        assert_eq!(evaluate(&base, &mods), evaluate(&base, &reversed));
    }

    #[test]
    fn stats_without_base_start_at_zero() {
        let mods = [modifier(SPEED, ModifierKind::Flat, 5)];
        assert_eq!(evaluate(&StatSheet::new(), &mods)[&SPEED], 5);
    }

    #[test]
    fn stats_without_modifiers_keep_base() {
        assert_eq!(evaluate(&sheet(HP, 42), &[])[&HP], 42);
    }

    #[test]
    fn modifiers_only_touch_their_stat() {
        let mods = [modifier(HP, ModifierKind::Flat, 10)];
        let base = StatSheet::from([(HP, 1), (SPEED, 1)]);
        let result = evaluate(&base, &mods);
        assert_eq!((result[&HP], result[&SPEED]), (11, 1));
    }

    #[test]
    fn percent_below_minus_100_floors_at_zero() {
        let mods = [modifier(HP, ModifierKind::PercentAdd, -5000)];
        assert_eq!(evaluate(&sheet(HP, 100), &mods)[&HP], 0);
    }

    #[test]
    fn overflow_saturates() {
        let mods = [modifier(HP, ModifierKind::Flat, i64::MAX)];
        assert_eq!(evaluate(&sheet(HP, i64::MAX), &mods)[&HP], i64::MAX);
        let mult = [modifier(HP, ModifierKind::PercentMult, 1_000_000)];
        assert_eq!(evaluate(&sheet(HP, i64::MAX), &mult)[&HP], i64::MAX);
    }
}
