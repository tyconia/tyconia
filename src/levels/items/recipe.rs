use super::ItemId;
use bevy::prelude::*;

/// Recipes that can be be used to craft this item
#[derive(Component, Clone, Debug, PartialEq, Hash, Reflect)]
pub struct Recipes(pub Vec<Recipe>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct Recipe {
    // required ingredients for this recipe
    pub ingredients: Vec<(ItemId, usize)>,
    // other items produced by this recipe
    pub by_products: Option<Vec<(ItemId, usize)>>,
    // number of items produced by this recipe
    pub output_quantity: usize,

    pub research_required: Vec<super::ResearchId>,
}
