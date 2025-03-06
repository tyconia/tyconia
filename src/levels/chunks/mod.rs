use crate::actions::CursorWorldPosition;
use crate::loading::TextureAssets;
use std::time::Duration;

use crate::GameState;
use bevy::prelude::*;
use bevy::time::common_conditions::*;
use bevy_ecs_tilemap::prelude::*;

mod surfaces;
pub use surfaces::*;

mod textures;
pub use textures::*;

pub struct ChunkPlugin;

impl ChunkPlugin {
    pub const FLOOR_LAYER: f32 = 1.;
    pub const COUNTERTOP_LAYER: f32 = 2.;
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TilemapPlugin, SurfacesPlugin))
            .add_systems(
                OnEnter(GameState::Playing),
                (render_floors, render_countertops)
                    .chain()
                    .after(crate::loading::load_item_asset_map),
            )
            .add_systems(
                Update,
                cycle_textures
                    .run_if(in_state(GameState::Playing).and(on_timer(Duration::from_secs(2)))),
            );
    }
}

pub fn cycle_textures(mut tilemap_textures: Query<&mut TilemapTexture>) {
    for mut tilemap_texture in tilemap_textures.iter_mut() {
        if let TilemapTexture::Vector(ref mut textures) = &mut *tilemap_texture {
            if let Some(last) = textures.pop() {
                textures.insert(0, last);
            }
        }
    }
}

#[derive(Component)]
pub struct HighlightedTile;

pub type HighlightedtilesQuery<'a, 'b, 'c> =
    Query<'a, 'b, (Entity, &'c mut TileColor, Option<&'c OriginalColor>), With<HighlightedTile>>;

pub type TilemapQuery<'a, 'b, 'c, TilemapFilters> = Query<
    'a,
    'b,
    (
        Entity,
        &'c TilemapSize,
        &'c TilemapGridSize,
        &'c TilemapType,
        &'c TileStorage,
        &'c Transform,
    ),
    TilemapFilters,
>;

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

// marker component for objects on top of the floor tilemap
#[derive(Component)]
pub struct BuildingTilemap;

pub type BuildingTilemapQuery<'a, 'b, 'c> =
    TilemapQuery<'a, 'b, 'c, (With<BuildingTilemap>, Without<FloorTilemap>)>;

// marker component for the floor tilemap
#[derive(Component)]
pub struct FloorTilemap;

pub type FloorTilemapQuery<'a, 'b, 'c> =
    TilemapQuery<'a, 'b, 'c, (Without<BuildingTilemap>, With<FloorTilemap>)>;

fn render_countertops(
    mut cmd: Commands,
    textures: Res<TextureAssets>,
    item_map: Query<&crate::loading::ItemTextureMap, With<crate::Level>>,
) {
    const QUADRANT_SIDE_LENGTH: u32 = 32;
    let texture = TilemapTexture::Vector(item_map.single().to_vec());
    //let inserter: Handle<Image> = textures.isometric_inserters.clone();
    //let infinite_io: Handle<Image> = textures.infinite_io.clone();
    //let belts: Handle<Image> = textures.isometric_belts.clone();
    //let burger: Handle<Image> = textures.burger.clone();

    let map_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };

    let filled_sized = TilemapSize {
        x: QUADRANT_SIDE_LENGTH,
        y: QUADRANT_SIDE_LENGTH,
    };

    let mut floor_storage = TileStorage::empty(map_size);
    let tilemap_entity = cmd.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    for x in 18..20 {
        for y in 0..20 {
            let tile = cmd
                .spawn(TileBundle {
                    position: TilePos { x, y },
                    texture_index: TileTextureIndex(match (x % 2, y % 2) {
                        (0, _) => 1,
                        _ => 0,
                    }),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                })
                .id();

            floor_storage.set(&TilePos { x, y }, tile);
        }
    }
    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = (TilemapTileSize { x: 32.0, y: 16.0 }).into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    let tile_map_transform = get_tilemap_center_transform(
        &map_size,
        &grid_size,
        &map_type,
        ChunkPlugin::COUNTERTOP_LAYER,
    );

    // draw burgbers
    for y in 0..20 {
        let x = 20;
        let tile_pos = TilePos { x, y };
        let tile_center = tile_pos.center_in_world(&grid_size, &map_type);

        let mut tile_transform =
            Transform::from_translation(tile_map_transform * tile_center.extend(5.))
                .with_scale(Vec3::splat(0.3));

        tile_transform.translation.y -= 3.;

        //cmd.spawn((
        //    Sprite {
        //        image: burger.clone(),
        //        ..default()
        //    },
        //    tile_transform,
        //));

        let tile = cmd.spawn(TileBundle {
            position: tile_pos,
            texture_index: TileTextureIndex(2),

            tilemap_id: TilemapId(tilemap_entity),
            ..default()
        });

        floor_storage.set(&TilePos { x, y }, tile.id());
    }

    cmd.entity(tilemap_entity).insert((
        StateScoped(GameState::Playing),
        BuildingTilemap,
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: floor_storage.clone(),
            texture,
            tile_size,
            map_type,
            transform: tile_map_transform,
            render_settings: TilemapRenderSettings {
                y_sort: true,
                render_chunk_size: UVec2::new(3, 1),
            },
            ..Default::default()
        },
    ));
}

