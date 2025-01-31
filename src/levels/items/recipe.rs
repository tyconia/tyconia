use super::ItemId;
use bevy::prelude::*;

/// Recipes that can be be used to craft this item
#[derive(Component, Clone, Debug, PartialEq, Hash, Reflect)]
pub struct Recipes(pub Vec<Recipe>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct RecipeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct Recipe {
    // required ingredients for this recipe
    pub ingredients: Vec<super::ItemEntry>,

    // items produced by this recipe
    pub output: Vec<super::ItemEntry>,

    pub research_required: Vec<super::ResearchId>,

    /// duration in milliseconds
    pub duration: u32,
}
