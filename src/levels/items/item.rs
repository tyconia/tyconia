use super::recipe::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::path::PathBuf;

#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct ItemRuntime {
    item: ItemId,
    stack_size: StackSize,
    unlocked: bool,
    recipes: Recipes,

    #[reflect(ignore)]
    image: Handle<Image>,
}

#[derive(Resource, Clone, PartialEq, Default)]
pub struct ItemMap(HashMap<(ItemId, super::Meta), ItemRuntime>);

/// Represents an item.
/// Item unlock/lock depends on available recipes
#[derive(Component, Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub struct ItemId {
    pub display_name: String,
    pub snake_name: String,
}

/// Maximum amount of x item per stack
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct StackSize(pub usize);

impl Default for StackSize {
    fn default() -> Self {
        Self(10)
    }
}
