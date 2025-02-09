use crate::{actions::*, *};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

/// allows an inventory to interact with the world
pub struct InventoryInteractWorldPlugin;

impl Plugin for InventoryInteractWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (inventory_interact_world,)
                .run_if(in_state(InGameState::Normal).and(any_with_component::<InventoryActive>)),
        );
    }
}

/// marker for inventory where world interactions for buildings account from
//#[derive(Component)]
//pub struct InventoryBuildingSource;

fn inventory_interact_world(
    mut cmd: Commands,
    mut inter_actions: EventReader<actions::InterAction>,
    cursor: Res<CursorWorldPosition>,
    mut inventory: Query<(&mut Inventory, &InventoryActive)>,
    mut building_tilemap: TilemapQueryMut<(With<BuildingTilemap>, Without<FloorTilemap>)>,
) {
    inter_actions
        .read()
        .for_each(|inter_action| match *inter_action {
            InterAction::Construct => {
                let (building_tilemap_entity, mut tile_storage, tile_pos) =
                    building_tilemap.cursor_tile_position(&*cursor);
                let (inventory, inventory_active) = inventory.single_mut();

                inventory_active.0.map(|mut active| {
                    inventory.0.get(active).map(|mut inventory_slot| {
                        //
                        if let &Some(ItemEntry { item, quantity }) = &inventory_slot {
                            if let Some(tile_pos) = tile_pos {
                                let texture_index = match item.0.as_str() {
                                    "auto_arm" => Some(0),
                                    "infinite_io" => Some(1),
                                    "mover_belt" => Some(2),
                                    _ => None,
                                };
                                texture_index.map(|index| {
                                    let tile = cmd
                                        .spawn(TileBundle {
                                            position: tile_pos,
                                            texture_index: TileTextureIndex(index),

                                            tilemap_id: TilemapId(building_tilemap_entity),
                                            ..default()
                                        })
                                        .id();

                                    tile_storage.set(&tile_pos, tile);
                                });
                            }
                        }
                    })
                });
            }
            InterAction::Deconstruct => {}
            InterAction::Distribute => {}
            InterAction::Pipette => {}
            InterAction::CopyConfiguration => {}
            InterAction::PasteConfiguration => {}
        });
}
