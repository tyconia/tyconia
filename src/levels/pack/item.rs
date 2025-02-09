use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::asset_collection::AssetCollection;

/// Represents an item.
/// Item unlock/lock depends on available recipes
#[derive(Component, Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub struct ItemId(pub String);

#[derive(Resource, Debug, PartialEq, Clone, Reflect, Eq)]
pub struct ItemThumbnail(pub HashMap<ItemId, Handle<Image>>);

impl AssetCollection for ItemThumbnail {
    fn load(world: &mut World) -> Vec<UntypedHandle> {
        vec![]
    }

    fn create(world: &mut World) -> Self {
        world.resource_scope(
            |world, assets: bevy::prelude::Mut<bevy_asset_loader::dynamic_asset::DynamicAssets>| {},
        );
        Self(HashMap::new())
    }
}

#[derive(Component, Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub struct ItemCategory(pub String);

#[derive(Component, Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub struct ItemEntry {
    pub item: ItemId,
    pub quantity: usize,
}

#[derive(Component, Default, Debug, PartialEq, Clone, Reflect, Eq, Hash)]
#[require(InventoryActive)]
pub struct Inventory(pub Vec<Option<ItemEntry>>);

// current item selected in inventory
// with this removed, inventory is considered read-only
#[derive(Component, Default, Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub struct InventoryActive(pub Option<usize>);

impl Inventory {
    pub fn with_capacity(capacity: usize) -> Self {
        Self((0..capacity).enumerate().map(|_| None).collect())
    }

    pub fn dump(&mut self, item_entries: Vec<ItemEntry>) {
        let mut item_entries = item_entries.into_iter();

        for slot in self.0.iter_mut() {
            if slot.is_none() {
                if let Some(item_entry) = item_entries.next() {
                    *slot = Some(item_entry);
                } else {
                    break;
                }
            }
        }
    }
}

/// Maximum amount of x item per stack
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct StackSize(pub usize);

impl Default for StackSize {
    fn default() -> Self {
        Self(10)
    }
}