#[derive(Debug, Component)]
pub struct OriginalColor(pub TileColor);

fn render_floors(
    mut cmd: Commands,
    textures: Res<TextureAssets>,
    item_map: Query<&crate::loading::ItemTextureMap, With<crate::Level>>,
) {
    const QUADRANT_SIDE_LENGTH: u32 = 32;

    let texture = TilemapTexture::Vector(item_map.single().to_vec());

    let map_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };

    let mut floor_storage = TileStorage::empty(map_size);
    let tilemap_entity = cmd.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    cmd.entity(tilemap_id.0).with_children(|parent| {
        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let color = {
                    /// Size of Tile on a checkerboard pattern
                    const FLOOR_TILE_SIZE: u32 = 2;
                    let tile_x = x / FLOOR_TILE_SIZE;
                    let tile_y = y / FLOOR_TILE_SIZE;
                    //if (tile_x + tile_y) % 2 == 0 {
                    //    TileColor(Color::srgba_u8(100, 100, 100, 250))
                    //} else {
                    TileColor::default()
                    //}
                };

                let tile_pos = TilePos { x, y };

                let tile_entity = parent
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id,
                            texture_index: TileTextureIndex(0),
                            color,
                            ..Default::default()
                        },
                        OriginalColor(color),
                    ))
                    .id();
                floor_storage.set(&tile_pos, tile_entity);
            }
        }
    });

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = (TilemapTileSize { x: 32.0, y: 16.0 }).into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    cmd.entity(tilemap_entity).insert((
        StateScoped(GameState::Playing),
        FloorTilemap,
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: floor_storage.clone(),
            texture,
            tile_size,
            map_type,
            transform: get_tilemap_center_transform(
                &map_size,
                &grid_size,
                &map_type,
                ChunkPlugin::FLOOR_LAYER,
            ),
            render_settings: TilemapRenderSettings {
                y_sort: true,
                render_chunk_size: UVec2::new(3, 1),
            },
            ..Default::default()
        },
    ));
}

pub fn cursor_tile_position(
    cursor_world_position: &CursorWorldPosition,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    map_transform: &Transform,
) -> Option<TilePos> {
    // Grab the cursor position from the `Res<CursorPos>`
    let cursor_pos: Vec2 = cursor_world_position.0;
    // We need to make sure that the cursor's world position is correct relative to the map
    // due to any map transformation.
    let cursor_in_map_pos: Vec2 = {
        // Extend the cursor_pos vec3 by 0.0 and 1.0
        let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
        let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
        cursor_in_map_pos.xy()
    };

    // Once we have a world position we can transform it into a possible tile position.
    TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
}

pub trait CursorTilemap {
    fn cursor_tile_position<'a>(
        &'a self,
        cursor_world_position: &CursorWorldPosition,
    ) -> (Entity, &'a TileStorage, Option<TilePos>);
}

impl<'a, 'b, 'c, TilemapFilter: bevy::ecs::query::QueryFilter> CursorTilemap
    for TilemapQuery<'a, 'b, 'c, TilemapFilter>
{
    fn cursor_tile_position(
        &self,
        cursor_world_position: &CursorWorldPosition,
    ) -> (Entity, &TileStorage, Option<TilePos>) {
        let (entity, map_size, grid_size, map_type, tile_storage, map_transform) = self.single();

        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_world_position.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.

        let tile_pos = TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type);

        (entity, tile_storage, tile_pos)
    }
}

pub trait CursorTilemapMut {
    fn cursor_tile_position<'a>(
        &'a mut self,
        cursor_world_position: &CursorWorldPosition,
    ) -> (Entity, Mut<'a, TileStorage>, Option<TilePos>);
}

impl<'a, 'b, 'c, TilemapFilter: bevy::ecs::query::QueryFilter> CursorTilemapMut
    for TilemapQueryMut<'a, 'b, 'c, TilemapFilter>
{
    fn cursor_tile_position(
        &mut self,
        cursor_world_position: &CursorWorldPosition,
    ) -> (Entity, Mut<TileStorage>, Option<TilePos>) {
        let (entity, map_size, grid_size, map_type, tile_storage, map_transform) =
            self.single_mut();

        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_world_position.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.

        let tile_pos = TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type);

        (entity, tile_storage, tile_pos)
    }
}
