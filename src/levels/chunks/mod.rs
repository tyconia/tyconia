use crate::actions::CursorWorldPosition;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub type TilemapQueryMut<'a, 'b, 'c, TilemapFilters> = Query<
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
pub struct BuildingTilemap;

#[derive(Component)]
pub struct FloorTilemap;

pub trait CursorTilemapMut {
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
