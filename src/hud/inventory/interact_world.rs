use crate::{actions::*, *};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

type TilemapQueryMut<'a, 'b, 'c, TilemapFilters> = Query<
    'a,
    'b,
    (
        Entity,
        &'c TilemapSize,
        &'c TilemapGridSize,
        &'c TilemapType,
        &'c mut TileStorage,
        &'c Transform,
    ),
    TilemapFilters,
>;

#[derive(Component)]
struct BuildingTilemap;

#[derive(Component)]
struct FloorTilemap;

trait CursorTilemapMut {
    fn cursor_tile_position<'a>(
        &'a mut self,
        cursor_world_position: &CursorWorldPosition,
    ) -> Option<(Entity, Mut<'a, TileStorage>, Option<TilePos>)>;
}

impl<'a, 'b, 'c, TilemapFilter: bevy::ecs::query::QueryFilter> CursorTilemapMut
    for TilemapQueryMut<'a, 'b, 'c, TilemapFilter>
{
    fn cursor_tile_position<'s>(
        &'s mut self,
        cursor_world_position: &CursorWorldPosition,
    ) -> Option<(Entity, Mut<'s, TileStorage>, Option<TilePos>)> {
        let Ok((entity, map_size, grid_size, map_type, tile_storage, map_transform)) =
            self.get_single_mut()
        else {
            return None;
        };

        let cursor_pos: Vec2 = cursor_world_position.0;
        let cursor_in_map_pos: Vec2 = {
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };

        let tile_pos = TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type);

        Some((entity, tile_storage, tile_pos))
    }
}

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
    let Ok((inventory, inventory_active)) = inventory.get_single_mut() else {
        return;
    };

    inter_actions
        .read()
        .for_each(|inter_action| match *inter_action {
            InterAction::Construct => {
                let Some((building_tilemap_entity, mut tile_storage, tile_pos)) =
                    building_tilemap.cursor_tile_position(&*cursor)
                else {
                    return;
                };

                inventory_active.0.map(|active| {
                    inventory.0.get(active).map(|inventory_slot| {
                        //
                        if let &Some(ItemEntry { item, quantity: _ }) = &inventory_slot {
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
