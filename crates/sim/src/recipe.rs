//! Recipe graph. Items without a recipe are raw materials.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::items::ItemId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct StationId(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDef {
    pub output: ItemId,
    /// How many items one craft produces. Must not be zero.
    pub output_count: u32,
    pub inputs: Vec<(ItemId, u32)>,
    pub station: StationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeError {
    /// The cycle in order, first item repeated at the end.
    Cycle(Vec<ItemId>),
    ZeroOutput(ItemId),
    CostOverflow(ItemId),
}

impl fmt::Display for RecipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "recipe error: {self:?}")
    }
}

impl std::error::Error for RecipeError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeBook {
    recipes: BTreeMap<ItemId, RecipeDef>,
}

impl RecipeBook {
    pub fn recipe(&self, output: ItemId) -> Option<&RecipeDef> {
        self.recipes.get(&output)
    }

    /// Adds a recipe, replacing any earlier one for the same output.
    pub fn insert(&mut self, recipe: RecipeDef) -> Result<(), RecipeError> {
        if recipe.output_count == 0 {
            return Err(RecipeError::ZeroOutput(recipe.output));
        }
        self.recipes.insert(recipe.output, recipe);
        Ok(())
    }

    /// Raw materials needed to craft `quantity` of `item`. Crafts are whole:
    /// a recipe that yields 2 and is asked for 3 runs twice.
    pub fn raw_cost(
        &self,
        item: ItemId,
        quantity: u64,
    ) -> Result<BTreeMap<ItemId, u64>, RecipeError> {
        self.visit(item, &mut Vec::new(), &mut BTreeSet::new())?;
        let mut cost = BTreeMap::new();
        self.collect_cost(item, quantity, &mut cost)?;
        Ok(cost)
    }

    fn collect_cost(
        &self,
        item: ItemId,
        quantity: u64,
        cost: &mut BTreeMap<ItemId, u64>,
    ) -> Result<(), RecipeError> {
        let Some(recipe) = self.recipes.get(&item) else {
            let total = cost.entry(item).or_insert(0);
            *total = total
                .checked_add(quantity)
                .ok_or(RecipeError::CostOverflow(item))?;
            return Ok(());
        };
        let crafts = quantity.div_ceil(u64::from(recipe.output_count));
        for (input, per_craft) in &recipe.inputs {
            let needed = crafts
                .checked_mul(u64::from(*per_craft))
                .ok_or(RecipeError::CostOverflow(item))?;
            self.collect_cost(*input, needed, cost)?;
        }
        Ok(())
    }

    /// Fails with the offending cycle if any recipe depends on itself.
    pub fn validate(&self) -> Result<(), RecipeError> {
        let mut done = BTreeSet::new();
        for output in self.recipes.keys() {
            self.visit(*output, &mut Vec::new(), &mut done)?;
        }
        Ok(())
    }

    fn visit(
        &self,
        item: ItemId,
        path: &mut Vec<ItemId>,
        done: &mut BTreeSet<ItemId>,
    ) -> Result<(), RecipeError> {
        if done.contains(&item) {
            return Ok(());
        }
        if let Some(start) = path.iter().position(|seen| *seen == item) {
            let mut cycle = path[start..].to_vec();
            cycle.push(item);
            return Err(RecipeError::Cycle(cycle));
        }
        if let Some(recipe) = self.recipes.get(&item) {
            path.push(item);
            for (input, _) in &recipe.inputs {
                self.visit(*input, path, done)?;
            }
            path.pop();
        }
        done.insert(item);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLANK: ItemId = ItemId(1);
    const LOG: ItemId = ItemId(2);
    const AXE: ItemId = ItemId(3);

    fn recipe(output: ItemId, inputs: &[(ItemId, u32)]) -> RecipeDef {
        RecipeDef {
            output,
            output_count: 1,
            inputs: inputs.to_vec(),
            station: StationId(0),
        }
    }

    #[test]
    fn acyclic_book_validates() {
        let mut book = RecipeBook::default();
        book.insert(recipe(PLANK, &[(LOG, 1)])).unwrap();
        book.insert(recipe(AXE, &[(PLANK, 2), (LOG, 1)])).unwrap();
        assert_eq!(book.validate(), Ok(()));
    }

    #[test]
    fn direct_cycle_is_reported() {
        let mut book = RecipeBook::default();
        book.insert(recipe(PLANK, &[(PLANK, 1)])).unwrap();
        assert_eq!(book.validate(), Err(RecipeError::Cycle(vec![PLANK, PLANK])));
    }

    #[test]
    fn indirect_cycle_is_reported_in_order() {
        let mut book = RecipeBook::default();
        book.insert(recipe(PLANK, &[(AXE, 1)])).unwrap();
        book.insert(recipe(AXE, &[(PLANK, 1)])).unwrap();
        assert_eq!(
            book.validate(),
            Err(RecipeError::Cycle(vec![PLANK, AXE, PLANK]))
        );
    }

    #[test]
    fn shared_ingredients_are_not_cycles() {
        let mut book = RecipeBook::default();
        book.insert(recipe(PLANK, &[(LOG, 1)])).unwrap();
        book.insert(recipe(AXE, &[(PLANK, 1), (LOG, 1)])).unwrap();
        book.insert(recipe(ItemId(4), &[(AXE, 1), (PLANK, 1)]))
            .unwrap();
        assert_eq!(book.validate(), Ok(()));
    }

    #[test]
    fn zero_output_is_rejected() {
        let mut book = RecipeBook::default();
        let mut bad = recipe(PLANK, &[(LOG, 1)]);
        bad.output_count = 0;
        assert_eq!(book.insert(bad), Err(RecipeError::ZeroOutput(PLANK)));
    }

    #[test]
    fn raw_material_costs_itself() {
        let book = RecipeBook::default();
        assert_eq!(book.raw_cost(LOG, 3), Ok(BTreeMap::from([(LOG, 3)])));
    }

    #[test]
    fn raw_cost_rolls_up_nested_recipes() {
        let mut book = RecipeBook::default();
        book.insert(recipe(PLANK, &[(LOG, 1)])).unwrap();
        book.insert(recipe(AXE, &[(PLANK, 2), (LOG, 1)])).unwrap();
        // 2 planks = 2 logs, plus 1 log
        assert_eq!(book.raw_cost(AXE, 1), Ok(BTreeMap::from([(LOG, 3)])));
        assert_eq!(book.raw_cost(AXE, 4), Ok(BTreeMap::from([(LOG, 12)])));
    }

    #[test]
    fn crafts_are_whole() {
        let mut book = RecipeBook::default();
        let mut planks = recipe(PLANK, &[(LOG, 1)]);
        planks.output_count = 2;
        book.insert(planks).unwrap();
        assert_eq!(book.raw_cost(PLANK, 3), Ok(BTreeMap::from([(LOG, 2)])));
    }

    #[test]
    fn raw_cost_reports_cycles_instead_of_looping() {
        let mut book = RecipeBook::default();
        book.insert(recipe(PLANK, &[(AXE, 1)])).unwrap();
        book.insert(recipe(AXE, &[(PLANK, 1)])).unwrap();
        assert!(matches!(book.raw_cost(AXE, 1), Err(RecipeError::Cycle(_))));
    }

    #[test]
    fn raw_cost_detects_overflow() {
        let mut book = RecipeBook::default();
        book.insert(recipe(PLANK, &[(LOG, u32::MAX)])).unwrap();
        assert_eq!(
            book.raw_cost(PLANK, u64::MAX),
            Err(RecipeError::CostOverflow(PLANK))
        );
    }
}
